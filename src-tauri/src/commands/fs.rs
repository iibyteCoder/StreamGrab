//! 文件系统相关命令
//!
//! 处理文件和目录的操作

use std::fs;
use std::path::PathBuf;

use tauri::Manager;

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
        fs::remove_file(&path).map_err(|e| format!("删除文件失败: {}", e))?;
    } else if path.is_dir() {
        fs::remove_dir_all(&path).map_err(|e| format!("删除文件夹失败: {}", e))?;
    }

    log::info!("Successfully deleted: {:?}", path);
    Ok(())
}

/// 获取数据库文件路径
#[tauri::command]
pub async fn get_db_path(app: tauri::AppHandle) -> Result<String, String> {
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("Failed to get config directory: {}", e))?;

    Ok(config_dir
        .join("streamgrab.db")
        .to_string_lossy()
        .to_string())
}
