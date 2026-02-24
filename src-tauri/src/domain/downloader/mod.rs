//! 下载器领域模块
//!
//! 定义下载器抽象和具体实现

mod m3u8dl;
mod ffmpeg;
mod registry;

pub use m3u8dl::M3U8DLDownloader;
pub use ffmpeg::FFmpegDownloader;
pub use registry::DownloaderRegistry;

use super::config::{DownloaderType, ResolvedConfig};
use std::sync::Arc;

/// 下载器抽象 trait
///
/// 定义所有下载器必须实现的接口
pub trait Downloader: Send + Sync {
    /// 检测是否支持该 URL
    ///
    /// 返回 true 表示此下载器可以处理该 URL
    fn detect(&self, url: &str) -> bool;

    /// 解析媒体信息
    ///
    /// 返回媒体信息（分辨率、编码、时长等）
    fn parse(&self, url: &str, config: &ResolvedConfig) -> Result<MediaInfo, String>;

    /// 执行下载
    ///
    /// 返回下载句柄，    fn download(
        &self,
        url: &str,
        config: &ResolvedConfig,
        on_progress: Option<Box<dyn Fn(ProgressData) + Send + Sync>>,
    ) -> Result<DownloadHandle, String>;

    /// 构建命令行参数
    ///
    /// 用于调试和日志记录
    fn build_cmd(&self, url: &str, config: &ResolvedConfig) -> Vec<String>;
}

/// 媒体信息
#[derive(Debug, Clone)]
pub struct MediaInfo {
    /// 视频流列表
    pub video_streams: Vec<StreamInfo>,
    /// 音频流列表
    pub audio_streams: Vec<StreamInfo>,
    /// 字幕流列表
    pub subtitle_streams: Vec<StreamInfo>,
    /// 是否为直播
    pub is_live: bool,
    /// 是否加密
    pub is_encrypted: bool,
    /// 总时长（秒）
    pub duration: Option<f64>,
    /// 分片数量
    pub segment_count: Option<i32>,
    /// 错误信息
    pub error: Option<String>,
}

/// 流信息
#[derive(Debug, Clone)]
pub struct StreamInfo {
    /// 流 ID
    pub id: String,
    /// 语言代码
    pub language: Option<String>,
    /// 描述/名称
    pub name: Option<String>,
    /// 编解码器
    pub codecs: Option<String>,
    /// 分辨率（视频）
    pub resolution: Option<String>,
    /// 帧率（视频）
    pub frame_rate: Option<f64>,
    /// 带宽（视频）
    pub bandwidth: Option<i64>,
    /// 声道数（音频）
    pub channels: Option<String>,
    /// 媒体类型
    pub media_type: StreamMediaType,
    /// 色域（视频）
    pub video_range: Option<String>,
    /// 分片数量
    pub segments: Option<i32>,
    /// 播放列表时长
    pub playlist_duration: Option<f64>,
}

/// 流媒体类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamMediaType {
    Video,
    Audio,
    Subtitle,
}

/// 下载进度数据
#[derive(Debug, Clone)]
pub struct ProgressData {
    /// 任务 ID
    pub task_id: String,
    /// 进度百分比
    pub percent: i32,
    /// 下载速度（字节/秒）
    pub speed: i64,
    /// 已下载大小
    pub downloaded_size: i64,
    /// 总大小
    pub total_size: i64,
    /// 已下载分片数
    pub downloaded_segments: i32,
    /// 总分片数
    pub total_segments: i32,
    /// 剩余时间（秒）
    pub eta: i32,
    /// 当前操作
    pub current_action: String,
}

/// 下载句柄
///
/// 用于控制下载进程
pub struct DownloadHandle {
    /// 任务 ID
    pub task_id: String,
    /// 进程 ID（如果有）
    pub pid: Option<u32>,
    /// 停止标志
    stopped: Arc<std::sync::atomic::AtomicBool>,
}

impl DownloadHandle {
    /// 创建新的下载句柄
    pub fn new(task_id: String) -> Self {
        Self {
            task_id,
            pid: None,
            stopped: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// 停止下载
    pub fn stop(&self) {
        self.stopped
            .store(true, std::sync::Ordering::Relaxed);
    }

    /// 检查是否已停止
    pub fn is_stopped(&self) -> bool {
        self.stopped.load(std::sync::Ordering::Relaxed)
    }
}
