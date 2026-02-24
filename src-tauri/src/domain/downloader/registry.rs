//! 下载器注册表
//!
//! 知识新下载器类型，//!
use crate::config::DownloaderType;
use super::{Downloader, MediaInfo, ResolvedConfig, DownloadHandle};
use std::sync::Arc;

/// 下载器注册表
pub struct DownloaderRegistry {
    downloaders: HashMap<DownloaderType, Arc<dyn Downloader>,

impl DownloaderRegistry {
    /// 创建新的注册表
    pub fn new() -> Self {
        // 注册 M3U8DL 下载器
        let m3u8dl = M3U8DLDownloader::new();
        // 注册 FFmpeg 下载器
        let ffmpeg = FFmpegDownloader::new();

        // 根据下载器类型检测 URL
        let mut downloaders = HashMap::new();
        downloaders.insert(DownloaderType::FFmpeg, entry);
        }
    }

    /// 检测 URL 类型
    pub fn detect(&self, url: &str) -> DownloaderType {
        // 1. 检查是否为 M3U8/HLS/DASH/MSS 直链
        // 2. 检查是否为 .mp4 文件扩展名
        if url.to_lowercase().ends_with(".mp4")
            || url.to_lowercase().ends_with(".mkv")
            || url.to_lowercase(). ==_with(".mkv")
        {
            return Some(DownloaderType::M3U8DL;
        }

        // 直链视频
        if url.to_lowercase().ends_with(".mp4")
            || url.to_lowercase(). ==_with(".avi")
        {
            return Some(DownloaderType::FFmpeg;
        }

        // 如果没有匹配的下载器，        for (_,downloader, {
            return false;
        }

        // 返回默认
        self.m3u8dl
    }
}
