//! 密钥相关命令
//!
//! 处理解密密钥的 CRUD 操作

use std::sync::Arc;
use tauri::{AppHandle, Manager};

use crate::db::{Database, KeyRecord};

/// 获取数据库实例
fn get_db(app: &AppHandle) -> Result<Arc<Database>, String> {
    app.try_state::<Arc<Database>>()
        .map(|s| s.inner().clone())
        .ok_or_else(|| "Database not initialized".to_string())
}

/// 加载所有密钥
#[tauri::command]
pub async fn load_keys(app: AppHandle) -> Result<Vec<KeyRecord>, String> {
    log::info!("Loading all keys");

    let db = get_db(&app)?;
    db.keys.load_all()
}

/// 添加密钥
#[tauri::command]
pub async fn add_key(key: KeyRecord, app: AppHandle) -> Result<(), String> {
    log::info!("Adding key: {}", key.id);

    let db = get_db(&app)?;
    db.keys.add(&key)
}

/// 更新密钥
#[tauri::command]
pub async fn update_key(key: KeyRecord, app: AppHandle) -> Result<(), String> {
    log::info!("Updating key: {}", key.id);

    let db = get_db(&app)?;
    db.keys.update(&key)
}

/// 删除密钥
#[tauri::command]
pub async fn delete_key(id: String, app: AppHandle) -> Result<(), String> {
    log::info!("Deleting key: {}", id);

    let db = get_db(&app)?;
    db.keys.delete(&id)
}

/// 清除所有密钥
#[tauri::command]
pub async fn clear_keys(app: AppHandle) -> Result<(), String> {
    log::info!("Clearing all keys");

    let db = get_db(&app)?;
    db.keys.clear()
}

/// 记录密钥使用时间
#[tauri::command]
pub async fn record_key_usage(id: String, app: AppHandle) -> Result<(), String> {
    let db = get_db(&app)?;
    db.keys.record_usage(&id)
}
