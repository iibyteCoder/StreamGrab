//! 任务实体
//!
//! 定义任务的核心属性，与基础设施无关

use serde::{Deserialize, Serialize};

/// 任务状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TaskStatus {
    /// 等待中
    Pending,
    /// 解析中
    Analyzing,
    /// 下载中
    Downloading,
    /// 混流中
    Muxing,
    /// 已暂停
    Paused,
    /// 已完成
    Completed,
    /// 已失败
    Failed,
    /// 已取消
    Cancelled,
}

impl TaskStatus {
    /// 是否是活跃状态（正在处理）
    pub fn is_active(&self) -> bool {
        matches!(
            self,
            TaskStatus::Analyzing | TaskStatus::Downloading | TaskStatus::Muxing
        )
    }

    /// 是否是最终状态
    pub fn is_finished(&self) -> bool {
        matches!(
            self,
            TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled
        )
    }
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Pending => write!(f, "pending"),
            TaskStatus::Analyzing => write!(f, "analyzing"),
            TaskStatus::Downloading => write!(f, "downloading"),
            TaskStatus::Muxing => write!(f, "muxing"),
            TaskStatus::Paused => write!(f, "paused"),
            TaskStatus::Completed => write!(f, "completed"),
            TaskStatus::Failed => write!(f, "failed"),
            TaskStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// 任务实体
#[derive(Debug, Clone)]
pub struct TaskEntity {
    /// 唯一标识
    pub id: String,
    /// 下载 URL
    pub url: String,
    /// 文件名
    pub file_name: String,
    /// 保存目录
    pub save_dir: String,
    /// 输出路径（完成后）
    pub output_path: Option<String>,
    /// 当前状态
    pub status: TaskStatus,
    /// 错误信息
    pub error: Option<String>,
    /// 是否被中断
    pub was_interrupted: bool,
}

/// 进度数据
#[derive(Debug, Clone, Default)]
pub struct ProgressData {
    /// 进度百分比
    pub percent: i32,
    /// 总体进度（视频+音频合并）
    pub overall_percent: i32,
    /// 下载速度 (bytes/s)
    pub speed: i64,
    /// 已下载大小
    pub downloaded_size: i64,
    /// 总大小
    pub total_size: i64,
    /// 已下载分片数
    pub downloaded_segments: i32,
    /// 总分片数
    pub total_segments: i32,
    /// 预估剩余时间 (秒)
    pub eta: i32,
    /// 当前操作描述
    pub current_action: String,
}

/// 媒体信息
#[derive(Debug, Clone, Default)]
pub struct MediaInfo {
    /// 分辨率 (如 "1920x1080")
    pub resolution: Option<String>,
    /// 视频宽度
    pub width: Option<i32>,
    /// 视频高度
    pub height: Option<i32>,
    /// 帧率
    pub frame_rate: Option<f64>,
    /// 视频编码
    pub video_codec: Option<String>,
    /// 视频范围 (SDR/HDR)
    pub video_range: Option<String>,
    /// 音频编码
    pub audio_codec: Option<String>,
    /// 音频声道数
    pub audio_channels: Option<String>,
    /// 音频语言
    pub audio_language: Option<String>,
    /// 总时长 (秒)
    pub duration: Option<f64>,
    /// 分片数
    pub segment_count: Option<i32>,
    /// 是否是直播
    pub is_live: bool,
    /// 是否加密
    pub is_encrypted: bool,
    /// 文件格式
    pub file_format: Option<String>,
}
