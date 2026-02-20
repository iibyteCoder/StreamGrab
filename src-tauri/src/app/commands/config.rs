//! 配置相关命令
//!
//! 处理应用配置的读取、保存、导入、导出

use std::fs;

use tauri::AppHandle;

use super::utils::get_db;

/// 加载所有配置
#[tauri::command]
pub async fn load_settings(
    app: AppHandle,
) -> Result<std::collections::HashMap<String, serde_json::Value>, String> {
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

/// 导出配置到指定路径
#[tauri::command]
pub async fn export_config(file_path: String, app: AppHandle) -> Result<(), String> {
    log::info!("Exporting config to: {}", file_path);

    let db = get_db(&app)?;
    let settings = db.settings.load_all()?;

    let content = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(&file_path, content).map_err(|e| format!("Failed to export config: {}", e))?;

    Ok(())
}

/// 从指定路径导入配置
#[tauri::command]
pub async fn import_config(file_path: String, app: AppHandle) -> Result<(), String> {
    log::info!("Importing config from: {}", file_path);

    let content =
        fs::read_to_string(&file_path).map_err(|e| format!("Failed to read config file: {}", e))?;

    let settings: std::collections::HashMap<String, serde_json::Value> =
        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;

    let db = get_db(&app)?;
    db.settings.save_all(&settings)
}
