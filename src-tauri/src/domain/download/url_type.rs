//! URL 类型检测与引擎分派
//!
//! 纯业务逻辑：识别 URL 的媒体类型，并映射到默认下载引擎。
//! 添加任务时用户无需选择工具——分派在此自动完成。

use super::engine::ToolId;
use serde::{Deserialize, Serialize};

/// URL 类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

        // 检查流媒体扩展名
        if url_lower.ends_with(".m3u8") || url_lower.contains(".m3u8?") {
            return UrlType::Hls;
        }
        if url_lower.ends_with(".mpd") || url_lower.contains(".mpd?") {
            return UrlType::Dash;
        }
        if url_lower.ends_with(".ism/manifest")
            || url_lower.contains(".ism/manifest?")
            || url_lower.ends_with(".isml/manifest")
            || url_lower.contains(".isml/manifest?")
        {
            return UrlType::Mss;
        }

        // 检查常见视频/音频扩展名
        let video_extensions = [
            ".mp4", ".mkv", ".avi", ".mov", ".wmv", ".flv", ".webm", ".m4v", ".ts", ".m2ts",
            ".mp3", ".m4a", ".aac", ".ogg", ".flac", ".wav",
        ];

        for ext in &video_extensions {
            if url_lower.ends_with(ext) || url_lower.contains(&format!("{ext}?")) {
                return UrlType::HttpVideo;
            }
        }

        UrlType::Unknown
    }

    /// 该类型的默认引擎（`None` = 未知类型，由注册表回退兜底）
    pub fn engine(self) -> Option<ToolId> {
        match self {
            Self::Hls | Self::Dash | Self::Mss => Some(ToolId::Nm3u8dl),
            Self::HttpVideo => Some(ToolId::Ffmpeg),
            Self::Unknown => None,
        }
    }

    /// 是否需要使用 FFmpeg 下载
    pub fn needs_ffmpeg(self) -> bool {
        matches!(self, Self::HttpVideo)
    }

    /// 是否是流媒体格式（N_m3u8DL-RE 支持）
    pub fn is_streaming(self) -> bool {
        matches!(self, Self::Hls | Self::Dash | Self::Mss)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_stream_urls() {
        assert_eq!(
            UrlType::detect("https://example.com/video/index.m3u8"),
            UrlType::Hls
        );
        assert_eq!(
            UrlType::detect("https://example.com/index.m3u8?token=abc"),
            UrlType::Hls
        );
        assert_eq!(
            UrlType::detect("https://example.com/manifest.mpd"),
            UrlType::Dash
        );
        assert_eq!(
            UrlType::detect("https://example.com/video.ism/manifest"),
            UrlType::Mss
        );
        assert_eq!(
            UrlType::detect("https://example.com/video.isml/manifest?filter=x"),
            UrlType::Mss
        );
    }

    #[test]
    fn detect_direct_video_urls() {
        assert_eq!(
            UrlType::detect("https://example.com/movie.mp4"),
            UrlType::HttpVideo
        );
        assert_eq!(
            UrlType::detect("https://example.com/movie.MP4?sig=1"),
            UrlType::HttpVideo
        );
        assert_eq!(
            UrlType::detect("https://example.com/song.mp3"),
            UrlType::HttpVideo
        );
        assert_eq!(
            UrlType::detect("https://example.com/clip.webm"),
            UrlType::HttpVideo
        );
    }

    #[test]
    fn detect_unknown_urls() {
        assert_eq!(
            UrlType::detect("https://example.com/page.html"),
            UrlType::Unknown
        );
        assert_eq!(UrlType::detect("https://example.com/"), UrlType::Unknown);
        assert_eq!(UrlType::detect("not a url"), UrlType::Unknown);
    }

    #[test]
    fn engine_mapping() {
        assert_eq!(UrlType::Hls.engine(), Some(ToolId::Nm3u8dl));
        assert_eq!(UrlType::Dash.engine(), Some(ToolId::Nm3u8dl));
        assert_eq!(UrlType::Mss.engine(), Some(ToolId::Nm3u8dl));
        assert_eq!(UrlType::HttpVideo.engine(), Some(ToolId::Ffmpeg));
        assert_eq!(UrlType::Unknown.engine(), None);

        assert!(UrlType::HttpVideo.needs_ffmpeg());
        assert!(!UrlType::Hls.needs_ffmpeg());
        assert!(UrlType::Hls.is_streaming());
        assert!(!UrlType::HttpVideo.is_streaming());
    }
}
