//! 任务预设命令
//!
//! 预设 = 命名的 TaskOverrides 组合，DB 持久化

use tauri::State;

use super::api;
use crate::domain::task::TaskPreset;
use crate::infrastructure::Database;

/// 加载全部预设
#[tauri::command(rename_all = "camelCase")]
pub async fn load_presets(db: State<'_, Database>) -> Result<Vec<TaskPreset>, String> {
    api(db.presets.load_all())
}

/// 保存预设（按 ID upsert）
#[tauri::command(rename_all = "camelCase")]
pub async fn save_preset(db: State<'_, Database>, preset: TaskPreset) -> Result<(), String> {
    api(db.presets.save(&preset))
}

/// 删除预设
#[tauri::command(rename_all = "camelCase")]
pub async fn delete_preset(db: State<'_, Database>, id: String) -> Result<(), String> {
    api(db.presets.delete(&id))
}
