//! 下载相关命令
//!
//! 封装 N_m3u8DL-RE 和 FFmpeg 进程的启动、停止、暂停、恢复等操作

use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

// Windows 平台：隐藏控制台窗口的标志
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

use super::utils::get_tool_paths_from_config;
use crate::domain::download::{
    flush_progress, parse_resolution, record_progress, AudioStream, BaseStream, StreamInfo,
    SubtitleStream, UrlType, VideoStream,
};
use crate::infrastructure::process::manager::ProcessManager;
use crate::infrastructure::process::parser::OutputParser;
use crate::infrastructure::tools::{
    get_downloader_exe_path, get_ffmpeg_exe_path, get_ffprobe_exe_path,
};

/// 进程管理器状态
static PROCESS_MANAGER: once_cell::sync::Lazy<Arc<Mutex<ProcessManager>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(ProcessManager::new())));

/// 开始下载命令
#[tauri::command(rename_all = "camelCase")]
pub async fn start_download(
    task_id: String,
    url: String,
    args: Vec<String>,
    save_dir: String,
    save_name: Option<String>,
    app: AppHandle,
) -> Result<(), String> {
    log::info!("Starting download: task_id={}, url={}", task_id, url);

    let manager = PROCESS_MANAGER.clone();

    // 从配置中获取工具路径
    let tool_paths = get_tool_paths_from_config(&app);

    // 获取 N_m3u8DL-RE 可执行文件路径
    let program_path = match get_downloader_exe_path(tool_paths.downloader_dir.as_deref()) {
        Some(path) => {
            let path_str = path.to_string_lossy().to_string();
            log::info!("Using N_m3u8DL-RE: {}", path_str);
            path_str
        }
        None => {
            return Err(
                "N_m3u8DL-RE 未找到。请在设置中配置工具目录路径，或使用设置页面的【下载】按钮自动下载。".to_string(),
            );
        }
    };

    // 提取混流格式（需要在修改 args 之前）
    let mux_format = args
        .iter()
        .position(|a| a.starts_with("-M") || a == "-M")
        .and_then(|idx| args.get(idx + 1))
        .and_then(|s| {
            // 解析 format=xxx
            s.split(':')
                .find_map(|part| part.strip_prefix("format=").map(|f| f.to_lowercase()))
        });

    // 获取 FFmpeg 可执行文件路径，用于 N_m3u8DL-RE 的混流操作
    let mut final_args = args;
    if let Some(ffmpeg_path) = get_ffmpeg_exe_path(tool_paths.ffmpeg_dir.as_deref()) {
        let ffmpeg_path_str = ffmpeg_path.to_string_lossy().to_string();
        log::info!("Using FFmpeg for muxing: {}", ffmpeg_path_str);
        final_args.extend_from_slice(&["--ffmpeg-binary-path".to_string(), ffmpeg_path_str]);
    } else {
        log::warn!("FFmpeg not found, N_m3u8DL-RE may fail for muxing operations");
    }

    // 克隆用于回调的变量
    let task_id_clone = task_id.clone();
    let app_clone = app.clone();

    // 进度状态：跟踪视频和音频的分片进度
    // 用于计算总体进度，避免视频完成后音频开始时进度跳回0
    use std::sync::atomic::{AtomicU32, Ordering};
    let video_total = Arc::new(AtomicU32::new(0));
    let video_downloaded = Arc::new(AtomicU32::new(0));
    let audio_total = Arc::new(AtomicU32::new(0));
    let audio_downloaded = Arc::new(AtomicU32::new(0));

    let video_total_clone = video_total.clone();
    let video_downloaded_clone = video_downloaded.clone();
    let audio_total_clone = audio_total.clone();
    let audio_downloaded_clone = audio_downloaded.clone();

    // 输出回调函数
    let on_output = move |output: String| {
        // 解析输出
        let parser = OutputParser::new();
        if let Some(event) = parser.parse(&output) {
            // 根据事件类型发送到前端
            match event.event_type.as_str() {
                "progress" => {
                    // 获取流类型和分片信息
                    let stream_type = event
                        .data
                        .get("streamType")
                        .and_then(|s| s.as_str())
                        .unwrap_or("Vid");
                    let downloaded = event
                        .data
                        .get("downloadedSegments")
                        .and_then(|s| s.as_u64())
                        .unwrap_or(0) as u32;
                    let total = event
                        .data
                        .get("totalSegments")
                        .and_then(|s| s.as_u64())
                        .unwrap_or(0) as u32;

                    // 更新对应流的进度状态
                    if stream_type == "Vid" {
                        video_total_clone.store(total, Ordering::Relaxed);
                        video_downloaded_clone.store(downloaded, Ordering::Relaxed);
                    } else {
                        audio_total_clone.store(total, Ordering::Relaxed);
                        audio_downloaded_clone.store(downloaded, Ordering::Relaxed);
                    }

                    // 计算总体进度
                    let v_total = video_total_clone.load(Ordering::Relaxed);
                    let v_dl = video_downloaded_clone.load(Ordering::Relaxed);
                    let a_total = audio_total_clone.load(Ordering::Relaxed);
                    let a_dl = audio_downloaded_clone.load(Ordering::Relaxed);

                    let total_segments = v_total + a_total;
                    let total_downloaded = v_dl + a_dl;

                    let overall_percent = if total_segments > 0 {
                        (total_downloaded as f64 / total_segments as f64 * 100.0).round() as i32
                    } else {
                        0
                    };

                    // 发送总体进度（保留原始信息）
                    let mut progress_data = event.data.clone();
                    progress_data["overallPercent"] = serde_json::json!(overall_percent);
                    progress_data["totalDownloadedSegments"] = serde_json::json!(total_downloaded);
                    progress_data["totalSegments"] = serde_json::json!(total_segments);

                    // 记录进度历史（后端持久化）
                    let speed = event
                        .data
                        .get("speed")
                        .and_then(|s| s.as_i64())
                        .unwrap_or(0);
                    let downloaded_size = event
                        .data
                        .get("downloadedSize")
                        .and_then(|s| s.as_i64())
                        .unwrap_or(0);
                    record_progress(&task_id_clone, overall_percent, speed, downloaded_size);

                    let _ = app_clone.emit(
                        &format!("download:progress:{}", task_id_clone),
                        &progress_data,
                    );
                }
                "status" => {
                    let _ =
                        app_clone.emit(&format!("download:status:{}", task_id_clone), &event.data);
                }
                "log" => {
                    let _ = app_clone.emit(&format!("download:log:{}", task_id_clone), &event.data);
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
        // 刷新进度历史到数据库
        flush_progress(&task_id_clone);

        if success {
            // 尝试找到实际生成的输出文件
            let output_path = find_output_file(
                &save_dir_clone,
                save_name_clone.as_deref(),
                mux_format.as_deref(),
            );

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
            final_args,
            Some(&save_dir),
            on_output,
            on_complete,
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// 停止下载命令
#[tauri::command(rename_all = "camelCase")]
pub async fn stop_download(task_id: String) -> Result<(), String> {
    log::info!("Stopping download: task_id={}", task_id);

    let manager = PROCESS_MANAGER.clone();
    let mut manager_guard = manager.lock().await;

    manager_guard
        .stop_process(&task_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// 暂停下载命令
#[tauri::command(rename_all = "camelCase")]
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
#[tauri::command(rename_all = "camelCase")]
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

/// 解析 URL 获取流信息（接收完整参数数组）
/// 前端使用 buildParseArgs 构建参数，复用所有应用设置
#[tauri::command(rename_all = "camelCase")]
pub async fn parse_url(args: Vec<String>, app: AppHandle) -> Result<StreamInfo, String> {
    // 从参数中提取 URL（第一个参数）
    let url = args.first().cloned().unwrap_or_default();
    log::info!("Parsing URL: {}", url);

    // 检测 URL 类型
    let url_type = UrlType::detect(&url);
    log::info!("Detected URL type: {:?}", url_type);

    // 从配置中获取工具路径
    let tool_paths = get_tool_paths_from_config(&app);

    // 如果是 HTTP 直链视频，使用 ffmpeg 获取信息
    if url_type.needs_ffmpeg() {
        return parse_http_video_url(&url, tool_paths.ffmpeg_dir.as_deref()).await;
    }

    // 如果不是流媒体格式，返回错误
    if !url_type.is_streaming() {
        return Err("不支持的 URL 格式。请输入 M3U8、DASH 或 MSS 流媒体链接。".to_string());
    }

    // 获取 N_m3u8DL-RE 可执行文件路径
    let program = match get_downloader_exe_path(tool_paths.downloader_dir.as_deref()) {
        Some(path) => path.to_string_lossy().to_string(),
        None => {
            return Err(
                "N_m3u8DL-RE 未找到。请在设置中配置工具目录路径，或使用设置页面的【下载】按钮自动下载。".to_string(),
            );
        }
    };

    // 替换参数中的占位符（不再需要文件，但保持兼容）
    let processed_args: Vec<String> = args
        .iter()
        .map(|arg| {
            if arg == "streamgrab_parse" {
                std::env::temp_dir()
                    .join("streamgrab_parse")
                    .display()
                    .to_string()
            } else {
                arg.clone()
            }
        })
        .collect();

    log::info!("Running parse command: {} {:?}", program, processed_args);

    // 执行命令（Windows 平台隐藏窗口）
    #[cfg(target_os = "windows")]
    let output = Command::new(&program)
        .args(&processed_args)
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("执行解析命令失败: {}", e))?;

    #[cfg(not(target_os = "windows"))]
    let output = Command::new(&program)
        .args(&processed_args)
        .output()
        .map_err(|e| format!("执行解析命令失败: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    log::debug!("Parse stdout: {}", stdout);
    if !stderr.is_empty() {
        log::warn!("Parse stderr: {}", stderr);
    }

    // 检查命令是否成功
    if !output.status.success() {
        return Err(format!(
            "解析失败: {}",
            if stderr.is_empty() { &stdout } else { &stderr }
        ));
    }

    // 直接从 stdout 解析流信息（避免文件锁定问题）
    let stream_info = parse_stdout_streams(&stdout);
    log::info!(
        "Successfully parsed stream info from stdout: {} videos, {} audios",
        stream_info.videos.len(),
        stream_info.audios.len()
    );

    Ok(stream_info)
}

/// 使用 FFmpeg 解析 HTTP 直链视频
async fn parse_http_video_url(url: &str, ffmpeg_dir: Option<&str>) -> Result<StreamInfo, String> {
    // 获取 ffprobe 可执行文件路径
    let ffprobe = match get_ffprobe_exe_path(ffmpeg_dir) {
        Some(path) => path.to_string_lossy().to_string(),
        None => {
            return Err(
                "FFprobe 未找到。请在设置中配置 FFmpeg 目录路径，或使用设置页面的【下载】按钮自动下载。".to_string(),
            );
        }
    };

    log::info!("Parsing HTTP video URL with ffprobe: {}", ffprobe);

    // Windows 平台隐藏窗口
    #[cfg(target_os = "windows")]
    let output = Command::new(&ffprobe)
        .args([
            "-v",
            "quiet",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
            url,
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("执行 ffprobe 失败: {}。请确保 FFmpeg 已安装并配置正确。", e))?;

    #[cfg(not(target_os = "windows"))]
    let output = Command::new(&ffprobe)
        .args([
            "-v",
            "quiet",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
            url,
        ])
        .output()
        .map_err(|e| format!("执行 ffprobe 失败: {}。请确保 FFmpeg 已安装并配置正确。", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("解析视频信息失败: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_ffprobe_output(&stdout)
}

/// 解析 ffprobe JSON 输出
fn parse_ffprobe_output(json: &str) -> Result<StreamInfo, String> {
    let parsed: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("解析 ffprobe 输出失败: {}", e))?;

    let mut videos: Vec<VideoStream> = vec![];
    let mut audios: Vec<AudioStream> = vec![];
    let mut duration = 0.0;

    // 解析时长
    if let Some(format) = parsed.get("format") {
        if let Some(dur) = format.get("duration").and_then(|d| d.as_str()) {
            duration = dur.parse().unwrap_or(0.0);
        } else if let Some(dur) = format.get("duration").and_then(|d| d.as_f64()) {
            duration = dur;
        }
    }

    // 解析流
    if let Some(streams) = parsed.get("streams").and_then(|s| s.as_array()) {
        for stream in streams {
            let codec_type = stream
                .get("codec_type")
                .and_then(|t| t.as_str())
                .unwrap_or("");

            if codec_type == "video" {
                let width = stream.get("width").and_then(|w| w.as_u64()).unwrap_or(0) as u32;
                let height = stream.get("height").and_then(|h| h.as_u64()).unwrap_or(0) as u32;
                let codec = stream
                    .get("codec_name")
                    .and_then(|c| c.as_str())
                    .unwrap_or("")
                    .to_string();
                let bitrate = stream
                    .get("bit_rate")
                    .and_then(|b| b.as_str())
                    .and_then(|b| b.parse().ok())
                    .unwrap_or(0);

                let resolution = format!("{}x{}", width, height);

                videos.push(VideoStream {
                    base: BaseStream {
                        id: format!("video_{}", resolution),
                        bandwidth: bitrate,
                        codecs: codec,
                        language: String::new(),
                        name: resolution.clone(),
                        group_id: None,
                        selected: Some(true),
                    },
                    resolution,
                    width,
                    height,
                    frame_rate: 0.0,
                    video_range: "SDR".to_string(),
                });
            } else if codec_type == "audio" {
                let codec = stream
                    .get("codec_name")
                    .and_then(|c| c.as_str())
                    .unwrap_or("")
                    .to_string();
                let channels = stream
                    .get("channels")
                    .and_then(|c| c.as_u64())
                    .unwrap_or(2)
                    .to_string();
                let sample_rate = stream
                    .get("sample_rate")
                    .and_then(|s| s.as_str())
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                let bitrate = stream
                    .get("bit_rate")
                    .and_then(|b| b.as_str())
                    .and_then(|b| b.parse().ok())
                    .unwrap_or(0);

                audios.push(AudioStream {
                    base: BaseStream {
                        id: "audio".to_string(),
                        bandwidth: bitrate,
                        codecs: codec,
                        language: stream
                            .get("tags")
                            .and_then(|t| t.get("language"))
                            .and_then(|l| l.as_str())
                            .unwrap_or("")
                            .to_string(),
                        name: "Audio".to_string(),
                        group_id: None,
                        selected: Some(true),
                    },
                    channels,
                    sample_rate,
                    is_default: true,
                });
            }
        }
    }

    if videos.is_empty() && audios.is_empty() {
        return Err("未找到有效的视频或音频流".to_string());
    }

    Ok(StreamInfo {
        videos,
        audios,
        subtitles: vec![],
        duration,
        segment_count: 0,
        is_live: false,
        is_encrypted: false,
    })
}

/// 从 stdout 解析流信息
/// N_m3u8DL-RE 输出格式示例：
/// Vid 960x544 | 785 Kbps | mp4a.40.2 | 56 Segments | ~02m49s
/// Aud audio-32000 | Audio | 57 Segments | ~02m49s
///
/// 支持两种格式：
/// 1. 直接格式: `Vid 960x544 | 785 Kbps | ...`
/// 2. 日志格式: `21:29:59.639 INFO : Vid 960x544 | 785 Kbps | ...`
fn parse_stdout_streams(stdout: &str) -> StreamInfo {
    let mut videos: Vec<VideoStream> = vec![];
    let mut audios: Vec<AudioStream> = vec![];
    let mut subtitles: Vec<SubtitleStream> = vec![];
    let mut max_segments: u32 = 0;
    let mut duration: f64 = 0.0;

    // 日志行正则: `HH:MM:SS.mmm LEVEL : message`
    let log_line_regex =
        regex::Regex::new(r"^\d{2}:\d{2}:\d{2}\.\d+\s+(INFO|WARN|ERROR|DEBUG)\s*:\s*(.+)$")
            .unwrap();

    for line in stdout.lines() {
        let line = line.trim();

        // 去掉日志前缀，提取实际消息
        let message = if let Some(caps) = log_line_regex.captures(line) {
            caps.get(2).unwrap().as_str()
        } else {
            line
        };

        // 解析视频流: Vid 960x544 | 785 Kbps | mp4a.40.2 | 56 Segments | ~02m49s
        if message.starts_with("Vid ") || message.contains("| Vid ") {
            if let Some(video) = parse_video_line(message) {
                if video.base.bandwidth > 0 {
                    max_segments = max_segments.max(video.base.bandwidth);
                }
                videos.push(video);
            }
        }

        // 解析音频流: Aud audio-32000 | Audio | 57 Segments | ~02m49s
        if message.starts_with("Aud ") || message.contains("| Aud ") {
            if let Some(audio) = parse_audio_line(message) {
                audios.push(audio);
            }
        }

        // 解析字幕流: Sub ...
        if message.starts_with("Sub ") || message.contains("| Sub ") {
            if let Some(subtitle) = parse_subtitle_line(message) {
                subtitles.push(subtitle);
            }
        }

        // 解析时长: ~02m49s 或 Segments 信息
        if let Some(segs) = parse_segments_from_line(message) {
            max_segments = max_segments.max(segs);
        }
        if let Some(dur) = parse_duration_from_line(message) {
            duration = duration.max(dur);
        }
    }

    // 按带宽降序排序视频
    videos.sort_by(|a, b| b.base.bandwidth.cmp(&a.base.bandwidth));
    // 按带宽降序排序音频
    audios.sort_by(|a, b| b.base.bandwidth.cmp(&a.base.bandwidth));

    StreamInfo {
        videos,
        audios,
        subtitles,
        duration,
        segment_count: max_segments,
        is_live: false,
        is_encrypted: false,
    }
}

/// 解析视频行
/// 格式: Vid 960x544 | 785 Kbps | mp4a.40.2 | 56 Segments | ~02m49s
fn parse_video_line(line: &str) -> Option<VideoStream> {
    // 提取 "Vid ..." 部分
    let vid_part = if let Some(stripped) = line.strip_prefix("Vid ") {
        stripped
    } else if let Some(pos) = line.find("| Vid ") {
        &line[pos + 6..]
    } else {
        return None;
    };

    let parts: Vec<&str> = vid_part.split('|').map(|s| s.trim()).collect();
    if parts.is_empty() {
        return None;
    }

    // 第一部分是分辨率: 960x544
    let resolution = parts.first().unwrap_or(&"").to_string();
    let (width, height) = parse_resolution(&resolution);

    // 第二部分是码率: 785 Kbps
    let bandwidth = parts
        .get(1)
        .and_then(|s| s.split_whitespace().next())
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0)
        * 1000; // Kbps -> bps

    // 第三部分是编码: mp4a.40.2
    let codecs = parts.get(2).unwrap_or(&"").to_string();

    Some(VideoStream {
        base: BaseStream {
            id: resolution.clone(),
            bandwidth,
            codecs,
            language: String::new(),
            name: resolution.clone(),
            group_id: None,
            selected: None,
        },
        resolution,
        width,
        height,
        frame_rate: 0.0,
        video_range: "SDR".to_string(),
    })
}

/// 解析音频行
/// 格式: Aud audio-32000 | Audio | 57 Segments | ~02m49s
fn parse_audio_line(line: &str) -> Option<AudioStream> {
    let aud_part = if let Some(stripped) = line.strip_prefix("Aud ") {
        stripped
    } else if let Some(pos) = line.find("| Aud ") {
        &line[pos + 6..]
    } else {
        return None;
    };

    let parts: Vec<&str> = aud_part.split('|').map(|s| s.trim()).collect();
    if parts.is_empty() {
        return None;
    }

    // 第一部分是 ID: audio-32000
    let id = parts.first().unwrap_or(&"").to_string();

    // 从 ID 中提取码率 (audio-32000 -> 32000)
    let bandwidth = id
        .split('-')
        .next_back()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);

    Some(AudioStream {
        base: BaseStream {
            id: id.clone(),
            bandwidth,
            codecs: String::new(),
            language: String::new(),
            name: id,
            group_id: None,
            selected: None,
        },
        channels: "2".to_string(),
        sample_rate: 0,
        is_default: false,
    })
}

/// 解析字幕行
fn parse_subtitle_line(line: &str) -> Option<SubtitleStream> {
    let sub_part = if let Some(stripped) = line.strip_prefix("Sub ") {
        stripped
    } else if let Some(pos) = line.find("| Sub ") {
        &line[pos + 6..]
    } else {
        return None;
    };

    let parts: Vec<&str> = sub_part.split('|').map(|s| s.trim()).collect();
    let id = parts.first().unwrap_or(&"").to_string();

    Some(SubtitleStream {
        base: BaseStream {
            id: id.clone(),
            bandwidth: 0,
            codecs: String::new(),
            language: String::new(),
            name: id,
            group_id: None,
            selected: None,
        },
        format: "srt".to_string(),
        is_default: false,
        is_forced: false,
    })
}

/// 从行中解析分片数
fn parse_segments_from_line(line: &str) -> Option<u32> {
    // 查找 "XX Segments" 模式
    let re = regex::Regex::new(r"(\d+)\s*[Ss]egment").ok()?;
    let cap = re.captures(line)?;
    cap.get(1)?.as_str().parse().ok()
}

/// 从行中解析时长 (~02m49s 格式)
fn parse_duration_from_line(line: &str) -> Option<f64> {
    // 查找 ~XXmYYs 格式
    let re = regex::Regex::new(r"~(\d+)m(\d+)s").ok()?;
    let cap = re.captures(line)?;
    let minutes: f64 = cap.get(1)?.as_str().parse().ok()?;
    let seconds: f64 = cap.get(2)?.as_str().parse().ok()?;
    Some(minutes * 60.0 + seconds)
}

/// 获取 N_m3u8DL-RE 版本
#[tauri::command]
pub async fn get_n_m3u8dl_version() -> Result<String, String> {
    // TODO: 实际执行命令获取版本
    Ok("N_m3u8DL-RE v1.0.0".to_string())
}

/// 检测 URL 类型
#[tauri::command(rename_all = "camelCase")]
pub async fn detect_url_type(url: String) -> Result<String, String> {
    let url_type = UrlType::detect(&url);
    Ok(format!("{:?}", url_type).to_lowercase())
}

/// 开始 HTTP 直链视频下载（使用 FFmpeg）
#[tauri::command(rename_all = "camelCase")]
pub async fn start_http_video_download(
    task_id: String,
    url: String,
    save_dir: String,
    save_name: String,
    app: AppHandle,
) -> Result<(), String> {
    log::info!(
        "Starting HTTP video download: task_id={}, url={}",
        task_id,
        url
    );

    let manager = PROCESS_MANAGER.clone();

    // 从配置中获取工具路径
    let tool_paths = get_tool_paths_from_config(&app);

    // 获取 FFmpeg 可执行文件路径
    let ffmpeg = match get_ffmpeg_exe_path(tool_paths.ffmpeg_dir.as_deref()) {
        Some(path) => path.to_string_lossy().to_string(),
        None => {
            return Err(
                "FFmpeg 未找到。请在设置中配置 FFmpeg 目录路径，或使用设置页面的【下载】按钮自动下载。".to_string(),
            );
        }
    };

    // 构建输出路径
    let output_path = PathBuf::from(&save_dir).join(&save_name);

    // FFmpeg 下载命令: ffmpeg -i URL -c copy -progress pipe:2 output.mp4
    // 使用 -progress pipe:2 让 FFmpeg 输出进度信息到 stderr
    let args = vec![
        "-i".to_string(),
        url,
        "-c".to_string(),
        "copy".to_string(), // 直接复制流，不重新编码
        "-progress".to_string(),
        "pipe:2".to_string(), // 输出进度到 stderr
        "-y".to_string(),     // 覆盖已存在文件
        output_path.display().to_string(),
    ];

    log::info!("Running ffmpeg download: {} {:?}", ffmpeg, args);

    // 克隆用于回调的变量
    let task_id_clone = task_id.clone();
    let app_clone = app.clone();
    let output_path_clone = output_path.clone();

    // 用于累积 FFmpeg 进度输出的缓冲区
    // FFmpeg -progress 模式下，每行输出一个 key=value，需要累积后解析
    use std::sync::Mutex;
    let progress_buffer: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    let progress_buffer_clone = progress_buffer.clone();

    // 用于跟踪总时长（从初始输出中提取）
    use std::sync::atomic::{AtomicI64, Ordering};
    let total_duration = Arc::new(AtomicI64::new(0)); // 微秒
    let total_duration_clone = total_duration.clone();

    // 输出回调函数
    let on_output = move |output: String| {
        // 检查是否是 Duration 行（在进度输出之前）
        if output.contains("Duration:") {
            if let Some(duration) = parse_ffmpeg_duration(&output) {
                total_duration_clone.store(duration, Ordering::Relaxed);
                log::debug!("Detected video duration: {} us", duration);
            }
        }

        // 检查是否是进度输出的行（包含 = 但不是普通信息）
        let is_progress_line = output.contains('=')
            && !output.starts_with("Input")
            && !output.starts_with("Output")
            && !output.starts_with("Stream")
            && !output.starts_with("Metadata")
            && !output.starts_with("  ");

        if is_progress_line {
            // 累积到缓冲区
            if let Ok(mut buffer) = progress_buffer_clone.lock() {
                buffer.push_str(&output);
                buffer.push('\n');

                // 检查是否是一个完整的进度块（以 progress= 结尾）
                if output.starts_with("progress=") {
                    // 解析累积的进度数据
                    let progress_data = buffer.clone();
                    buffer.clear();

                    // 解析进度
                    if let Some((current_time_us, total_size)) =
                        parse_ffmpeg_progress_detailed(&progress_data)
                    {
                        let duration = total_duration_clone.load(Ordering::Relaxed);
                        let percent = if duration > 0 {
                            ((current_time_us as f64 / duration as f64) * 100.0).min(100.0) as i32
                        } else {
                            0
                        };

                        // 获取速度
                        let speed = parse_ffmpeg_speed_from_buffer(&progress_data).unwrap_or(0);

                        // 记录进度历史（后端持久化）
                        record_progress(
                            &task_id_clone,
                            percent,
                            speed,
                            total_size.unwrap_or(0) as i64,
                        );

                        let _ = app_clone.emit(
                            &format!("download:progress:{}", task_id_clone),
                            serde_json::json!({
                                "percent": percent,
                                "overallPercent": percent,
                                "speed": speed,
                                "downloadedSize": total_size.unwrap_or(0)
                            }),
                        );
                    }
                }
            }
        } else if !output.is_empty() {
            // 非进度信息，作为日志发送
            let _ = app_clone.emit(
                &format!("download:log:{}", task_id_clone),
                serde_json::json!({ "message": output }),
            );
        }
    };

    // 完成回调函数
    let task_id_clone = task_id.clone();
    let app_clone = app;

    let on_complete = move |success: bool, error_msg: Option<String>| {
        // 刷新进度历史到数据库
        flush_progress(&task_id_clone);

        if success {
            let _ = app_clone.emit(
                &format!("download:complete:{}", task_id_clone),
                serde_json::json!({ "outputPath": output_path_clone.to_str() }),
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
            &ffmpeg,
            args,
            Some(&save_dir),
            on_output,
            on_complete,
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// 解析 FFmpeg 进度输出（详细版本）
///
/// FFmpeg -progress 输出格式（每个 key=value 一行）：
/// frame=123
/// fps=30.00
/// stream_0_0_q=-1.0
/// bitrate=1234.5kbits/s
/// total_size=12345678
/// out_time_us=83450000
/// out_time_ms=83450
/// out_time=00:01:23.450000
/// dup_frames=0
/// drop_frames=0
/// speed=1.00x
/// progress=continue
///
/// 返回 (当前时间微秒, 已下载字节数)
fn parse_ffmpeg_progress_detailed(output: &str) -> Option<(i64, Option<u64>)> {
    let mut out_time_us: Option<i64> = None;
    let mut total_size: Option<u64> = None;

    for line in output.lines() {
        let line = line.trim();
        if line.starts_with("out_time_us=") {
            // 解析 out_time_us（微秒）
            out_time_us = line.strip_prefix("out_time_us=")?.parse().ok();
        } else if line.starts_with("total_size=") {
            // 解析 total_size
            total_size = line.strip_prefix("total_size=")?.parse().ok();
        }
    }

    out_time_us.map(|time| (time, total_size))
}

/// 从累积的进度缓冲区中解析速度
fn parse_ffmpeg_speed_from_buffer(buffer: &str) -> Option<i64> {
    for line in buffer.lines() {
        let line = line.trim();

        // 优先从 bitrate 获取实际速度
        if line.starts_with("bitrate=") {
            // 格式: bitrate=1234.5kbits/s 或 bitrate= 1234.5kbits/s
            let bitrate_str = line.strip_prefix("bitrate=")?;
            let bitrate_str = bitrate_str.trim().trim_end_matches("kbits/s");
            if let Ok(bitrate) = bitrate_str.parse::<f64>() {
                // kbits/s -> bits/s
                return Some((bitrate * 1000.0) as i64);
            }
        }

        // 备用：从 speed 估算
        if line.starts_with("speed=") {
            let speed_str = line.strip_prefix("speed=")?;
            let speed_str = speed_str.trim_end_matches('x').trim();
            if speed_str.parse::<f64>().is_ok() {
                // speed 解析成功，但在 bitrate 之前，继续查找 bitrate
            }
        }
    }

    // 如果没有找到 bitrate，尝试从 speed 估算（假设视频约 1.5Mbps）
    for line in buffer.lines() {
        let line = line.trim();
        if line.starts_with("speed=") {
            let speed_str = line.strip_prefix("speed=")?;
            let speed_str = speed_str.trim_end_matches('x').trim();
            if let Ok(speed) = speed_str.parse::<f64>() {
                // 估算：speed * 1.5Mbps
                return Some((speed * 1_500_000.0) as i64);
            }
        }
    }

    None
}

/// 解析 FFmpeg 输出中的视频总时长
///
/// FFmpeg 在开始时会输出类似这样的信息：
///   Duration: 00:05:30.00, start: 0.000000, bitrate: 1234 kb/s
fn parse_ffmpeg_duration(output: &str) -> Option<i64> {
    // 查找 Duration: HH:MM:SS.ms 格式
    let duration_re = regex::Regex::new(r"Duration:\s*(\d{2}):(\d{2}):(\d{2})\.(\d{2})").ok()?;

    for line in output.lines() {
        if let Some(caps) = duration_re.captures(line) {
            let hours: i64 = caps.get(1)?.as_str().parse().ok()?;
            let minutes: i64 = caps.get(2)?.as_str().parse().ok()?;
            let seconds: i64 = caps.get(3)?.as_str().parse().ok()?;
            let centiseconds: i64 = caps.get(4)?.as_str().parse().ok()?;

            // 转换为微秒
            let total_us =
                (hours * 3600 + minutes * 60 + seconds) * 1_000_000 + centiseconds * 10_000;
            return Some(total_us);
        }
    }
    None
}

/// 根据混流格式查找可能的文件扩展名
fn get_possible_extensions(mux_format: Option<&str>) -> Vec<&'static str> {
    match mux_format {
        Some("mp4") => vec!["mp4"],
        Some("mkv") => vec!["mkv"],
        _ => vec!["mp4", "mkv", "ts", "m4a", "m4v", "webm"],
    }
}

/// 查找实际生成的输出文件
///
/// N_m3u8DL-RE 可能会生成与 save_name 略有不同的文件名（例如添加扩展名、修改冲突文件名等）
/// 此函数尝试在输出目录中找到实际生成的媒体文件
fn find_output_file(
    save_dir: &str,
    save_name: Option<&str>,
    mux_format: Option<&str>,
) -> Option<String> {
    let dir = PathBuf::from(save_dir);
    if !dir.exists() {
        log::warn!("Save directory does not exist: {}", save_dir);
        return None;
    }

    let extensions = get_possible_extensions(mux_format);

    // 如果有指定的文件名，尝试精确匹配或带扩展名的匹配
    if let Some(name) = save_name {
        // 1. 尝试精确匹配（用户可能已经指定了扩展名）
        let exact_path = dir.join(name);
        if exact_path.exists() && is_media_file(&exact_path) {
            return exact_path.to_str().map(|s| s.to_string());
        }

        // 2. 尝试添加各种扩展名
        for ext in &extensions {
            let path_with_ext = dir.join(format!("{}.{}", name, ext));
            if path_with_ext.exists() {
                return path_with_ext.to_str().map(|s| s.to_string());
            }
        }

        // 3. 尝试匹配以 save_name 开头的文件（N_m3u8DL-RE 可能添加后缀避免冲突）
        if let Ok(entries) = std::fs::read_dir(&dir) {
            let mut candidates: Vec<_> = entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    let file_name = e.file_name();
                    let file_name_str = file_name.to_string_lossy();
                    // 文件名以 save_name 开头，且是媒体文件
                    file_name_str.starts_with(name) && is_media_file(&e.path())
                })
                .collect();

            // 按修改时间排序，取最新的
            candidates.sort_by(|a, b| {
                let time_a = a.metadata().ok().and_then(|m| m.modified().ok());
                let time_b = b.metadata().ok().and_then(|m| m.modified().ok());
                time_b.cmp(&time_a) // 降序，最新的在前
            });

            if let Some(latest) = candidates.first() {
                return latest.path().to_str().map(|s| s.to_string());
            }
        }
    }

    // 4. 如果没有指定文件名，尝试在目录中找到最新的媒体文件
    if let Ok(entries) = std::fs::read_dir(&dir) {
        let mut candidates: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| is_media_file(&e.path()))
            .collect();

        candidates.sort_by(|a, b| {
            let time_a = a.metadata().ok().and_then(|m| m.modified().ok());
            let time_b = b.metadata().ok().and_then(|m| m.modified().ok());
            time_b.cmp(&time_a)
        });

        if let Some(latest) = candidates.first() {
            return latest.path().to_str().map(|s| s.to_string());
        }
    }

    log::warn!("Could not find output file in directory: {}", save_dir);
    // 返回预期的路径（即使文件不存在，至少路径是正确的格式）
    save_name.map(|name| {
        let ext = extensions.first().unwrap_or(&"mp4");
        dir.join(format!("{}.{}", name, ext))
            .to_string_lossy()
            .to_string()
    })
}

/// 检查文件是否是媒体文件
fn is_media_file(path: &std::path::Path) -> bool {
    let media_extensions = [
        "mp4", "mkv", "ts", "m4a", "m4v", "webm", "avi", "mov", "flv", "wmv", "m2ts", "vob", "mp3",
        "aac", "ogg", "flac", "wav", "srt", "vtt", "ass",
    ];

    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| media_extensions.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

/// 获取文件信息
#[tauri::command]
pub async fn get_file_info(path: String) -> Result<FileInfo, String> {
    let path = PathBuf::from(&path);

    if !path.exists() {
        return Err("文件不存在".to_string());
    }

    let metadata = path
        .metadata()
        .map_err(|e| format!("获取文件信息失败: {}", e))?;

    let size = metadata.len();
    let modified = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as i64);

    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();

    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_string();

    Ok(FileInfo {
        path: path.to_string_lossy().to_string(),
        file_name,
        extension,
        size,
        modified,
        exists: true,
    })
}

/// 文件信息结构
#[derive(Debug, serde::Serialize)]
pub struct FileInfo {
    /// 文件完整路径
    pub path: String,
    /// 文件名
    pub file_name: String,
    /// 文件扩展名
    pub extension: String,
    /// 文件大小（字节）
    pub size: u64,
    /// 修改时间（Unix 毫秒时间戳）
    pub modified: Option<i64>,
    /// 文件是否存在
    pub exists: bool,
}

/// 媒体文件分析结果
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaFileInfo {
    /// 分辨率 (如 "1920x1080")
    pub resolution: Option<String>,
    /// 视频宽度
    pub width: Option<u32>,
    /// 视频高度
    pub height: Option<u32>,
    /// 帧率
    pub frame_rate: Option<f64>,
    /// 视频编码
    pub video_codec: Option<String>,
    /// 视频范围 (SDR/HDR/DV)
    pub video_range: Option<String>,
    /// 音频编码
    pub audio_codec: Option<String>,
    /// 音频声道数
    pub audio_channels: Option<String>,
    /// 音频语言
    pub audio_language: Option<String>,
    /// 总时长（秒）
    pub duration: Option<f64>,
    /// 文件大小（字节）
    pub file_size: Option<u64>,
    /// 比特率 (bps)
    pub bit_rate: Option<u64>,
    /// 文件格式
    pub format: Option<String>,
}

/// 分析媒体文件（使用 ffprobe）
#[tauri::command(rename_all = "camelCase")]
pub async fn analyze_media_file(
    file_path: String,
    app: AppHandle,
) -> Result<MediaFileInfo, String> {
    log::info!("Analyzing media file: {}", file_path);

    let path = PathBuf::from(&file_path);
    if !path.exists() {
        return Err(format!("文件不存在: {}", file_path));
    }

    // 从配置中获取工具路径
    let tool_paths = get_tool_paths_from_config(&app);

    // 获取 ffprobe 可执行文件路径
    let ffprobe = match get_ffprobe_exe_path(tool_paths.ffmpeg_dir.as_deref()) {
        Some(path) => path.to_string_lossy().to_string(),
        None => {
            return Err(
                "FFprobe 未找到。请在设置中配置 FFmpeg 目录路径，或使用设置页面的【下载】按钮自动下载。".to_string(),
            );
        }
    };

    log::info!("Analyzing media file with ffprobe: {}", ffprobe);

    // 执行 ffprobe（Windows 平台隐藏窗口）
    #[cfg(target_os = "windows")]
    let output = Command::new(&ffprobe)
        .args([
            "-v",
            "quiet",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
            &file_path,
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("执行 ffprobe 失败: {}。请确保 FFmpeg 已安装。", e))?;

    #[cfg(not(target_os = "windows"))]
    let output = Command::new(&ffprobe)
        .args([
            "-v",
            "quiet",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
            &file_path,
        ])
        .output()
        .map_err(|e| format!("执行 ffprobe 失败: {}。请确保 FFmpeg 已安装。", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("分析媒体文件失败: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_media_file_info(&stdout)
}

/// 解析 ffprobe 输出为 MediaFileInfo
fn parse_media_file_info(json: &str) -> Result<MediaFileInfo, String> {
    let parsed: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("解析 ffprobe 输出失败: {}", e))?;

    let mut info = MediaFileInfo {
        resolution: None,
        width: None,
        height: None,
        frame_rate: None,
        video_codec: None,
        video_range: None,
        audio_codec: None,
        audio_channels: None,
        audio_language: None,
        duration: None,
        file_size: None,
        bit_rate: None,
        format: None,
    };

    // 解析 format 信息
    if let Some(format) = parsed.get("format") {
        // 时长
        if let Some(dur) = format.get("duration").and_then(|d| d.as_str()) {
            info.duration = dur.parse().ok();
        } else if let Some(dur) = format.get("duration").and_then(|d| d.as_f64()) {
            info.duration = Some(dur);
        }

        // 文件大小
        if let Some(size) = format.get("size").and_then(|s| s.as_str()) {
            info.file_size = size.parse().ok();
        } else if let Some(size) = format.get("size").and_then(|s| s.as_u64()) {
            info.file_size = Some(size);
        }

        // 比特率
        if let Some(br) = format.get("bit_rate").and_then(|b| b.as_str()) {
            info.bit_rate = br.parse().ok();
        } else if let Some(br) = format.get("bit_rate").and_then(|b| b.as_u64()) {
            info.bit_rate = Some(br);
        }

        // 格式
        info.format = format
            .get("format_long_name")
            .and_then(|n| n.as_str())
            .map(|s| s.to_string());
    }

    // 解析流信息
    if let Some(streams) = parsed.get("streams").and_then(|s| s.as_array()) {
        for stream in streams {
            let codec_type = stream
                .get("codec_type")
                .and_then(|t| t.as_str())
                .unwrap_or("");

            if codec_type == "video" && info.video_codec.is_none() {
                // 只取第一个视频流
                let width = stream.get("width").and_then(|w| w.as_u64()).unwrap_or(0) as u32;
                let height = stream.get("height").and_then(|h| h.as_u64()).unwrap_or(0) as u32;

                info.width = Some(width);
                info.height = Some(height);
                info.resolution = Some(format!("{}x{}", width, height));

                info.video_codec = stream
                    .get("codec_name")
                    .and_then(|c| c.as_str())
                    .map(|s| s.to_string());

                // 帧率 (可能是 "30/1" 或 "29.97" 格式)
                info.frame_rate = stream
                    .get("r_frame_rate")
                    .and_then(|r| r.as_str())
                    .and_then(|r| {
                        // 处理 "30000/1001" 格式
                        if r.contains('/') {
                            let parts: Vec<&str> = r.split('/').collect();
                            if parts.len() == 2 {
                                let num: f64 = parts[0].parse().ok()?;
                                let den: f64 = parts[1].parse().ok()?;
                                return Some(num / den);
                            }
                        }
                        r.parse().ok()
                    });

                // 视频范围 (HDR/SDR)
                // 从 side_data_list 或 color_transfer 检测
                let transfer = stream
                    .get("color_transfer")
                    .and_then(|t| t.as_str())
                    .unwrap_or("");
                let is_hdr = stream.get("side_data_list").is_some()
                    || transfer.contains("smpte2084")
                    || transfer.contains("arib-std-b67");
                info.video_range = Some(if is_hdr {
                    "HDR".to_string()
                } else {
                    "SDR".to_string()
                });
            } else if codec_type == "audio" && info.audio_codec.is_none() {
                // 只取第一个音频流
                info.audio_codec = stream
                    .get("codec_name")
                    .and_then(|c| c.as_str())
                    .map(|s| s.to_string());

                info.audio_channels = stream
                    .get("channels")
                    .and_then(|c| c.as_u64())
                    .map(|c| c.to_string());

                info.audio_language = stream
                    .get("tags")
                    .and_then(|t| t.get("language"))
                    .and_then(|l| l.as_str())
                    .map(|s| s.to_string());
            }
        }
    }

    Ok(info)
}
