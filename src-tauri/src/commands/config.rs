//! 配置相关命令
//!
//! 处理应用配置的读取、保存、导入、导出

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::DialogExt;

use serde::{Deserialize, Serialize};

use crate::db::Database;

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

/// 删除文件或文件夹
/// 如果是文件夹，会递归删除所有内容
#[tauri::command]
pub async fn delete_file_or_folder(path: String) -> Result<(), String> {
    log::info!("Deleting: {}", path);

    let path = PathBuf::from(&path);

    if !path.exists() {
        return Err("文件或文件夹不存在".to_string());
    }

    if path.is_file() {
        fs::remove_file(&path)
            .map_err(|e| format!("删除文件失败: {}", e))?;
    } else if path.is_dir() {
        fs::remove_dir_all(&path)
            .map_err(|e| format!("删除文件夹失败: {}", e))?;
    }

    log::info!("Successfully deleted: {:?}", path);
    Ok(())
}

/// 选择目录
#[tauri::command]
pub async fn select_directory(app: AppHandle) -> Result<Option<String>, String> {
    log::info!("Opening directory picker");

    let folder_path = app.dialog()
        .file()
        .blocking_pick_folder();

    match folder_path {
        Some(path) => {
            let path_str = path.to_string();
            log::info!("Selected directory: {}", path_str);
            Ok(Some(path_str))
        }
        None => {
            log::info!("Directory selection cancelled");
            Ok(None)
        }
    }
}

/// 选择文件
#[tauri::command]
pub async fn select_file(
    app: AppHandle,
    filters: Option<Vec<FileFilter>>,
) -> Result<Option<String>, String> {
    log::info!("Opening file picker with filters: {:?}", filters);

    let mut dialog = app.dialog().file();

    // 添加文件过滤器
    if let Some(filter_list) = filters {
        for filter in filter_list {
            dialog = dialog.add_filter(
                filter.name,
                &filter.extensions.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
            );
        }
    }

    let file_path = dialog.blocking_pick_file();

    match file_path {
        Some(path) => {
            let path_str = path.to_string();
            log::info!("Selected file: {}", path_str);
            Ok(Some(path_str))
        }
        None => {
            log::info!("File selection cancelled");
            Ok(None)
        }
    }
}

/// 文件过滤器
#[derive(Debug, Serialize, Deserialize)]
pub struct FileFilter {
    pub name: String,
    pub extensions: Vec<String>,
}
