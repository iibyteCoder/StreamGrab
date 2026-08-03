//! 进程管理器
//!
//! 管理下载工具子进程的生命周期：启动、输出读取、停止与退出清理。
//!
//! ## 孤儿进程防护（双保险）
//!
//! - [`ProcessManager::stop_all_sync`] 经 `RunEvent::Exit` hook 在应用退出时调用
//! - `Drop` 实现兜底：管理器销毁时杀掉全部残留进程树
//!
//! 子进程被杀后管道关闭，stdout/stderr 读取线程随 EOF 自然退出。

use std::collections::{HashMap, VecDeque};
use std::io::{BufRead, BufReader, Read};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

use encoding_rs::GBK;

use crate::shared::{AppError, AppResult, ResolvedPath};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
use crate::infrastructure::platform::CREATE_NO_WINDOW;

/// 活跃进程信息
struct ProcessInfo {
    pid: u32,
    stop_flag: Arc<Mutex<bool>>,
}

/// 进程管理器
pub struct ProcessManager {
    processes: HashMap<String, ProcessInfo>,
}

/// 按 PID 终止进程树（尽力而为，失败忽略）
fn kill_pid(pid: u32) {
    #[cfg(target_os = "windows")]
    {
        // Windows: taskkill 终止进程树（含子进程，隐藏窗口）
        let _ = Command::new("taskkill")
            .args(["/F", "/T", "/PID"])
            .arg(pid.to_string())
            .creation_flags(CREATE_NO_WINDOW)
            .output();
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = Command::new("kill")
            .arg("-TERM")
            .arg(pid.to_string())
            .output();
    }
}

/// 解码进程输出字节（Windows 中文环境优先 GBK，回退 UTF-8）
fn decode_output(buf: &[u8]) -> String {
    if cfg!(target_os = "windows") {
        let (decoded, _, had_errors) = GBK.decode(buf);
        if !had_errors {
            return decoded.into_owned();
        }
    }
    String::from_utf8_lossy(buf).into_owned()
}

/// 读取停止标志（Mutex 中毒视为「已停止」，避免级联 panic）
fn read_stop_flag(flag: &Mutex<bool>) -> bool {
    *flag.lock().unwrap_or_else(|e| e.into_inner())
}

/// 输出行缓冲上限（条）。N_m3u8DL-RE 进度帧刷屏，仅保留尾部以便提取最后错误。
const MAX_BUFFERED_LINES: usize = 200;

/// 透传给用户的错误提示最大长度（字符），防刷屏
const MAX_ERROR_HINT_LEN: usize = 300;

/// 错误关键词（小写匹配；N_m3u8DL-RE 日志与 FFmpeg stderr 通用）
const ERROR_KEYWORDS: &[&str] = &[
    "error",
    "exception",
    "failed",
    "failure",
    "forbidden",
    "unauthorized",
    "status code",
    "not found",
    "refused",
    "denied",
    "timeout",
    "403",
    "404",
    "500",
    "拒绝",
    "失败",
    "错误",
    "找不到",
    "无法",
    "超时",
];

/// 从任务输出缓冲提取错误摘要；无有效信息返回 `None`
fn collect_error_hint(lines: &Mutex<VecDeque<String>>) -> Option<String> {
    let q = lines.lock().ok()?;
    extract_error_hint(&q.iter().map(String::as_str).collect::<Vec<_>>())
}

/// 从进程输出行中提取用于展示的错误摘要（纯函数，可单测）。
///
/// 策略：优先取命中 [`ERROR_KEYWORDS`] 的行（如 `... 403 (Forbidden)`、
/// `Unhandled exception: ...`、FFmpeg 的 `Error: ...`），最多取最后 3 条以 `|` 拼接；
/// 未命中时兜底最近一条「日志行」（N_m3u8DL-RE 带时间戳前缀、FFmpeg 以
/// Error/Invalid 开头），覆盖关键词未命中但仍失败的场景。
/// 结果剥离日志行前缀并截断至 [`MAX_ERROR_HINT_LEN`]。
fn extract_error_hint(lines: &[&str]) -> Option<String> {
    let hits: Vec<&str> = lines
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .filter(|s| {
            ERROR_KEYWORDS
                .iter()
                .any(|k| s.to_ascii_lowercase().contains(k))
        })
        .map(clean_log_line)
        .collect();
    if !hits.is_empty() {
        let tail = hits[hits.len().saturating_sub(3)..].join(" | ");
        return Some(truncate(&tail));
    }
    lines
        .iter()
        .rev()
        .find(|s| looks_like_log_line(s))
        .map(|s| truncate(clean_log_line(s)))
}

/// 是否像日志行（而非进度刷屏行）
fn looks_like_log_line(line: &str) -> bool {
    let t = line.trim_start();
    let lower = t.to_ascii_lowercase();
    // N_m3u8DL-RE 日志行：`01:15:05.728 WARN : ...`（HH:MM:SS 时间戳）
    (t.len() >= 3 && t.as_bytes()[0].is_ascii_digit() && t.as_bytes()[2] == b':')
        || lower.starts_with("error")
        || lower.starts_with("invalid")
        || lower.starts_with("failed")
        || lower.starts_with("unhandled")
}

/// 剥掉 N_m3u8DL-RE 日志行的时间戳/级别前缀（`01:15:05.728 WARN : ` → 内容），
/// 其余行原样返回
fn clean_log_line(line: &str) -> &str {
    let t = line.trim();
    match t.find(" : ") {
        Some(pos) => {
            let prefix = &t[..pos];
            let head = prefix.split_whitespace().next_back().unwrap_or("");
            let is_level = matches!(head, "INFO" | "WARN" | "ERROR" | "DEBUG" | "FATAL");
            let has_ts = prefix
                .as_bytes()
                .first()
                .is_some_and(|b| b.is_ascii_digit());
            if is_level || has_ts {
                t[pos + 3..].trim()
            } else {
                t
            }
        }
        None => t,
    }
}

/// 截断至 [`MAX_ERROR_HINT_LEN`] 字符（含省略号）
fn truncate(s: &str) -> String {
    if s.chars().count() <= MAX_ERROR_HINT_LEN {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(MAX_ERROR_HINT_LEN - 1).collect();
        out.push('…');
        out
    }
}

/// 组装下载失败消息：`{base}。{详细输出}`
fn format_download_error(base: &str, hint: Option<&str>) -> String {
    match hint {
        Some(h) if !h.trim().is_empty() => format!("{base}。{h}"),
        _ => base.to_string(),
    }
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            processes: HashMap::new(),
        }
    }

    /// 启动下载进程
    ///
    /// `program` 和 `working_dir` 均为已验证的 [`ResolvedPath`]（非空+绝对+存在），
    /// 由命令层构造一次往下传递，编译期保证不会收到空/相对/不存在的路径。
    /// `on_output` 逐行回调（已解码去尾换行），`on_complete(success, error)` 退出时回调。
    ///
    /// **排序保证**：`on_complete` 在进程退出且两个输出读取线程 EOF 排空
    /// （含退出瞬间倾泻的输出）之后才触发，调用方可在其开头安全地冲刷
    /// [`EngineSession::finalize`] 残余缓冲。
    pub fn start_process<F, G>(
        &mut self,
        task_id: String,
        program: &ResolvedPath,
        args: Vec<String>,
        working_dir: Option<&ResolvedPath>,
        on_output: F,
        on_complete: G,
    ) -> AppResult<()>
    where
        F: Fn(String) + Send + Sync + 'static,
        G: Fn(bool, Option<String>) + Send + Sync + 'static,
    {
        if self.processes.contains_key(&task_id) {
            return Err(AppError::process(format!("任务 {task_id} 已在运行中")));
        }

        log::info!("Starting process: {program} with args: {args:?}");

        let mut cmd = Command::new(program.as_path());
        cmd.args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(dir) = working_dir {
            cmd.current_dir(dir.as_path());
        }

        #[cfg(target_os = "windows")]
        cmd.creation_flags(CREATE_NO_WINDOW);

        let mut child = cmd
            .spawn()
            .map_err(|e| AppError::process(format!("启动进程 '{program}' 失败: {e}")))?;
        let pid = child.id();
        log::info!("Process started with PID: {pid}");

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| AppError::process("无法捕获 stdout"))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| AppError::process("无法捕获 stderr"))?;

        let stop_flag = Arc::new(Mutex::new(false));
        self.processes.insert(
            task_id.clone(),
            ProcessInfo {
                pid,
                stop_flag: Arc::clone(&stop_flag),
            },
        );

        let output_callback: Arc<dyn Fn(String) + Send + Sync> = Arc::new(on_output);
        let complete_callback: Arc<dyn Fn(bool, Option<String>) + Send + Sync> =
            Arc::new(on_complete);

        // 输出行缓冲：reader 线程逐行写入，等待线程在失败时提取错误摘要。
        // 容量有限（保留最近 N 行）、随任务生命周期消亡，无全局状态泄漏。
        let output_lines: Arc<Mutex<VecDeque<String>>> = Arc::new(Mutex::new(VecDeque::new()));
        let buffered_callback: Arc<dyn Fn(String) + Send + Sync> = {
            let lines = Arc::clone(&output_lines);
            let orig = Arc::clone(&output_callback);
            Arc::new(move |text| {
                if let Ok(mut q) = lines.lock() {
                    q.push_back(text.clone());
                    while q.len() > MAX_BUFFERED_LINES {
                        q.pop_front();
                    }
                }
                orig(text);
            })
        };

        // stdout/stderr 读取线程（子进程被杀后管道关闭，线程随 EOF 退出）
        let stdout_reader = Self::spawn_reader(
            task_id.clone(),
            "stdout",
            stdout,
            Arc::clone(&buffered_callback),
            Arc::clone(&stop_flag),
        );
        let stderr_reader = Self::spawn_reader(
            task_id.clone(),
            "stderr",
            stderr,
            Arc::clone(&buffered_callback),
            Arc::clone(&stop_flag),
        );

        // 等待线程
        thread::spawn(move || {
            let result = child.wait();
            // 先等两个输出读取线程退出（进程退出 → 管道关闭 → EOF 排空），
            // 保证 on_output 收齐全部输出——N_m3u8DL-RE 非 TTY 下将进度帧
            // 积压到进程退出瞬间一次性倾泻，若在 join 之前触发 on_complete，
            // 完成事件会抢跑在进度数据之前，前端订阅读者已注销、进度缓冲无人冲刷。
            let _ = stdout_reader.join();
            let _ = stderr_reader.join();
            match result {
                Ok(status) => {
                    let success = status.success();
                    let code = status.code().unwrap_or(-1);
                    log::info!(
                        "Process {task_id} (PID: {pid}) exited with code {code}, success: {success}"
                    );
                    if success {
                        complete_callback(true, None);
                    } else {
                        let hint = collect_error_hint(&output_lines);
                        complete_callback(
                            false,
                            Some(format_download_error(
                                &format!("进程退出码: {code}"),
                                hint.as_deref(),
                            )),
                        );
                    }
                }
                Err(e) => {
                    log::error!("Failed to wait for process {task_id}: {e}");
                    if read_stop_flag(&stop_flag) {
                        complete_callback(false, Some("下载已取消".into()));
                    } else {
                        let hint = collect_error_hint(&output_lines);
                        complete_callback(
                            false,
                            Some(format_download_error(
                                &format!("进程错误: {e}"),
                                hint.as_deref(),
                            )),
                        );
                    }
                }
            }
        });

        Ok(())
    }

    /// 输出读取线程：按 `\n` 切分解码并回调
    ///
    /// 回调接收**含行尾 `\n`** 的原始文本（不裁剪）。这是为了让
    /// [`EngineSession`] 能按 `\n` 排水内部缓冲——N_m3u8DL-RE 在非 TTY
    /// 下会把多条进度更新粘连在一行内，会话需保留 `\n` 作为完整块边界。
    ///
    /// 返回 [`JoinHandle`]：等待线程必须在触发完成回调前 join 本线程，
    /// 保证进程退出瞬间倾泻的输出（可能整块不含 `\n`）全部到达回调。
    fn spawn_reader(
        task_id: String,
        stream: &'static str,
        reader: impl Read + Send + 'static,
        callback: Arc<dyn Fn(String) + Send + Sync>,
        stop_flag: Arc<Mutex<bool>>,
    ) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let mut reader = BufReader::new(reader);
            let mut buf = Vec::new();
            loop {
                if read_stop_flag(&stop_flag) {
                    break;
                }
                buf.clear();
                match reader.read_until(b'\n', &mut buf) {
                    Ok(0) => break, // EOF
                    Ok(_) => {
                        let text = decode_output(&buf);
                        callback(text);
                    }
                    Err(e) => {
                        log::error!("Error reading {stream} for {task_id}: {e}");
                        break;
                    }
                }
            }
            // EOF：冲刷尚未以 `\n` 结尾的剩余缓冲，避免丢失最后一行/进度块
            if !buf.is_empty() {
                let text = decode_output(&buf);
                if !text.trim().is_empty() {
                    callback(text);
                }
            }
            log::debug!("{stream} reader thread exited for task {task_id}");
        })
    }

    /// 停止指定任务的进程
    pub fn stop_process(&mut self, task_id: &str) {
        if let Some(info) = self.processes.remove(task_id) {
            *info.stop_flag.lock().unwrap_or_else(|e| e.into_inner()) = true;
            kill_pid(info.pid);
            log::info!("Process {task_id} (PID: {}) stop signal sent", info.pid);
        } else {
            log::warn!("Process {task_id} not found in active processes");
        }
    }

    /// 任务是否运行中
    pub fn is_running(&self, task_id: &str) -> bool {
        self.processes.contains_key(task_id)
    }

    /// 运行中的任务数
    pub fn running_count(&self) -> usize {
        self.processes.len()
    }

    /// 停止所有进程（同步；供 Drop 与应用退出 hook 使用）
    pub fn stop_all_sync(&mut self) {
        for (task_id, info) in self.processes.drain() {
            *info.stop_flag.lock().unwrap_or_else(|e| e.into_inner()) = true;
            kill_pid(info.pid);
            log::info!("Process {task_id} (PID: {}) killed on cleanup", info.pid);
        }
    }
}

impl Drop for ProcessManager {
    fn drop(&mut self) {
        if !self.processes.is_empty() {
            log::info!(
                "ProcessManager dropping, killing {} remaining process(es)",
                self.processes.len()
            );
            self.stop_all_sync();
        }
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 真实 N_m3u8DL-RE 403 失败样本（进度刷屏 + 错误行）
    #[test]
    fn extracts_forbidden_error_from_nm3u8dl_output() {
        let lines: Vec<&str> = vec![
            "Vid 1922x1080 | 3075 Kbps ------------------------------ 0/559 0.00% -0.00Bps --:--:--",
            "01:15:05.728 WARN : Response status code does not indicate success: 403 (Forbidden).",
            "Unhandled exception: System.Exception: Download init file failed!",
        ];
        let hint = extract_error_hint(&lines).unwrap();
        assert!(hint.contains("403"), "got: {hint}");
        assert!(hint.contains("Download init file failed"), "got: {hint}");
        // 进度刷屏行不应出现在提示中
        assert!(!hint.contains("0/559"), "got: {hint}");
        // 日志级别前缀已被剥离
        assert!(!hint.contains("WARN :"), "got: {hint}");
    }

    #[test]
    fn falls_back_to_last_log_line() {
        let lines: Vec<&str> = vec![
            "Vid 1922x1080 ------------------------------ 0/559 0.00%",
            "01:15:05.728 INFO : 开始下载...",
        ];
        let hint = extract_error_hint(&lines).unwrap();
        assert!(hint.contains("开始下载"), "got: {hint}");
    }

    #[test]
    fn empty_or_progress_only_yields_none() {
        assert!(extract_error_hint(&[]).is_none());
        let lines = vec!["Vid 1922x1080 ------------------------------ 0/559 0.00% --:--:--"];
        assert!(extract_error_hint(&lines).is_none());
    }

    #[test]
    fn truncates_long_lines() {
        let long = format!("01:15:05.728 WARN : {}", "x".repeat(1000));
        let hint = extract_error_hint(&[&long]).unwrap();
        assert!(hint.chars().count() <= MAX_ERROR_HINT_LEN);
    }

    #[test]
    fn formats_error_with_and_without_hint() {
        assert_eq!(
            format_download_error("进程退出码: 1", Some("403")),
            "进程退出码: 1。403"
        );
        assert_eq!(
            format_download_error("进程退出码: 1", None),
            "进程退出码: 1"
        );
        assert_eq!(
            format_download_error("进程退出码: 1", Some("  ")),
            "进程退出码: 1"
        );
    }

    #[test]
    fn cleans_nm3u8dl_log_prefix() {
        assert_eq!(clean_log_line("01:15:05.728 WARN : 下载失败"), "下载失败");
        assert_eq!(
            clean_log_line("Unhandled exception: boom"),
            "Unhandled exception: boom"
        );
    }
}
