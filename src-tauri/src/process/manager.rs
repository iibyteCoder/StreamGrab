//! 进程管理器
//!
//! 管理下载进程的生命周期

use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

use anyhow::{bail, Context, Result};
use encoding_rs::GBK;

// Windows 平台：隐藏控制台窗口的标志
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// 进程信息
struct ProcessInfo {
    /// 进程 ID
    pid: u32,
    /// 停止信号发送器
    stop_flag: Arc<Mutex<bool>>,
}

/// 进程管理器
pub struct ProcessManager {
    /// 活跃的进程
    processes: HashMap<String, ProcessInfo>,
}

impl ProcessManager {
    /// 创建新的进程管理器
    pub fn new() -> Self {
        Self {
            processes: HashMap::new(),
        }
    }

    /// 启动下载进程
    ///
    /// # Arguments
    /// * `task_id` - 任务 ID
    /// * `program` - 程序路径
    /// * `args` - 命令行参数
    /// * `on_output` - 输出回调
    /// * `on_complete` - 完成回调
    #[allow(clippy::type_complexity)]
    pub async fn start_process<F, G>(
        &mut self,
        task_id: String,
        program: &str,
        args: Vec<String>,
        on_output: F,
        on_complete: G,
    ) -> Result<()>
    where
        F: Fn(String) + Send + Sync + 'static,
        G: Fn(bool, Option<String>) + Send + Sync + 'static,
    {
        // 检查是否已有相同任务在运行
        if self.processes.contains_key(&task_id) {
            return Err(anyhow::anyhow!("Task {} is already running", task_id));
        }

        log::info!("Starting process: {} with args: {:?}", program, args);

        // 检查程序是否存在
        if program != "N_m3u8DL-RE" {
            // 如果是绝对路径，检查文件是否存在
            let program_path = Path::new(program);
            if program_path.is_absolute() && !program_path.exists() {
                bail!(
                    "N_m3u8DL-RE program not found at specified path: {}",
                    program
                );
            }
        }

        // 启动子进程
        #[cfg(target_os = "windows")]
        let mut cmd = Command::new(program);
        #[cfg(not(target_os = "windows"))]
        let mut cmd = Command::new(program);

        cmd.args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Windows 平台：隐藏控制台窗口
        #[cfg(target_os = "windows")]
        cmd.creation_flags(CREATE_NO_WINDOW);

        let mut child = match cmd.spawn() {
            Ok(child) => child,
            Err(e) => {
                let error_msg = if program == "N_m3u8DL-RE" {
                    format!(
                        "Failed to start N_m3u8DL-RE. Please ensure it is installed and in PATH. Error: {}",
                        e
                    )
                } else {
                    format!("Failed to start download process '{}': {}", program, e)
                };
                log::error!("{}", error_msg);
                bail!("{}", error_msg);
            }
        };

        let pid = child.id();
        log::info!("Process started with PID: {}", pid);

        // 获取 stdout 和 stderr
        let stdout = child.stdout.take().context("Failed to capture stdout")?;
        let stderr = child.stderr.take().context("Failed to capture stderr")?;

        // 创建停止标志
        let stop_flag = Arc::new(Mutex::new(false));
        let stop_flag_clone = stop_flag.clone();

        // 保存进程信息
        self.processes.insert(
            task_id.clone(),
            ProcessInfo {
                pid,
                stop_flag: stop_flag.clone(),
            },
        );

        let task_id_clone = task_id.clone();

        // 将回调包装在 Arc 中以便跨线程共享
        let output_callback = Arc::new(on_output);
        let complete_callback = Arc::new(on_complete);

        // 启动 stdout 读取线程
        let output_callback_clone = Arc::clone(&output_callback);
        let stop_flag_stdout = stop_flag_clone.clone();

        thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            let mut buf = Vec::new();

            loop {
                // 检查停止标志
                if *stop_flag_stdout.lock().unwrap() {
                    log::info!("stdout reader stopped for task {}", task_id_clone);
                    break;
                }

                buf.clear();
                match reader.read_until(b'\n', &mut buf) {
                    Ok(0) => {
                        // EOF
                        break;
                    }
                    Ok(_) => {
                        // 使用 GBK 解码（Windows 中文环境）
                        // 如果 GBK 解码失败，回退到 UTF-8，再失败则使用替换字符
                        let text = if cfg!(target_os = "windows") {
                            let (decoded, _encoding, had_errors) = GBK.decode(&buf);
                            if had_errors {
                                // 回退到 UTF-8
                                String::from_utf8_lossy(&buf).into_owned()
                            } else {
                                decoded.into_owned()
                            }
                        } else {
                            String::from_utf8_lossy(&buf).into_owned()
                        };
                        let text = text.trim_end().to_string();
                        log::debug!("[STDOUT] {}", text);
                        output_callback_clone(text);
                    }
                    Err(e) => {
                        log::error!("Error reading stdout: {}", e);
                        break;
                    }
                }
            }
            log::info!("stdout reader thread exited for task {}", task_id_clone);
        });

        // 启动 stderr 读取线程
        let task_id_clone = task_id.clone();
        let output_callback_stderr = Arc::clone(&output_callback);
        let stop_flag_stderr = stop_flag_clone;

        thread::spawn(move || {
            let mut reader = BufReader::new(stderr);
            let mut buf = Vec::new();

            loop {
                // 检查停止标志
                if *stop_flag_stderr.lock().unwrap() {
                    log::info!("stderr reader stopped for task {}", task_id_clone);
                    break;
                }

                buf.clear();
                match reader.read_until(b'\n', &mut buf) {
                    Ok(0) => {
                        // EOF
                        break;
                    }
                    Ok(_) => {
                        // 使用 GBK 解码（Windows 中文环境）
                        let text = if cfg!(target_os = "windows") {
                            let (decoded, _encoding, had_errors) = GBK.decode(&buf);
                            if had_errors {
                                String::from_utf8_lossy(&buf).into_owned()
                            } else {
                                decoded.into_owned()
                            }
                        } else {
                            String::from_utf8_lossy(&buf).into_owned()
                        };
                        let text = text.trim_end().to_string();
                        log::debug!("[STDERR] {}", text);
                        // stderr 也可能包含有用信息，同样传递给回调
                        output_callback_stderr(text);
                    }
                    Err(e) => {
                        log::error!("Error reading stderr: {}", e);
                        break;
                    }
                }
            }
            log::info!("stderr reader thread exited for task {}", task_id_clone);
        });

        // 等待进程完成
        let task_id_for_wait = task_id;
        let stop_flag_wait = stop_flag;

        thread::spawn(move || {
            let result = child.wait();

            match result {
                Ok(status) => {
                    let success = status.success();
                    let exit_code = status.code().unwrap_or(-1);

                    log::info!(
                        "Process {} (PID: {}) exited with code: {}, success: {}",
                        task_id_for_wait,
                        pid,
                        exit_code,
                        success
                    );

                    if success {
                        complete_callback(true, None);
                    } else {
                        complete_callback(
                            false,
                            Some(format!("Process exited with code: {}", exit_code)),
                        );
                    }
                }
                Err(e) => {
                    log::error!("Failed to wait for process {}: {}", task_id_for_wait, e);

                    // 检查是否是被主动停止的
                    let was_stopped = *stop_flag_wait.lock().unwrap();
                    if was_stopped {
                        complete_callback(false, Some("Download cancelled".to_string()));
                    } else {
                        complete_callback(false, Some(format!("Process error: {}", e)));
                    }
                }
            }
        });

        Ok(())
    }

    /// 停止下载进程
    ///
    /// # Arguments
    /// * `task_id` - 任务 ID
    pub async fn stop_process(&mut self, task_id: &str) -> Result<()> {
        if let Some(info) = self.processes.remove(task_id) {
            // 设置停止标志
            *info.stop_flag.lock().unwrap() = true;

            // 尝试终止进程
            // 由于子进程句柄已在等待线程中，我们使用系统调用来终止
            #[cfg(target_os = "windows")]
            {
                // Windows: 使用 taskkill 终止进程树（隐藏窗口）
                let _ = Command::new("taskkill")
                    .args(["/F", "/T", "/PID"])
                    .arg(info.pid.to_string())
                    .creation_flags(CREATE_NO_WINDOW)
                    .output();
            }

            #[cfg(not(target_os = "windows"))]
            {
                // Unix: 发送 SIGTERM
                let _ = Command::new("kill")
                    .arg("-TERM")
                    .arg(info.pid.to_string())
                    .output();
            }

            log::info!("Process {} (PID: {}) stop signal sent", task_id, info.pid);
        } else {
            log::warn!("Process {} not found in active processes", task_id);
        }

        Ok(())
    }

    /// 检查进程是否在运行
    #[allow(dead_code)]
    pub fn is_running(&self, task_id: &str) -> bool {
        self.processes.contains_key(task_id)
    }

    /// 获取运行中的任务数量
    #[allow(dead_code)]
    pub fn running_count(&self) -> usize {
        self.processes.len()
    }

    /// 停止所有进程
    #[allow(dead_code)]
    pub async fn stop_all(&mut self) {
        let task_ids: Vec<String> = self.processes.keys().cloned().collect();

        for task_id in task_ids {
            let _ = self.stop_process(&task_id).await;
        }
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}
