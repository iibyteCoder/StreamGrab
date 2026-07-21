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

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

use encoding_rs::GBK;

use crate::shared::{AppError, AppResult};

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

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            processes: HashMap::new(),
        }
    }

    /// 启动下载进程
    ///
    /// 参数不含程序路径的解析——调用方（命令层/引擎）负责提供完整路径。
    /// `on_output` 逐行回调（已解码去尾换行），`on_complete(success, error)` 退出时回调。
    pub fn start_process<F, G>(
        &mut self,
        task_id: String,
        program: &str,
        args: Vec<String>,
        working_dir: Option<&str>,
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

        // 绝对路径必须存在
        let program_path = Path::new(program);
        if program_path.is_absolute() && !program_path.exists() {
            return Err(AppError::tool_not_found(format!("工具不存在: {program}")));
        }

        let mut cmd = Command::new(program);
        cmd.args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(dir) = working_dir {
            if !dir.is_empty() {
                let work_path = Path::new(dir);
                if work_path.exists() {
                    cmd.current_dir(work_path);
                } else {
                    log::warn!("Working directory does not exist, using default: {dir}");
                }
            }
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

        // stdout/stderr 读取线程（子进程被杀后管道关闭，线程随 EOF 退出）
        Self::spawn_reader(
            task_id.clone(),
            "stdout",
            stdout,
            Arc::clone(&output_callback),
            Arc::clone(&stop_flag),
        );
        Self::spawn_reader(
            task_id.clone(),
            "stderr",
            stderr,
            Arc::clone(&output_callback),
            Arc::clone(&stop_flag),
        );

        // 等待线程
        thread::spawn(move || match child.wait() {
            Ok(status) => {
                let success = status.success();
                let code = status.code().unwrap_or(-1);
                log::info!(
                    "Process {task_id} (PID: {pid}) exited with code {code}, success: {success}"
                );
                if success {
                    complete_callback(true, None);
                } else {
                    complete_callback(false, Some(format!("进程退出码: {code}")));
                }
            }
            Err(e) => {
                log::error!("Failed to wait for process {task_id}: {e}");
                if read_stop_flag(&stop_flag) {
                    complete_callback(false, Some("下载已取消".into()));
                } else {
                    complete_callback(false, Some(format!("进程错误: {e}")));
                }
            }
        });

        Ok(())
    }

    /// 输出读取线程：逐行解码并回调
    fn spawn_reader(
        task_id: String,
        stream: &'static str,
        reader: impl Read + Send + 'static,
        callback: Arc<dyn Fn(String) + Send + Sync>,
        stop_flag: Arc<Mutex<bool>>,
    ) {
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
                        callback(text.trim_end().to_string());
                    }
                    Err(e) => {
                        log::error!("Error reading {stream} for {task_id}: {e}");
                        break;
                    }
                }
            }
            log::debug!("{stream} reader thread exited for task {task_id}");
        });
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
