//! 任务命令
//!
//! 任务 CRUD + 状态机校验 + 终态写入历史

use tauri::State;

use super::api;
use crate::domain::media::MediaInfo;
use crate::domain::task::TaskOverrides;
use crate::domain::task::{ProgressData, TaskRecord, TaskStatus};
use crate::infrastructure::db::repository::ProgressSample;
use crate::infrastructure::Database;

/// 加载全部任务（按创建时间倒序）
#[tauri::command(rename_all = "camelCase")]
pub async fn load_all_tasks(db: State<'_, Database>) -> Result<Vec<TaskRecord>, String> {
    api(db.tasks.load_all())
}

/// 加载可恢复任务（被中断的 + 活跃状态的）
#[tauri::command(rename_all = "camelCase")]
pub async fn load_recoverable_tasks(db: State<'_, Database>) -> Result<Vec<TaskRecord>, String> {
    api(db.tasks.load_recoverable())
}

/// 获取单个任务
#[tauri::command(rename_all = "camelCase")]
pub async fn get_task(
    db: State<'_, Database>,
    task_id: String,
) -> Result<Option<TaskRecord>, String> {
    api(db.tasks.get(&task_id))
}

/// 创建任务（携带任务级覆盖）
#[tauri::command(rename_all = "camelCase")]
pub async fn create_task(db: State<'_, Database>, task: TaskRecord) -> Result<(), String> {
    api(db.tasks.create(&task))
}

/// 更新任务状态
///
/// 经状态机校验合法迁移；进入终态时自动写入历史记录
#[tauri::command(rename_all = "camelCase")]
pub async fn update_task_status(
    db: State<'_, Database>,
    task_id: String,
    status: String,
    error: Option<String>,
) -> Result<(), String> {
    api((|| {
        let next = TaskStatus::parse(&status)?;
        let current = db
            .tasks
            .get(&task_id)?
            .ok_or_else(|| crate::shared::AppError::other(format!("任务不存在: {task_id}")))?;

        if !current.status.can_transition_to(next) {
            return Err(crate::shared::AppError::other(format!(
                "非法状态迁移: {} → {}",
                current.status, next
            )));
        }

        db.tasks.update_status(&task_id, next, error.as_deref())?;

        // 终态 → 写入历史快照
        if next.is_finished() {
            if let Some(updated) = db.tasks.get(&task_id)? {
                db.history.insert_from_task(&updated)?;
            }
        }
        Ok(())
    })())
}

/// 更新输出路径
#[tauri::command(rename_all = "camelCase")]
pub async fn update_task_output_path(
    db: State<'_, Database>,
    task_id: String,
    output_path: String,
) -> Result<(), String> {
    api(db.tasks.update_output_path(&task_id, &output_path))
}

/// 更新任务进度（JSON 列）
#[tauri::command(rename_all = "camelCase")]
pub async fn update_task_progress(
    db: State<'_, Database>,
    task_id: String,
    progress: ProgressData,
) -> Result<(), String> {
    api(db.tasks.update_progress(&task_id, &progress))
}

/// 更新媒体信息（JSON 列）
#[tauri::command(rename_all = "camelCase")]
pub async fn update_task_media_info(
    db: State<'_, Database>,
    task_id: String,
    media_info: MediaInfo,
) -> Result<(), String> {
    api(db.tasks.update_media_info(&task_id, &media_info))
}

/// 保存任务级覆盖
#[tauri::command(rename_all = "camelCase")]
pub async fn save_task_overrides(
    db: State<'_, Database>,
    task_id: String,
    overrides: TaskOverrides,
) -> Result<(), String> {
    api(db.tasks.save_overrides(&task_id, &overrides))
}

/// 删除任务（含进度历史）
#[tauri::command(rename_all = "camelCase")]
pub async fn delete_task(db: State<'_, Database>, task_id: String) -> Result<(), String> {
    api(db.tasks.delete(&task_id))
}

/// 清除已结束任务，返回删除数量（历史记录保留）
#[tauri::command(rename_all = "camelCase")]
pub async fn clear_finished_tasks(db: State<'_, Database>) -> Result<usize, String> {
    api(db.tasks.clear_finished())
}

/// 清空全部任务
#[tauri::command(rename_all = "camelCase")]
pub async fn clear_all_tasks(db: State<'_, Database>) -> Result<(), String> {
    api(db.tasks.clear_all())
}

/// 将活跃任务标记为已中断（应用启动时调用）
#[tauri::command(rename_all = "camelCase")]
pub async fn mark_active_tasks_interrupted(db: State<'_, Database>) -> Result<usize, String> {
    api(db.tasks.mark_active_interrupted())
}

/// 查询任务的速率曲线数据
#[tauri::command(rename_all = "camelCase")]
pub async fn get_progress_history(
    db: State<'_, Database>,
    task_id: String,
    limit: Option<usize>,
) -> Result<Vec<ProgressSample>, String> {
    api(db.progress.query(&task_id, limit))
}

/// 清除任务的速率曲线数据
#[tauri::command(rename_all = "camelCase")]
pub async fn clear_progress_history(
    db: State<'_, Database>,
    task_id: String,
) -> Result<(), String> {
    api(db.progress.clear(&task_id))
}
