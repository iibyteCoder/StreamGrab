//! 任务相关命令
//!
//! 处理任务的 CRUD 操作

use tauri::AppHandle;

use crate::db::TaskRecord;

use super::utils::get_db;

/// 加载所有任务
#[tauri::command]
pub async fn load_all_tasks(app: AppHandle) -> Result<Vec<TaskRecord>, String> {
    log::info!("Loading all tasks");

    let db = get_db(&app)?;
    db.tasks.load_all()
}

/// 加载可恢复的任务
#[tauri::command]
pub async fn load_recoverable_tasks(app: AppHandle) -> Result<Vec<TaskRecord>, String> {
    log::info!("Loading recoverable tasks");

    let db = get_db(&app)?;
    db.tasks.load_recoverable()
}

/// 保存任务（创建或更新）
#[tauri::command]
pub async fn save_task(task: TaskRecord, app: AppHandle) -> Result<(), String> {
    log::info!("Saving task: {}", task.id);

    let db = get_db(&app)?;
    db.tasks.upsert(&task)
}

/// 批量保存任务
#[tauri::command]
pub async fn save_tasks(tasks: Vec<TaskRecord>, app: AppHandle) -> Result<(), String> {
    log::info!("Saving {} tasks", tasks.len());

    let db = get_db(&app)?;
    for task in tasks {
        db.tasks.upsert(&task)?;
    }

    Ok(())
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

/// 更新任务进度
#[tauri::command]
pub async fn update_task_progress(
    task_id: String,
    progress_json: String,
    app: AppHandle,
) -> Result<(), String> {
    let db = get_db(&app)?;
    db.tasks.update_progress(&task_id, &progress_json)
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
