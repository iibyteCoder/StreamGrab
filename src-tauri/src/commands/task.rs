//! 任务相关命令
//!
//! 处理任务的 CRUD 操作

use tauri::AppHandle;

use crate::db::task::{FullTaskRecord, TaskMediaInfo, TaskRecord};

use super::utils::get_db;

/// 加载所有任务
#[tauri::command]
pub async fn load_all_tasks(app: AppHandle) -> Result<Vec<FullTaskRecord>, String> {
    log::info!("Loading all tasks");

    let db = get_db(&app)?;
    db.tasks.load_all()
}

/// 加载可恢复的任务
#[tauri::command]
pub async fn load_recoverable_tasks(app: AppHandle) -> Result<Vec<FullTaskRecord>, String> {
    log::info!("Loading recoverable tasks");

    let db = get_db(&app)?;
    db.tasks.load_recoverable()
}

/// 创建任务
#[tauri::command]
pub async fn create_task(task: TaskRecord, app: AppHandle) -> Result<(), String> {
    log::info!("Creating task: {}", task.id);

    let db = get_db(&app)?;
    db.tasks.create(&task)
}

/// 更新任务状态
#[tauri::command]
pub async fn update_task_status(
    task_id: String,
    status: String,
    error: Option<String>,
    app: AppHandle,
) -> Result<(), String> {
    log::info!("Updating task {} status to: {}", task_id, status);

    let db = get_db(&app)?;
    db.tasks.update_status(&task_id, &status, error.as_deref())
}

/// 更新任务输出路径
#[tauri::command]
pub async fn update_task_output_path(
    task_id: String,
    output_path: String,
    app: AppHandle,
) -> Result<(), String> {
    log::info!("Updating task {} output path", task_id);

    let db = get_db(&app)?;
    db.tasks.update_output_path(&task_id, &output_path)
}

/// 更新任务进度
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn update_task_progress(
    task_id: String,
    percent: i32,
    speed: i64,
    downloaded_size: i64,
    total_size: i64,
    downloaded_segments: i32,
    total_segments: i32,
    eta: i32,
    current_action: String,
    app: AppHandle,
) -> Result<(), String> {
    let db = get_db(&app)?;
    db.tasks.update_progress(
        &task_id,
        percent,
        speed,
        downloaded_size,
        total_size,
        downloaded_segments,
        total_segments,
        eta,
        &current_action,
    )
}

/// 更新任务媒体信息
#[tauri::command]
pub async fn update_task_media_info(
    task_id: String,
    media_info: TaskMediaInfo,
    app: AppHandle,
) -> Result<(), String> {
    log::info!("Updating task {} media info", task_id);
    let db = get_db(&app)?;
    db.tasks.update_media_info(&task_id, &media_info)
}

/// 删除任务
#[tauri::command]
pub async fn delete_task(task_id: String, app: AppHandle) -> Result<(), String> {
    log::info!("Deleting task: {}", task_id);

    let db = get_db(&app)?;
    db.tasks.delete(&task_id)
}

/// 清除已完成的任务
#[tauri::command]
pub async fn clear_finished_tasks(app: AppHandle) -> Result<usize, String> {
    log::info!("Clearing finished tasks");

    let db = get_db(&app)?;
    db.tasks.clear_finished()
}

/// 标记活跃任务为已中断
#[tauri::command]
pub async fn mark_active_tasks_interrupted(app: AppHandle) -> Result<usize, String> {
    log::info!("Marking active tasks as interrupted");

    let db = get_db(&app)?;
    db.tasks.mark_active_interrupted()
}

/// 清除所有任务
#[tauri::command]
pub async fn clear_all_tasks(app: AppHandle) -> Result<(), String> {
    log::info!("Clearing all tasks");

    let db = get_db(&app)?;
    db.tasks.clear_all()
}

/// 获取任务进度历史
#[tauri::command]
pub async fn get_progress_history(
    task_id: String,
    limit: Option<usize>,
    app: AppHandle,
) -> Result<Vec<crate::db::task::ProgressHistoryRecord>, String> {
    log::info!("Getting progress history for task: {}", task_id);

    let db = get_db(&app)?;
    db.tasks.get_progress_history(&task_id, limit)
}

/// 清除任务进度历史
#[tauri::command]
pub async fn clear_progress_history(task_id: String, app: AppHandle) -> Result<(), String> {
    log::info!("Clearing progress history for task: {}", task_id);

    let db = get_db(&app)?;
    db.tasks.clear_progress_history(&task_id)
}

/// 保存进度历史记录
#[tauri::command]
pub async fn save_progress_history(
    task_id: String,
    percent: i32,
    speed: i64,
    downloaded_size: i64,
    app: AppHandle,
) -> Result<(), String> {
    let db = get_db(&app)?;
    db.tasks
        .add_progress_history(&task_id, percent, speed, downloaded_size)
}
