//! 历史记录命令
//!
//! 任务终态快照的查询与清理（独立于任务表）

use tauri::State;

use super::api;
use crate::domain::task::HistoryRecord;
use crate::infrastructure::Database;

/// 加载全部历史（按完成时间倒序）
#[tauri::command(rename_all = "camelCase")]
pub async fn load_history(db: State<'_, Database>) -> Result<Vec<HistoryRecord>, String> {
    api(db.history.load_all())
}

/// 删除单条历史
#[tauri::command(rename_all = "camelCase")]
pub async fn delete_history_record(db: State<'_, Database>, id: i64) -> Result<(), String> {
    api(db.history.delete(id))
}

/// 清空历史，返回删除数量
#[tauri::command(rename_all = "camelCase")]
pub async fn clear_history(db: State<'_, Database>) -> Result<usize, String> {
    api(db.history.clear())
}
