//! 配置相关命令
//!
//! 处理应用配置的读取、保存、导入、导出

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Manager};

use serde::{Deserialize, Serialize};

use crate::db::{Database, HistoryRecord};

// ============================================
// 配置相关命令（使用 SQLite）
// ============================================

/// 获取数据库实例
fn get_db(app: &AppHandle) -> Result<Arc<Database>, String> {
    app.try_state::<Arc<Database>>()
        .map(|s| s.inner().clone())
        .ok_or_else(|| "Database not initialized".to_string())
}

/// 加载所有配置
#[tauri::command]
pub async fn load_settings(app: AppHandle) -> Result<std::collections::HashMap<String, serde_json::Value>, String> {
    log::info!("Loading all settings");

    let db = get_db(&app)?;
    db.settings.load_all()
}

/// 保存单个配置模块
#[tauri::command]
pub async fn save_setting(
    key: String,
    value: serde_json::Value,
    app: AppHandle,
) -> Result<(), String> {
    log::info!("Saving setting: {}", key);

    let db = get_db(&app)?;
    db.settings.save(&key, &value)
}

/// 批量保存配置
#[tauri::command]
pub async fn save_settings(
    settings: std::collections::HashMap<String, serde_json::Value>,
    app: AppHandle,
) -> Result<(), String> {
    log::info!("Saving {} settings sections", settings.len());

    let db = get_db(&app)?;
    db.settings.save_all(&settings)
}

/// 重置单个配置模块
#[tauri::command]
pub async fn reset_setting(key: String, app: AppHandle) -> Result<(), String> {
    log::info!("Resetting setting: {}", key);

    let db = get_db(&app)?;
    db.settings.reset(&key)
}

/// 重置所有配置
#[tauri::command]
pub async fn reset_all_settings(app: AppHandle) -> Result<(), String> {
    log::info!("Resetting all settings");

    let db = get_db(&app)?;
    db.settings.reset_all()
}

// ============================================
// 配置导入/导出（文件系统）
// ============================================

/// 导出配置到指定路径
#[tauri::command]
pub async fn export_config(
    file_path: String,
    app: AppHandle,
) -> Result<(), String> {
    log::info!("Exporting config to: {}", file_path);

    let db = get_db(&app)?;
    let settings = db.settings.load_all()?;

    let content = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(&file_path, content)
        .map_err(|e| format!("Failed to export config: {}", e))?;

    Ok(())
}

/// 从指定路径导入配置
#[tauri::command]
pub async fn import_config(
    file_path: String,
    app: AppHandle,
) -> Result<(), String> {
    log::info!("Importing config from: {}", file_path);

    let content = fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let settings: std::collections::HashMap<String, serde_json::Value> = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse config file: {}", e))?;

    let db = get_db(&app)?;
    db.settings.save_all(&settings)
}

/// 获取数据库文件路径
#[tauri::command]
pub async fn get_db_path(app: AppHandle) -> Result<String, String> {
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("Failed to get config directory: {}", e))?;

    Ok(config_dir.join("streamgrab.db").to_string_lossy().to_string())
}

// ============================================
// 系统命令
// ============================================

/// 在文件管理器中打开路径
#[tauri::command]
pub async fn open_in_explorer(path: String) -> Result<(), String> {
    log::info!("Opening in explorer: {}", path);

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open explorer: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open finder: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open file manager: {}", e))?;
    }

    Ok(())
}

/// 检查文件是否存在
#[tauri::command]
pub async fn file_exists(path: String) -> Result<bool, String> {
    Ok(PathBuf::from(&path).exists())
}

/// 选择目录
#[tauri::command]
pub async fn select_directory(_app: AppHandle) -> Result<Option<String>, String> {
    // TODO: 使用 tauri-plugin-dialog 实现
    Ok(None)
}

/// 选择文件
#[tauri::command]
pub async fn select_file(
    _app: AppHandle,
    _filters: Option<Vec<FileFilter>>,
) -> Result<Option<String>, String> {
    // TODO: 使用 tauri-plugin-dialog 实现
    Ok(None)
}

/// 文件过滤器
#[derive(Debug, Serialize, Deserialize)]
pub struct FileFilter {
    pub name: String,
    pub extensions: Vec<String>,
}

// ============================================
// 历史记录相关命令（使用 SQLite）
// ============================================

/// 加载历史记录
#[tauri::command]
pub async fn load_history(app: AppHandle) -> Result<Vec<HistoryRecord>, String> {
    log::info!("Loading history records");

    let db = get_db(&app)?;
    db.history.load_all()
}

/// 添加历史记录
#[tauri::command]
pub async fn add_history_record(
    record: HistoryRecord,
    app: AppHandle,
) -> Result<(), String> {
    log::info!("Adding history record: {}", record.id);

    let db = get_db(&app)?;
    db.history.add(&record)
}

/// 清除历史记录
#[tauri::command]
pub async fn clear_history(app: AppHandle) -> Result<(), String> {
    log::info!("Clearing history records");

    let db = get_db(&app)?;
    db.history.clear()
}

/// 删除单条历史记录
#[tauri::command]
pub async fn delete_history_record(
    id: String,
    app: AppHandle,
) -> Result<(), String> {
    log::info!("Deleting history record: {}", id);

    let db = get_db(&app)?;
    db.history.delete(&id)
}
