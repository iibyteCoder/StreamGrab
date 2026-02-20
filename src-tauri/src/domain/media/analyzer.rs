//! 媒体分析器
//!
//! 负责分析媒体文件信息

use serde::Serialize;

/// 媒体文件分析结果
#[derive(Debug, Clone, Serialize)]
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

/// 媒体分析器 Trait
///
/// 由基础设施层实现，封装 ffprobe 调用
pub trait MediaAnalyzer: Send + Sync {
    /// 分析媒体文件
    fn analyze(&self, file_path: &str) -> Result<MediaFileInfo, String>;
}
