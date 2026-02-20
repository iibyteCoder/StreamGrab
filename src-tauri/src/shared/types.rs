//! 流信息类型定义
//!
//! 用于解析 URL 返回的流信息

use serde::{Deserialize, Serialize};

/// URL 类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UrlType {
    /// HLS 流媒体 (m3u8)
    Hls,
    /// DASH 流媒体 (mpd)
    Dash,
    /// MSS 流媒体 (ism)
    Mss,
    /// HTTP 直链视频 (mp4, mkv, etc.)
    HttpVideo,
    /// 未知/不支持
    Unknown,
}

impl UrlType {
    /// 检测 URL 类型
    pub fn detect(url: &str) -> Self {
        let url_lower = url.to_lowercase();

        // 检查扩展名
        if url_lower.ends_with(".m3u8") || url_lower.contains(".m3u8?") {
            return UrlType::Hls;
        }
        if url_lower.ends_with(".mpd") || url_lower.contains(".mpd?") {
            return UrlType::Dash;
        }
        if url_lower.ends_with(".ism/manifest")
            || url_lower.contains(".ism/manifest?")
            || url_lower.ends_with(".isml/manifest")
        {
            return UrlType::Mss;
        }

        // 检查常见视频扩展名
        let video_extensions = [
            ".mp4", ".mkv", ".avi", ".mov", ".wmv", ".flv", ".webm", ".m4v", ".ts", ".m2ts",
            ".mp3", ".m4a", ".aac", ".ogg", ".flac", ".wav",
        ];

        for ext in &video_extensions {
            // 检查是否以该扩展名结尾，或者扩展名后跟查询参数
            if url_lower.ends_with(ext) || url_lower.contains(&format!("{}?", ext)) {
                return UrlType::HttpVideo;
            }
        }

        UrlType::Unknown
    }

    /// 是否需要使用 ffmpeg 下载
    pub fn needs_ffmpeg(&self) -> bool {
        matches!(self, UrlType::HttpVideo)
    }

    /// 是否是流媒体格式（N_m3u8DL-RE 支持）
    pub fn is_streaming(&self) -> bool {
        matches!(self, UrlType::Hls | UrlType::Dash | UrlType::Mss)
    }
}

/// 流信息 - 与前端类型匹配
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
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
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
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
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
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
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioStream {
    #[serde(flatten)]
    pub base: BaseStream,
    pub channels: String,
    pub sample_rate: u32,
    pub is_default: bool,
}

/// 字幕流
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleStream {
    #[serde(flatten)]
    pub base: BaseStream,
    pub format: String,
    pub is_default: bool,
    pub is_forced: bool,
}

/// 解析分辨率字符串，例如 "1920x1080" -> (1920, 1080)
pub fn parse_resolution(resolution: &str) -> (u32, u32) {
    let parts: Vec<&str> = resolution.split('x').collect();
    if parts.len() == 2 {
        let width = parts[0].parse().unwrap_or(0);
        let height = parts[1].parse().unwrap_or(0);
        (width, height)
    } else {
        (0, 0)
    }
}
