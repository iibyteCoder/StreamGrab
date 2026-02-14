//! 下载相关命令
//!
//! 封装 N_m3u8DL-RE 进程的启动、停止、暂停、恢复等操作

use std::process::Command;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
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
/// * `program_path` - N_m3u8DL-RE 程序绝对路径（必须配置）
#[tauri::command(rename_all = "camelCase")]
pub async fn start_download(
    task_id: String,
    url: String,
    args: Vec<String>,
    save_dir: String,
    save_name: Option<String>,
    program_path: Option<String>,
    app: AppHandle,
) -> Result<(), String> {
    log::info!(
        "Starting download: task_id={}, url={}",
        task_id,
        url
    );

    let manager = PROCESS_MANAGER.clone();

    // 获取 N_m3u8DL-RE 程序路径（必须在设置中配置绝对路径）
    let program_path = match program_path {
        Some(path) if !path.is_empty() => {
            log::info!("Using N_m3u8DL-RE path: {}", path);
            path
        }
        _ => {
            return Err("N_m3u8DL-RE 路径未配置，请在设置中配置 N_m3u8DL-RE 的绝对路径".to_string());
        }
    };

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
    program_path: Option<String>,
    app: AppHandle,
) -> Result<(), String> {
    log::info!("Resuming download: task_id={}", task_id);

    // 恢复下载实际上就是重新启动下载
    // 前端需要在暂停时保存当前进度
    start_download(task_id, url, args, save_dir, save_name, program_path, app).await
}

/// 解析 URL 获取流信息
///
/// # Arguments
/// * `url` - 视频 URL
/// * `use_proxy` - 是否使用系统代理
/// * `custom_proxy` - 自定义代理地址
/// * `headers` - 请求头
/// * `program_path` - N_m3u8DL-RE 程序路径
/// * `app` - Tauri AppHandle
#[tauri::command(rename_all = "camelCase")]
pub async fn parse_url(
    url: String,
    use_proxy: bool,
    custom_proxy: Option<String>,
    headers: Vec<HeaderItem>,
    program_path: Option<String>,
    _app: AppHandle,
) -> Result<StreamInfo, String> {
    log::info!("Parsing URL: {}", url);

    // 获取 N_m3u8DL-RE 程序路径
    let program = match program_path {
        Some(path) if !path.is_empty() => path,
        _ => {
            return Err("N_m3u8DL-RE 路径未配置，请在设置中配置 N_m3u8DL-RE 的绝对路径".to_string());
        }
    };

    // 创建临时目录用于存储解析结果
    let temp_dir = std::env::temp_dir().join("streamgrab_parse");
    let _ = std::fs::create_dir_all(&temp_dir);

    // 生成唯一的文件名
    let parse_id = format!("parse_{}", chrono::Utc::now().timestamp_millis());

    // 构建命令行参数
    let mut args: Vec<String> = vec![
        url.clone(),
        "--skip-download".to_string(),           // 只解析不下载
        "--write-meta-json".to_string(),         // 输出 JSON 元数据
        format!("--tmp-dir={}", temp_dir.display()), // 临时目录
        format!("--save-name={}", parse_id),     // 保存名称
        "--no-log".to_string(),                  // 禁用日志文件
    ];

    // 添加代理设置
    if let Some(proxy) = custom_proxy {
        if !proxy.is_empty() {
            args.push(format!("--custom-proxy={}", proxy));
        }
    } else if use_proxy {
        args.push("--use-system-proxy".to_string());
    }

    // 添加请求头
    for header in headers {
        if header.enabled {
            args.push("-H".to_string());
            args.push(format!("{}: {}", header.key, header.value));
        }
    }

    log::info!("Running parse command: {} {:?}", program, args);

    // 执行命令
    let output = Command::new(&program)
        .args(&args)
        .output()
        .map_err(|e| format!("执行解析命令失败: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    log::debug!("Parse stdout: {}", stdout);
    if !stderr.is_empty() {
        log::warn!("Parse stderr: {}", stderr);
    }

    if !output.status.success() {
        return Err(format!("解析失败: {}", if stderr.is_empty() { stdout } else { stderr }));
    }

    // 读取生成的 JSON 文件
    // N_m3u8DL-RE 会生成 <save_name>.json 文件
    let json_path = temp_dir.join(format!("{}.json", parse_id));

    // 尝试查找 JSON 文件（可能有不同的命名方式）
    let json_path = if json_path.exists() {
        json_path
    } else {
        // 查找目录中的任何 JSON 文件
        let mut found = None;
        if let Ok(entries) = std::fs::read_dir(&temp_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "json").unwrap_or(false)
                    && path.file_name().map(|n| n.to_string_lossy().contains(&parse_id)).unwrap_or(false)
                {
                    found = Some(path);
                    break;
                }
            }
        }
        found.unwrap_or(json_path)
    };

    // 解析 JSON 文件
    let stream_info = parse_meta_json(&json_path, &url)?;

    // 清理临时文件
    let _ = std::fs::remove_file(&json_path);

    Ok(stream_info)
}

/// 解析 N_m3u8DL-RE 生成的元数据 JSON 文件
fn parse_meta_json(json_path: &std::path::Path, original_url: &str) -> Result<StreamInfo, String> {
    if !json_path.exists() {
        log::warn!("Meta JSON file not found: {:?}", json_path);
        // 返回空的流信息而不是错误
        return Ok(StreamInfo {
            videos: vec![],
            audios: vec![],
            subtitles: vec![],
            duration: 0.0,
            segment_count: 0,
            is_live: false,
            is_encrypted: false,
        });
    }

    let json_content = std::fs::read_to_string(json_path)
        .map_err(|e| format!("读取元数据文件失败: {}", e))?;

    let meta: serde_json::Value = serde_json::from_str(&json_content)
        .map_err(|e| format!("解析元数据 JSON 失败: {}", e))?;

    log::debug!("Parsed meta JSON: {:?}", meta);

    // 提取流信息
    let mut videos: Vec<VideoStream> = vec![];
    let mut audios: Vec<AudioStream> = vec![];
    let mut subtitles: Vec<SubtitleStream> = vec![];

    // 解析视频流
    if let Some(streams) = meta.get("Streams").and_then(|s| s.as_array()) {
        for (idx, stream) in streams.iter().enumerate() {
            let media_type = stream.get("MediaType").and_then(|m| m.as_str()).unwrap_or("");

            let base = BaseStream {
                id: stream.get("Id").and_then(|i| i.as_str()).unwrap_or(&idx.to_string()).to_string(),
                bandwidth: stream.get("Bandwidth").and_then(|b| b.as_u64()).unwrap_or(0) as u32,
                codecs: stream.get("Codecs").and_then(|c| c.as_str()).unwrap_or("").to_string(),
                language: stream.get("Language").and_then(|l| l.as_str()).unwrap_or("").to_string(),
                name: stream.get("Name").and_then(|n| n.as_str()).unwrap_or("").to_string(),
                group_id: stream.get("GroupId").and_then(|g| g.as_str()).map(|s| s.to_string()),
                selected: None,
            };

            match media_type {
                "Video" => {
                    let resolution = stream.get("Resolution").and_then(|r| r.as_str()).unwrap_or("");
                    let (width, height) = parse_resolution(resolution);

                    videos.push(VideoStream {
                        base,
                        resolution: resolution.to_string(),
                        width,
                        height,
                        frame_rate: stream.get("FrameRate").and_then(|f| f.as_f64()).unwrap_or(0.0) as f32,
                        video_range: stream.get("VideoRange").and_then(|v| v.as_str()).unwrap_or("SDR").to_string(),
                    });
                }
                "Audio" => {
                    audios.push(AudioStream {
                        base,
                        channels: stream.get("Channels").and_then(|c| c.as_str()).unwrap_or("2").to_string(),
                        sample_rate: 0, // JSON 中可能没有这个字段
                        is_default: false,
                    });
                }
                "Subtitles" => {
                    subtitles.push(SubtitleStream {
                        base,
                        format: "srt".to_string(), // 默认格式
                        is_default: false,
                        is_forced: false,
                    });
                }
                _ => {}
            }
        }
    }

    // 提取其他信息
    let duration = meta.get("Duration").and_then(|d| d.as_f64()).unwrap_or(0.0);
    let is_live = meta.get("IsLive").and_then(|l| l.as_bool()).unwrap_or(false);
    let is_encrypted = meta.get("IsEncrypted").and_then(|e| e.as_bool()).unwrap_or(false);

    // 计算分片数量
    let segment_count = meta.get("SegmentCount")
        .and_then(|s| s.as_u64())
        .unwrap_or(
            videos.iter().map(|v| v.base.bandwidth).max().unwrap_or(0) as u64
        ) as u32;

    // 如果没有解析到任何流，尝试从原始数据中提取
    if videos.is_empty() && audios.is_empty() && subtitles.is_empty() {
        log::warn!("No streams found in meta JSON, original URL: {}", original_url);
    }

    Ok(StreamInfo {
        videos,
        audios,
        subtitles,
        duration,
        segment_count,
        is_live,
        is_encrypted,
    })
}

/// 解析分辨率字符串，例如 "1920x1080" -> (1920, 1080)
fn parse_resolution(resolution: &str) -> (u32, u32) {
    let parts: Vec<&str> = resolution.split('x').collect();
    if parts.len() == 2 {
        let width = parts[0].parse().unwrap_or(0);
        let height = parts[1].parse().unwrap_or(0);
        (width, height)
    } else {
        (0, 0)
    }
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

/// 流信息 - 与前端类型匹配
#[derive(Debug, serde::Serialize)]
pub struct StreamInfo {
    pub videos: Vec<VideoStream>,
    pub audios: Vec<AudioStream>,
    pub subtitles: Vec<SubtitleStream>,
    pub duration: f64,
    pub segment_count: u32,
    pub is_live: bool,
    pub is_encrypted: bool,
}

/// 基础流
#[derive(Debug, serde::Serialize)]
pub struct BaseStream {
    pub id: String,
    pub bandwidth: u32,
    pub codecs: String,
    pub language: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected: Option<bool>,
}

/// 视频流
#[derive(Debug, serde::Serialize)]
pub struct VideoStream {
    #[serde(flatten)]
    pub base: BaseStream,
    pub resolution: String,
    pub width: u32,
    pub height: u32,
    pub frame_rate: f32,
    pub video_range: String,
}

/// 音频流
#[derive(Debug, serde::Serialize)]
pub struct AudioStream {
    #[serde(flatten)]
    pub base: BaseStream,
    pub channels: String,
    pub sample_rate: u32,
    pub is_default: bool,
}

/// 字幕流
#[derive(Debug, serde::Serialize)]
pub struct SubtitleStream {
    #[serde(flatten)]
    pub base: BaseStream,
    pub format: String,
    pub is_default: bool,
    pub is_forced: bool,
}

/// 获取 N_m3u8DL-RE 版本
#[tauri::command]
pub async fn get_n_m3u8dl_version() -> Result<String, String> {
    // TODO: 实际执行命令获取版本
    Ok("N_m3u8DL-RE v1.0.0".to_string())
}
