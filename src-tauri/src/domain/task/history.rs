//! 历史记录
//!
//! 任务进入终态（completed/failed/cancelled）时写入一条快照，
//! 独立于任务表：清除任务不删除历史。

use super::entity::TaskStatus;
use super::overrides::TaskOverrides;
use serde::{Deserialize, Serialize};

/// 历史记录
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryRecord {
    /// 自增 ID
    pub id: i64,
    /// 来源任务 ID（不加外键，任务删除后保留历史）
    pub task_id: Option<String>,
    pub url: String,
    pub file_name: String,
    pub save_dir: String,
    pub output_path: Option<String>,
    /// 文件大小（字节）
    pub file_size: Option<i64>,
    /// 终态状态
    pub status: TaskStatus,
    /// 失败原因（status=failed 时）
    pub error: Option<String>,
    /// 任务创建时间
    pub created_at: String,
    /// 到达终态时间
    pub completed_at: String,
    /// 任务级覆盖快照（「重新下载」时携带原参数）
    pub overrides: Option<TaskOverrides>,
}
