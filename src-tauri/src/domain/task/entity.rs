//! 任务实体
//!
//! 定义任务的核心属性与状态机，与基础设施无关

use crate::shared::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::fmt;

/// 任务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TaskStatus {
    /// 等待中（含等待定时开始）
    Pending,
    /// 解析中
    Analyzing,
    /// 下载中
    Downloading,
    /// 二进制合并中
    Merging,
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
    pub fn is_active(self) -> bool {
        matches!(
            self,
            Self::Analyzing | Self::Downloading | Self::Merging | Self::Muxing
        )
    }

    /// 是否是最终状态
    pub fn is_finished(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }

    /// 持久化字符串
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Analyzing => "analyzing",
            Self::Downloading => "downloading",
            Self::Merging => "merging",
            Self::Muxing => "muxing",
            Self::Paused => "paused",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    /// 从持久化字符串解析
    pub fn parse(s: &str) -> AppResult<Self> {
        match s {
            "pending" => Ok(Self::Pending),
            "analyzing" => Ok(Self::Analyzing),
            "downloading" => Ok(Self::Downloading),
            "merging" => Ok(Self::Merging),
            "muxing" => Ok(Self::Muxing),
            "paused" => Ok(Self::Paused),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            other => Err(AppError::parse(format!("未知任务状态: {other}"))),
        }
    }

    /// 状态机：判断从当前状态迁移到 `next` 是否合法
    ///
    /// 同状态幂等更新（如进度刷新写回 downloading）始终允许；
    /// 终态只能通过「重试/重新下载」回到 pending（由命令层显式触发）。
    pub fn can_transition_to(self, next: TaskStatus) -> bool {
        use TaskStatus::*;
        if self == next {
            return true;
        }
        matches!(
            (self, next),
            (
                Pending,
                Analyzing | Downloading | Paused | Failed | Cancelled
            ) | (Analyzing, Downloading | Paused | Failed | Cancelled)
                | (
                    Downloading,
                    Merging | Muxing | Completed | Paused | Failed | Cancelled
                )
                | (Merging, Muxing | Completed | Failed | Cancelled)
                | (Muxing, Completed | Failed | Cancelled)
                | (Paused, Analyzing | Downloading | Failed | Cancelled)
                | (Completed | Failed | Cancelled, Pending)
        )
    }
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for TaskStatus {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
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
    /// 是否被中断（应用退出时活跃任务置位，重启后可恢复）
    pub was_interrupted: bool,
}

/// 进度数据
///
/// 同时用于实时事件推送与 `tasks.progress_json` 列持久化
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ProgressData {
    /// 当前流进度百分比
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_round_trip() {
        for s in [
            "pending",
            "analyzing",
            "downloading",
            "merging",
            "muxing",
            "paused",
            "completed",
            "failed",
            "cancelled",
        ] {
            let status = TaskStatus::parse(s).unwrap();
            assert_eq!(status.as_str(), s);
            assert_eq!(status.to_string(), s);
        }
        assert!(TaskStatus::parse("bogus").is_err());
    }

    #[test]
    fn active_and_finished_classification() {
        assert!(TaskStatus::Downloading.is_active());
        assert!(TaskStatus::Merging.is_active());
        assert!(TaskStatus::Muxing.is_active());
        assert!(!TaskStatus::Pending.is_active());
        assert!(!TaskStatus::Paused.is_active());

        assert!(TaskStatus::Completed.is_finished());
        assert!(TaskStatus::Failed.is_finished());
        assert!(TaskStatus::Cancelled.is_finished());
        assert!(!TaskStatus::Downloading.is_finished());
    }

    #[test]
    fn state_machine_allows_normal_flow() {
        use TaskStatus::*;
        // 正常生命周期
        assert!(Pending.can_transition_to(Analyzing));
        assert!(Analyzing.can_transition_to(Downloading));
        assert!(Downloading.can_transition_to(Merging));
        assert!(Merging.can_transition_to(Muxing));
        assert!(Muxing.can_transition_to(Completed));
        // 幂等
        assert!(Downloading.can_transition_to(Downloading));
        // 任意活跃态可失败/取消
        assert!(Downloading.can_transition_to(Failed));
        assert!(Muxing.can_transition_to(Cancelled));
        // 暂停与恢复
        assert!(Downloading.can_transition_to(Paused));
        assert!(Paused.can_transition_to(Downloading));
        // 重试
        assert!(Failed.can_transition_to(Pending));
        assert!(Completed.can_transition_to(Pending));
    }

    #[test]
    fn state_machine_rejects_illegal_flow() {
        use TaskStatus::*;
        assert!(!Pending.can_transition_to(Completed));
        assert!(!Downloading.can_transition_to(Analyzing));
        assert!(!Completed.can_transition_to(Downloading));
        assert!(!Paused.can_transition_to(Completed));
        assert!(!Analyzing.can_transition_to(Muxing));
    }

    #[test]
    fn progress_data_serde_is_camel_case() {
        let p = ProgressData {
            percent: 42,
            speed: 1024,
            ..Default::default()
        };
        let json = serde_json::to_value(&p).unwrap();
        assert_eq!(json["overallPercent"], 0);
        assert_eq!(json["downloadedSize"], 0);
        assert_eq!(json["percent"], 42);
        // 缺字段的旧 JSON 可以反序列化（default 兜底）
        let partial: ProgressData =
            serde_json::from_str(r#"{"percent": 10, "speed": 100}"#).unwrap();
        assert_eq!(partial.percent, 10);
        assert_eq!(partial.total_size, 0);
    }
}
