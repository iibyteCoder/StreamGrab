//! URL 类型检测
//!
//! 纯业务逻辑，检测 URL 的流媒体类型

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
