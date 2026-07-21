//! ffprobe 执行与输出解析
//!
//! 媒体探测的唯一入口：统一执行参数 + 两种 JSON 解析
//! （[`StreamInfo`] 供 URL 解析、[`MediaInfo`] 供文件分析）。

use std::process::Command;

use crate::domain::download::{AudioStream, BaseStream, StreamInfo, VideoStream};
use crate::domain::media::{MediaAnalyzer, MediaInfo};
use crate::shared::{AppError, AppResult};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// ffprobe 标准参数：安静模式 + JSON 输出 + format/streams 信息
pub fn probe_args(target: &str) -> Vec<String> {
    [
        "-v",
        "quiet",
        "-print_format",
        "json",
        "-show_format",
        "-show_streams",
        target,
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

/// 执行 ffprobe，返回 JSON 输出
pub fn run_ffprobe(ffprobe_bin: &str, target: &str) -> AppResult<String> {
    let mut cmd = Command::new(ffprobe_bin);
    cmd.args(probe_args(target));
    #[cfg(target_os = "windows")]
    cmd.creation_flags(crate::infrastructure::platform::CREATE_NO_WINDOW);

    let output = cmd.output().map_err(|e| {
        AppError::tool_not_found(format!(
            "执行 ffprobe 失败: {e}。请确保 FFmpeg 已安装并配置正确。"
        ))
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::process(format!(
            "媒体探测失败: {}",
            stderr.trim()
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// `MediaAnalyzer` 的 ffprobe 实现
pub struct FfprobeAnalyzer {
    bin: String,
}

impl FfprobeAnalyzer {
    pub fn new(bin: impl Into<String>) -> Self {
        Self { bin: bin.into() }
    }
}

impl MediaAnalyzer for FfprobeAnalyzer {
    fn analyze(&self, file_path: &str) -> AppResult<MediaInfo> {
        let json = run_ffprobe(&self.bin, file_path)?;
        media_info_from_json(&json)
    }
}

/// 解析 ffprobe JSON 为 [`StreamInfo`]（`parse_url` 直链分支使用）
pub fn stream_info_from_json(json: &str) -> AppResult<StreamInfo> {
    let parsed: serde_json::Value = serde_json::from_str(json)?;

    let mut videos: Vec<VideoStream> = vec![];
    let mut audios: Vec<AudioStream> = vec![];
    let mut duration = 0.0;

    if let Some(format) = parsed.get("format") {
        duration = format
            .get("duration")
            .and_then(|d| {
                d.as_str()
                    .and_then(|s| s.parse().ok())
                    .or_else(|| d.as_f64())
            })
            .unwrap_or(0.0);
    }

    if let Some(streams) = parsed.get("streams").and_then(|s| s.as_array()) {
        for stream in streams {
            let codec_type = stream
                .get("codec_type")
                .and_then(|t| t.as_str())
                .unwrap_or("");

            if codec_type == "video" {
                let width = stream.get("width").and_then(|w| w.as_u64()).unwrap_or(0) as u32;
                let height = stream.get("height").and_then(|h| h.as_u64()).unwrap_or(0) as u32;
                let codec = str_field(stream, "codec_name");
                let bitrate = parse_field(stream, "bit_rate").unwrap_or(0);
                let resolution = format!("{width}x{height}");

                videos.push(VideoStream {
                    base: BaseStream {
                        id: format!("video_{resolution}"),
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
                    frame_rate: parse_frame_rate(stream) as f32,
                    video_range: detect_video_range(stream),
                });
            } else if codec_type == "audio" {
                let codec = str_field(stream, "codec_name");
                let channels = stream
                    .get("channels")
                    .and_then(|c| c.as_u64())
                    .unwrap_or(2)
                    .to_string();
                let sample_rate = parse_field(stream, "sample_rate").unwrap_or(0);
                let bitrate = parse_field(stream, "bit_rate").unwrap_or(0);
                let language = stream
                    .get("tags")
                    .and_then(|t| t.get("language"))
                    .and_then(|l| l.as_str())
                    .unwrap_or("")
                    .to_string();

                audios.push(AudioStream {
                    base: BaseStream {
                        id: "audio".to_string(),
                        bandwidth: bitrate,
                        codecs: codec,
                        language,
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
        return Err(AppError::parse("未找到有效的视频或音频流"));
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

/// 解析 ffprobe JSON 为统一 [`MediaInfo`]（文件分析使用）
pub fn media_info_from_json(json: &str) -> AppResult<MediaInfo> {
    let parsed: serde_json::Value = serde_json::from_str(json)?;
    let mut info = MediaInfo::default();

    // format 段
    if let Some(format) = parsed.get("format") {
        info.duration = format.get("duration").and_then(|d| {
            d.as_str()
                .and_then(|s| s.parse().ok())
                .or_else(|| d.as_f64())
        });
        info.file_size = format.get("size").and_then(|s| {
            s.as_str()
                .and_then(|v| v.parse().ok())
                .or_else(|| s.as_i64())
        });
        info.bit_rate = format.get("bit_rate").and_then(|b| {
            b.as_str()
                .and_then(|v| v.parse().ok())
                .or_else(|| b.as_i64())
        });
        info.file_format = format
            .get("format_long_name")
            .and_then(|n| n.as_str())
            .map(String::from);
    }

    // streams 段（只取第一个视频流和第一个音频流）
    if let Some(streams) = parsed.get("streams").and_then(|s| s.as_array()) {
        for stream in streams {
            let codec_type = stream
                .get("codec_type")
                .and_then(|t| t.as_str())
                .unwrap_or("");

            if codec_type == "video" && info.video_codec.is_none() {
                let width = stream.get("width").and_then(|w| w.as_u64()).unwrap_or(0) as u32;
                let height = stream.get("height").and_then(|h| h.as_u64()).unwrap_or(0) as u32;
                info.width = Some(width);
                info.height = Some(height);
                info.resolution = Some(format!("{width}x{height}"));
                info.video_codec = stream
                    .get("codec_name")
                    .and_then(|c| c.as_str())
                    .map(String::from);
                info.frame_rate = Some(parse_frame_rate(stream)).filter(|f| *f > 0.0);
                info.video_range = Some(detect_video_range(stream));
            } else if codec_type == "audio" && info.audio_codec.is_none() {
                info.audio_codec = stream
                    .get("codec_name")
                    .and_then(|c| c.as_str())
                    .map(String::from);
                info.audio_channels = stream
                    .get("channels")
                    .and_then(|c| c.as_u64())
                    .map(|c| c.to_string());
                info.audio_language = stream
                    .get("tags")
                    .and_then(|t| t.get("language"))
                    .and_then(|l| l.as_str())
                    .map(String::from);
            }
        }
    }

    Ok(info)
}

/// 字符串字段
fn str_field(stream: &serde_json::Value, key: &str) -> String {
    stream
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

/// 数值字段（字符串或数字形式）
fn parse_field<T: std::str::FromStr>(stream: &serde_json::Value, key: &str) -> Option<T> {
    stream.get(key).and_then(|v| {
        v.as_str().and_then(|s| s.parse().ok()).or_else(|| {
            v.as_u64()
                .and_then(|u| u.to_string().parse().ok())
                .or_else(|| v.as_i64().and_then(|i| i.to_string().parse().ok()))
        })
    })
}

/// 帧率（支持 `30000/1001` 分数格式）
fn parse_frame_rate(stream: &serde_json::Value) -> f64 {
    stream
        .get("r_frame_rate")
        .and_then(|r| r.as_str())
        .and_then(|r| {
            if let Some((num, den)) = r.split_once('/') {
                let num: f64 = num.parse().ok()?;
                let den: f64 = den.parse().ok()?;
                (den != 0.0).then_some(num / den)
            } else {
                r.parse().ok()
            }
        })
        .unwrap_or(0.0)
}

/// HDR/SDR 检测（color_transfer 或 side_data）
fn detect_video_range(stream: &serde_json::Value) -> String {
    let transfer = stream
        .get("color_transfer")
        .and_then(|t| t.as_str())
        .unwrap_or("");
    let is_hdr = stream.get("side_data_list").is_some()
        || transfer.contains("smpte2084")
        || transfer.contains("arib-std-b67");
    if is_hdr {
        "HDR".to_string()
    } else {
        "SDR".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = r#"{
        "streams": [
            {
                "codec_type": "video",
                "codec_name": "hevc",
                "width": 3840,
                "height": 2160,
                "r_frame_rate": "24000/1001",
                "bit_rate": "15000000",
                "color_transfer": "smpte2084"
            },
            {
                "codec_type": "audio",
                "codec_name": "aac",
                "channels": 2,
                "sample_rate": "48000",
                "bit_rate": "128000",
                "tags": { "language": "jpn" }
            }
        ],
        "format": {
            "duration": "149.50",
            "size": "283115520",
            "bit_rate": "15160000",
            "format_long_name": "QuickTime / MOV"
        }
    }"#;

    #[test]
    fn parses_stream_info() {
        let info = stream_info_from_json(FIXTURE).unwrap();
        assert_eq!(info.videos.len(), 1);
        assert_eq!(info.videos[0].resolution, "3840x2160");
        assert_eq!(info.videos[0].base.codecs, "hevc");
        assert_eq!(info.videos[0].video_range, "HDR");
        assert!((info.videos[0].frame_rate - 23.976).abs() < 0.01);
        assert_eq!(info.audios.len(), 1);
        assert_eq!(info.audios[0].base.language, "jpn");
        assert_eq!(info.duration, 149.5);
    }

    #[test]
    fn parses_media_info() {
        let info = media_info_from_json(FIXTURE).unwrap();
        assert_eq!(info.resolution.as_deref(), Some("3840x2160"));
        assert_eq!(info.video_codec.as_deref(), Some("hevc"));
        assert_eq!(info.video_range.as_deref(), Some("HDR"));
        assert_eq!(info.audio_codec.as_deref(), Some("aac"));
        assert_eq!(info.audio_channels.as_deref(), Some("2"));
        assert_eq!(info.audio_language.as_deref(), Some("jpn"));
        assert_eq!(info.duration, Some(149.5));
        assert_eq!(info.file_size, Some(283115520));
        assert_eq!(info.bit_rate, Some(15160000));
        assert_eq!(info.file_format.as_deref(), Some("QuickTime / MOV"));
    }

    #[test]
    fn stream_info_errors_on_no_streams() {
        let json = r#"{"streams": [], "format": {"duration": "1.0"}}"#;
        assert!(stream_info_from_json(json).is_err());
    }

    #[test]
    fn invalid_json_is_parse_error() {
        let err = stream_info_from_json("{broken").unwrap_err();
        assert!(matches!(err, crate::shared::AppError::Serialization(_)));
    }

    #[test]
    fn sdr_detection_default() {
        let json = r#"{
            "streams": [{"codec_type": "video", "codec_name": "h264", "width": 1920, "height": 1080, "r_frame_rate": "30/1"}],
            "format": {}
        }"#;
        let info = stream_info_from_json(json).unwrap();
        assert_eq!(info.videos[0].video_range, "SDR");
        assert_eq!(info.videos[0].frame_rate, 30.0);
    }
}
