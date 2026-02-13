//! 进程管理器
//!
//! 管理下载进程的生命周期

use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::thread;
use tokio::sync::mpsc;

use anyhow::{Context, Result};

/// 进程信息
struct ProcessInfo {
    /// 子进程句柄
    child: Child,
    /// 停止信号发送器
    stop_tx: mpsc::Sender<()>,
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
    pub async fn start_process<F, G>(
        &mut self,
        task_id: String,
        program: &str,
        args: Vec<String>,
        on_output: F,
        on_complete: G,
    ) -> Result<()>
    where
        F: Fn(String) + Send + 'static,
        G: Fn(bool, Option<String>) + Send + 'static,
    {
        // 检查是否已有相同任务在运行
        if self.processes.contains_key(&task_id) {
            return Err(anyhow::anyhow!("Task {} is already running", task_id));
        }

        log::info!("Starting process: {} with args: {:?}", program, args);

        // 启动子进程
        let mut child = Command::new(program)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to start download process")?;

        // 获取 stdout 和 stderr
        let stdout = child.stdout.take().context("Failed to capture stdout")?;
        let _stderr = child.stderr.take().context("Failed to capture stderr")?;

        // 创建停止信号通道
        let (stop_tx, mut stop_rx) = mpsc::channel::<()>(1);

        // 保存进程信息
        self.processes.insert(
            task_id.clone(),
            ProcessInfo {
                child,
                stop_tx: stop_tx.clone(),
            },
        );

        let task_id_clone = task_id.clone();

        // 启动输出读取线程
        thread::spawn(move || {
            let mut reader = BufReader::new(stdout).lines();

            loop {
                // 检查停止信号
                if stop_rx.try_recv().is_ok() {
                    log::info!("Process {} received stop signal", task_id_clone);
                    break;
                }

                // 读取 stdout
                match reader.next() {
                    Some(Ok(line)) => {
                        on_output(line);
                    }
                    Some(Err(e)) => {
                        log::error!("Error reading stdout: {}", e);
                    }
                    None => break,
                }
            }

            log::info!("Output reader thread exited for task {}", task_id_clone);
        });

        // 等待进程完成（在后台线程中）
        let task_id_for_callback = task_id;
        thread::spawn(move || {
            let result = wait_for_process(task_id_for_callback.clone());

            match result {
                Ok(exit_code) => {
                    let success = exit_code == 0;
                    on_complete(success, None);
                }
                Err(e) => {
                    on_complete(false, Some(e.to_string()));
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
        if let Some(mut info) = self.processes.remove(task_id) {
            // 发送停止信号
            let _ = info.stop_tx.send(()).await;

            // 终止进程
            info.child.kill().context("Failed to kill process")?;

            log::info!("Process {} stopped", task_id);
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

/// 等待进程完成（简化实现）
fn wait_for_process(task_id: String) -> Result<i32> {
    log::info!("Waiting for process: {}", task_id);
    Ok(0)
}
