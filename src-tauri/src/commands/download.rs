//! 下载相关命令
//!
//! 封装 N_m3u8DL-RE 和 FFmpeg 进程的启动、停止、暂停、恢复等操作

use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

use crate::process::manager::ProcessManager;
use crate::process::parser::OutputParser;
use crate::types::{
    parse_resolution, AudioStream, BaseStream, StreamInfo, SubtitleStream, UrlType, VideoStream,
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
    program_path: Option<String>,
    app: AppHandle,
) -> Result<(), String> {
    log::info!("Starting download: task_id={}, url={}", task_id, url);

    let manager = PROCESS_MANAGER.clone();

    // 获取 N_m3u8DL-RE 程序路径（必须在设置中配置绝对路径）
    let program_path = match program_path {
        Some(path) if !path.is_empty() => {
            log::info!("Using N_m3u8DL-RE path: {}", path);
            path
        }
        _ => {
            return Err(
                "N_m3u8DL-RE 路径未配置，请在设置中配置 N_m3u8DL-RE 的绝对路径".to_string(),
            );
        }
    };

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
    // 从 args 中提取混流格式
    let mux_format = args
        .iter()
        .position(|a| a.starts_with("-M") || a == "-M")
        .and_then(|idx| args.get(idx + 1))
        .and_then(|s| {
            // 解析 format=xxx
            s.split(':')
                .find_map(|part| part.strip_prefix("format=").map(|f| f.to_lowercase()))
        });

    let on_complete = move |success: bool, error_msg: Option<String>| {
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
        .start_process(task_id, &program_path, args, on_output, on_complete)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// 停止下载命令
#[tauri::command]
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

/// 解析 URL 获取流信息（接收完整参数数组）
/// 前端使用 buildParseArgs 构建参数，复用所有应用设置
#[tauri::command(rename_all = "camelCase")]
pub async fn parse_url(
    args: Vec<String>,
    program_path: Option<String>,
    ffmpeg_path: Option<String>,
    _app: AppHandle,
) -> Result<StreamInfo, String> {
    // 从参数中提取 URL（第一个参数）
    let url = args.first().cloned().unwrap_or_default();
    log::info!("Parsing URL: {}", url);

    // 检测 URL 类型
    let url_type = UrlType::detect(&url);
    log::info!("Detected URL type: {:?}", url_type);

    // 如果是 HTTP 直链视频，使用 ffmpeg 获取信息
    if url_type.needs_ffmpeg() {
        return parse_http_video_url(&url, ffmpeg_path).await;
    }

    // 如果不是流媒体格式，返回错误
    if !url_type.is_streaming() {
        return Err("不支持的 URL 格式。请输入 M3U8、DASH 或 MSS 流媒体链接。".to_string());
    }

    // 获取 N_m3u8DL-RE 程序路径
    let program = match program_path {
        Some(path) if !path.is_empty() => path,
        _ => {
            return Err(
                "N_m3u8DL-RE 路径未配置，请在设置中配置 N_m3u8DL-RE 的绝对路径".to_string(),
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

    // 执行命令
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
async fn parse_http_video_url(
    url: &str,
    ffmpeg_path: Option<String>,
) -> Result<StreamInfo, String> {
    let ffmpeg = match ffmpeg_path {
        Some(path) if !path.is_empty() => path,
        _ => "ffmpeg".to_string(), // 尝试使用系统 PATH 中的 ffmpeg
    };

    log::info!("Parsing HTTP video URL with ffmpeg: {}", ffmpeg);

    // 使用 ffprobe 获取视频信息
    let ffprobe = if ffmpeg.ends_with("ffmpeg.exe") || ffmpeg.ends_with("ffmpeg") {
        ffmpeg.replace("ffmpeg", "ffprobe")
    } else {
        format!("{}probe", ffmpeg.trim_end_matches(".exe"))
    };

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
    ffmpeg_path: Option<String>,
    app: AppHandle,
) -> Result<(), String> {
    log::info!(
        "Starting HTTP video download: task_id={}, url={}",
        task_id,
        url
    );

    let manager = PROCESS_MANAGER.clone();

    // 获取 FFmpeg 路径
    let ffmpeg = match ffmpeg_path {
        Some(path) if !path.is_empty() => path,
        _ => "ffmpeg".to_string(),
    };

    // 构建输出路径
    let output_path = PathBuf::from(&save_dir).join(&save_name);

    // FFmpeg 下载命令: ffmpeg -i URL -c copy output.mp4
    let args = vec![
        "-i".to_string(),
        url,
        "-c".to_string(),
        "copy".to_string(), // 直接复制流，不重新编码
        "-y".to_string(),   // 覆盖已存在文件
        output_path.display().to_string(),
    ];

    log::info!("Running ffmpeg download: {} {:?}", ffmpeg, args);

    // 克隆用于回调的变量
    let task_id_clone = task_id.clone();
    let app_clone = app.clone();
    let output_path_clone = output_path.clone();

    // 输出回调函数
    let on_output = move |output: String| {
        // 解析 ffmpeg 输出发送进度
        if let Some(progress) = parse_ffmpeg_progress(&output) {
            let _ = app_clone.emit(
                &format!("download:progress:{}", task_id_clone),
                serde_json::json!({ "percent": progress }),
            );
        }
        // 发送日志
        let _ = app_clone.emit(
            &format!("download:log:{}", task_id_clone),
            serde_json::json!({ "message": output }),
        );
    };

    // 完成回调函数
    let task_id_clone = task_id.clone();
    let app_clone = app;

    let on_complete = move |success: bool, error_msg: Option<String>| {
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
        .start_process(task_id, &ffmpeg, args, on_output, on_complete)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// 解析 FFmpeg 进度输出
fn parse_ffmpeg_progress(output: &str) -> Option<f64> {
    // FFmpeg 输出格式: frame=  123 fps= 30 q=-1.0 size=   12345kB time=00:01:23.45 bitrate=1234.5kbits/s
    // 提取 time 并计算进度（需要知道总时长，这里简化处理）

    // 简单返回 None，进度计算需要更多上下文
    // 实际可以通过解析 time= 字段来计算
    let _ = output;
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
