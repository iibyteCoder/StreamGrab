//! 配置相关命令
//!
//! 处理应用配置的读取、保存、导入、导出

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Manager};

use serde::{Deserialize, Serialize};

use crate::db::{HistoryDb, HistoryRecord};

/// 加载配置
///
/// # Arguments
/// * `file_name` - 配置文件名
/// * `app` - Tauri 应用句柄
#[tauri::command]
pub async fn load_config(
    file_name: String,
    app: AppHandle,
) -> Result<serde_json::Value, String> {
    log::info!("Loading config: {}", file_name);

    let config_path = get_config_path(&app, &file_name)?;

    if !config_path.exists() {
        log::info!("Config file not found, returning empty object");
        return Ok(serde_json::json!({}));
    }

    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let config: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse config file: {}", e))?;

    Ok(config)
}

/// 保存配置
///
/// # Arguments
/// * `file_name` - 配置文件名
/// * `config` - 配置内容
/// * `app` - Tauri 应用句柄
#[tauri::command]
pub async fn save_config(
    file_name: String,
    config: serde_json::Value,
    app: AppHandle,
) -> Result<(), String> {
    log::info!("Saving config: {}", file_name);

    let config_path = get_config_path(&app, &file_name)?;

    // 确保目录存在
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    // 格式化 JSON
    let content = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    // 写入文件（原子操作）
    let temp_path = config_path.with_extension("tmp");
    fs::write(&temp_path, content)
        .map_err(|e| format!("Failed to write config file: {}", e))?;

    fs::rename(&temp_path, &config_path)
        .map_err(|e| format!("Failed to save config file: {}", e))?;

    Ok(())
}

/// 导出配置到指定路径
///
/// # Arguments
/// * `file_path` - 导出路径
/// * `config` - 配置内容
#[tauri::command]
pub async fn export_config(
    file_path: String,
    config: serde_json::Value,
) -> Result<(), String> {
    log::info!("Exporting config to: {}", file_path);

    let content = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(&file_path, content)
        .map_err(|e| format!("Failed to export config: {}", e))?;

    Ok(())
}

/// 从指定路径导入配置
///
/// # Arguments
/// * `file_path` - 导入路径
#[tauri::command]
pub async fn import_config(file_path: String) -> Result<serde_json::Value, String> {
    log::info!("Importing config from: {}", file_path);

    let content = fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let config: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse config file: {}", e))?;

    Ok(config)
}

/// 获取配置文件路径
///
/// # Arguments
/// * `file_name` - 配置文件名
/// * `app` - Tauri 应用句柄
#[tauri::command]
pub async fn get_config_path_cmd(
    file_name: String,
    app: AppHandle,
) -> Result<String, String> {
    let path = get_config_path(&app, &file_name)?;
    Ok(path.to_string_lossy().to_string())
}

/// 获取配置文件的完整路径
fn get_config_path(app: &AppHandle, file_name: &str) -> Result<PathBuf, String> {
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("Failed to get config directory: {}", e))?;

    Ok(config_dir.join(file_name))
}

/// 在文件管理器中打开路径
///
/// # Arguments
/// * `path` - 文件或目录路径
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
///
/// # Arguments
/// * `path` - 文件路径
#[tauri::command]
pub async fn file_exists(path: String) -> Result<bool, String> {
    Ok(PathBuf::from(&path).exists())
}

/// 选择目录
///
/// 使用系统对话框选择目录
#[tauri::command]
pub async fn select_directory(_app: AppHandle) -> Result<Option<String>, String> {
    // TODO: 使用 tauri-plugin-dialog 实现
    Ok(None)
}

/// 选择文件
///
/// 使用系统对话框选择文件
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

/// 获取历史记录数据库实例
fn get_history_db(app: &AppHandle) -> Result<Arc<HistoryDb>, String> {
    app.try_state::<Arc<HistoryDb>>()
        .map(|s| s.inner().clone())
        .ok_or_else(|| "History database not initialized".to_string())
}

/// 加载历史记录
#[tauri::command]
pub async fn load_history(app: AppHandle) -> Result<Vec<HistoryRecord>, String> {
    log::info!("Loading history records");

    let db = get_history_db(&app)?;
    db.load_all()
}

/// 保存历史记录（批量替换，用于导入）
#[tauri::command]
pub async fn save_history(
    records: Vec<HistoryRecord>,
    app: AppHandle,
) -> Result<(), String> {
    log::info!("Saving {} history records (batch)", records.len());

    let db = get_history_db(&app)?;

    // 先清除现有记录
    db.clear()?;

    // 批量添加
    for record in records {
        db.add(&record)?;
    }

    Ok(())
}

/// 添加历史记录
#[tauri::command]
pub async fn add_history_record(
    record: HistoryRecord,
    app: AppHandle,
) -> Result<(), String> {
    log::info!("Adding history record: {}", record.id);

    let db = get_history_db(&app)?;
    db.add(&record)
}

/// 清除历史记录
#[tauri::command]
pub async fn clear_history(app: AppHandle) -> Result<(), String> {
    log::info!("Clearing history records");

    let db = get_history_db(&app)?;
    db.clear()
}

/// 删除单条历史记录
#[tauri::command]
pub async fn delete_history_record(
    id: String,
    app: AppHandle,
) -> Result<(), String> {
    log::info!("Deleting history record: {}", id);

    let db = get_history_db(&app)?;
    db.delete(&id)
}
