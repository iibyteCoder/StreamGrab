//! 下载相关命令
//!
//! 封装 N_m3u8DL-RE 进程的启动、停止、暂停、恢复等操作

use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::Mutex;

use crate::process::manager::ProcessManager;
use crate::process::parser::OutputParser;

/// 进程管理器状态
static PROCESS_MANAGER: once_cell::sync::Lazy<Arc<Mutex<ProcessManager>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(ProcessManager::new())));

/// 开始下载命令
///
/// # Arguments
/// * `task_id` - 任务 ID
/// * `url` - 下载 URL
/// * `args` - 命令行参数
/// * `save_dir` - 保存目录
/// * `save_name` - 保存文件名
#[tauri::command]
pub async fn start_download(
    task_id: String,
    url: String,
    args: Vec<String>,
    save_dir: String,
    save_name: Option<String>,
    app: AppHandle,
) -> Result<(), String> {
    log::info!(
        "Starting download: task_id={}, url={}",
        task_id,
        url
    );

    let manager = PROCESS_MANAGER.clone();

    // 获取 N_m3u8DL-RE 程序路径
    let program_path = get_n_m3u8dl_path(&app)?;

    // 克隆用于回调的变量
    let task_id_clone = task_id.clone();
    let app_clone = app.clone();

    // 输出回调函数
    let on_output = move |output: String| {
        // 解析输出
        let parser = OutputParser::new();
        if let Some(event) = parser.parse(&output) {
            // 根据事件类型发送到前端
            match event.event_type.as_str() {
                "progress" => {
                    let _ = app_clone.emit(
                        &format!("download:progress:{}", task_id_clone),
                        &event.data,
                    );
                }
                "status" => {
                    let _ = app_clone.emit(
                        &format!("download:status:{}", task_id_clone),
                        &event.data,
                    );
                }
                "log" => {
                    let _ = app_clone.emit(
                        &format!("download:log:{}", task_id_clone),
                        &event.data,
                    );
                }
                _ => {}
            }
        }
    };

    // 完成回调函数
    let task_id_clone = task_id.clone();
    let app_clone = app.clone();
    let save_dir_clone = save_dir.clone();
    let save_name_clone = save_name.clone();

    let on_complete = move |success: bool, error_msg: Option<String>| {
        if success {
            // 构建输出文件路径
            let output_path = if let Some(name) = &save_name_clone {
                format!("{}/{}", save_dir_clone, name)
            } else {
                save_dir_clone.clone()
            };

            let _ = app_clone.emit(
                &format!("download:complete:{}", task_id_clone),
                serde_json::json!({ "outputPath": output_path }),
            );
        } else {
            let _ = app_clone.emit(
                &format!("download:error:{}", task_id_clone),
                serde_json::json!({ "message": error_msg.unwrap_or_else(|| "Unknown error".to_string()) }),
            );
        }
    };

    // 启动进程
    let mut manager_guard = manager.lock().await;
    manager_guard
        .start_process(
            task_id,
            &program_path,
            args,
            on_output,
            on_complete,
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// 停止下载命令
///
/// # Arguments
/// * `task_id` - 任务 ID
#[tauri::command]
pub async fn stop_download(task_id: String) -> Result<(), String> {
    log::info!("Stopping download: task_id={}", task_id);

    let manager = PROCESS_MANAGER.clone();
    let mut manager_guard = manager.lock().await;

    manager_guard
        .stop_process(&task_id)
        .await
        .map_err(|e| e.to_string())?;

    // 发送状态更新
    // 注意：这里不能直接发送事件，因为回调已经设置了

    Ok(())
}

/// 暂停下载命令
///
/// # Arguments
/// * `task_id` - 任务 ID
#[tauri::command]
pub async fn pause_download(task_id: String) -> Result<(), String> {
    log::info!("Pausing download: task_id={}", task_id);

    // N_m3u8DL-RE 本身不支持暂停，这里我们通过停止进程实现
    // 实际的"暂停"功能需要在前端保存状态，恢复时重新启动

    let manager = PROCESS_MANAGER.clone();
    let mut manager_guard = manager.lock().await;

    manager_guard
        .stop_process(&task_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// 恢复下载命令
///
/// # Arguments
/// * `task_id` - 任务 ID
#[tauri::command]
pub async fn resume_download(
    task_id: String,
    url: String,
    args: Vec<String>,
    save_dir: String,
    save_name: Option<String>,
    app: AppHandle,
) -> Result<(), String> {
    log::info!("Resuming download: task_id={}", task_id);

    // 恢复下载实际上就是重新启动下载
    // 前端需要在暂停时保存当前进度
    start_download(task_id, url, args, save_dir, save_name, app).await
}

/// 解析 URL 获取流信息
///
/// # Arguments
/// * `url` - 视频 URL
/// * `_use_proxy` - 是否使用系统代理 (TODO)
/// * `_custom_proxy` - 自定义代理地址 (TODO)
/// * `_headers` - 请求头 (TODO)
#[tauri::command]
pub async fn parse_url(
    url: String,
    _use_proxy: bool,
    _custom_proxy: Option<String>,
    _headers: Vec<HeaderItem>,
) -> Result<StreamInfo, String> {
    log::info!("Parsing URL: {}", url);

    // TODO: 实际实现需要调用 N_m3u8DL-RE 的 JSON 输出模式
    Ok(StreamInfo {
        url: url.clone(),
        title: None,
        duration: None,
        videos: vec![],
        audios: vec![],
        subtitles: vec![],
    })
}

/// 请求头项
#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
pub struct HeaderItem {
    pub key: String,
    pub value: String,
    #[serde(default)]
    pub enabled: bool,
}

/// 流信息
#[derive(Debug, serde::Serialize)]
pub struct StreamInfo {
    pub url: String,
    pub title: Option<String>,
    pub duration: Option<f64>,
    pub videos: Vec<StreamTrack>,
    pub audios: Vec<StreamTrack>,
    pub subtitles: Vec<StreamTrack>,
}

/// 流轨道信息
#[derive(Debug, serde::Serialize)]
pub struct StreamTrack {
    pub id: String,
    pub codec: Option<String>,
    pub bitrate: Option<u32>,
    pub resolution: Option<String>,
    pub fps: Option<f32>,
    pub language: Option<String>,
    pub channels: Option<u8>,
}

/// 获取 N_m3u8DL-RE 版本
#[tauri::command]
pub async fn get_n_m3u8dl_version() -> Result<String, String> {
    // TODO: 实际执行命令获取版本
    Ok("N_m3u8DL-RE v1.0.0".to_string())
}

/// 获取 N_m3u8DL-RE 程序路径
fn get_n_m3u8dl_path(app: &AppHandle) -> Result<String, String> {
    // 尝试从配置获取路径
    // 如果没有配置，使用默认路径或系统 PATH

    // 获取应用资源目录
    if let Some(resource_dir) = app.path().resource_dir().ok() {
        let bundled_path = resource_dir.join("bin").join("N_m3u8DL-RE.exe");
        if bundled_path.exists() {
            return Ok(bundled_path.to_string_lossy().to_string());
        }
    }

    // 使用系统 PATH 中的程序
    Ok("N_m3u8DL-RE".to_string())
}
