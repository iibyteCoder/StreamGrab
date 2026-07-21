//! 媒体信息模型
//!
//! 统一的媒体元数据类型，供以下场景共用：
//! - 任务元数据持久化（`tasks.media_info_json` 列）
//! - 流解析结果提取（`parse_url` 后写入任务）
//! - 本地文件分析（`analyze_media_file` 命令）

use serde::{Deserialize, Serialize};

/// 媒体信息
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MediaInfo {
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
    /// 分片数（流媒体）
    pub segment_count: Option<u32>,
    /// 是否是直播
    pub is_live: bool,
    /// 是否加密
    pub is_encrypted: bool,
    /// 文件格式 (mp4/mkv/ts...)
    pub file_format: Option<String>,
    /// 文件大小（字节，本地文件分析时填充）
    pub file_size: Option<i64>,
    /// 比特率 (bps)
    pub bit_rate: Option<i64>,
}

impl MediaInfo {
    /// 合并两个信息源（`other` 中的 Some/非默认字段覆盖 self）
    ///
    /// 用于「流解析元数据 + 下载完成后文件分析结果」的增量补全。
    pub fn merge(&mut self, other: &MediaInfo) {
        macro_rules! merge_field {
            ($field:ident) => {
                if other.$field.is_some() {
                    self.$field = other.$field.clone();
                }
            };
        }
        merge_field!(resolution);
        merge_field!(width);
        merge_field!(height);
        merge_field!(frame_rate);
        merge_field!(video_codec);
        merge_field!(video_range);
        merge_field!(audio_codec);
        merge_field!(audio_channels);
        merge_field!(audio_language);
        merge_field!(duration);
        merge_field!(segment_count);
        merge_field!(file_format);
        merge_field!(file_size);
        merge_field!(bit_rate);
        self.is_live = self.is_live || other.is_live;
        self.is_encrypted = self.is_encrypted || other.is_encrypted;
    }
}

/// 解析分辨率字符串，例如 "1920x1080" -> (1920, 1080)
pub fn parse_resolution(resolution: &str) -> (u32, u32) {
    let mut parts = resolution.split('x');
    match (parts.next(), parts.next()) {
        (Some(w), Some(h)) => (w.parse().unwrap_or(0), h.parse().unwrap_or(0)),
        _ => (0, 0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_resolution_variants() {
        assert_eq!(parse_resolution("1920x1080"), (1920, 1080));
        assert_eq!(parse_resolution("3840x2160"), (3840, 2160));
        assert_eq!(parse_resolution("invalid"), (0, 0));
        assert_eq!(parse_resolution(""), (0, 0));
    }

    #[test]
    fn merge_prefers_other_when_present() {
        let mut base = MediaInfo {
            resolution: Some("1920x1080".into()),
            duration: Some(100.0),
            ..Default::default()
        };
        let update = MediaInfo {
            file_size: Some(12345),
            duration: Some(101.5),
            ..Default::default()
        };
        base.merge(&update);
        assert_eq!(base.resolution.as_deref(), Some("1920x1080"));
        assert_eq!(base.duration, Some(101.5));
        assert_eq!(base.file_size, Some(12345));
    }
}
