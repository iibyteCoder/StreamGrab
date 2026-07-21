//! 流信息类型定义
//!
//! `parse_url` 解析结果的领域模型，与前端 `src/domain/stream.ts` 对应

use serde::{Deserialize, Serialize};

// 分辨率解析的唯一实现位于 domain/media（消灭重复）
pub use crate::domain::media::parse_resolution;

/// 流信息
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct StreamInfo {
    pub videos: Vec<VideoStream>,
    pub audios: Vec<AudioStream>,
    pub subtitles: Vec<SubtitleStream>,
    /// 总时长（秒）
    pub duration: f64,
    /// 分片数
    pub segment_count: u32,
    /// 是否直播
    pub is_live: bool,
    /// 是否加密
    pub is_encrypted: bool,
}

/// 基础流属性（被各流类型 flatten 复用）
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
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
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
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
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AudioStream {
    #[serde(flatten)]
    pub base: BaseStream,
    pub channels: String,
    pub sample_rate: u32,
    pub is_default: bool,
}

/// 字幕流
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SubtitleStream {
    #[serde(flatten)]
    pub base: BaseStream,
    pub format: String,
    pub is_default: bool,
    pub is_forced: bool,
}
