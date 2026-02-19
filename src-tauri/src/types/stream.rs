//! 流信息类型定义
//!
//! 用于解析 URL 返回的流信息

use serde::{Deserialize, Serialize};

/// 流信息 - 与前端类型匹配
#[derive(Debug, Serialize)]
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
pub struct AudioStream {
    #[serde(flatten)]
    pub base: BaseStream,
    pub channels: String,
    pub sample_rate: u32,
    pub is_default: bool,
}

/// 字幕流
#[derive(Debug, Serialize)]
pub struct SubtitleStream {
    #[serde(flatten)]
    pub base: BaseStream,
    pub format: String,
    pub is_default: bool,
    pub is_forced: bool,
}

/// 请求头项
#[derive(Debug, Deserialize)]
pub struct HeaderItem {
    pub key: String,
    pub value: String,
    #[serde(default)]
    pub enabled: bool,
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
