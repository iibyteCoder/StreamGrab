//! 系统命令
//!
//! 对话框、文件系统操作、应用更新下载与安装

use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;

// ========================================
// 对话框
// ========================================

/// 文件过滤器
#[derive(Debug, Serialize, Deserialize)]
pub struct FileFilter {
    pub name: String,
    pub extensions: Vec<String>,
}

/// 选择目录
#[tauri::command]
pub async fn select_directory(app: AppHandle) -> Result<Option<String>, String> {
    Ok(app
        .dialog()
        .file()
        .blocking_pick_folder()
        .map(|p| p.to_string()))
}

/// 选择文件
#[tauri::command]
pub async fn select_file(
    app: AppHandle,
    filters: Option<Vec<FileFilter>>,
) -> Result<Option<String>, String> {
    let mut dialog = app.dialog().file();
    if let Some(filter_list) = filters {
        for filter in filter_list {
            dialog = dialog.add_filter(
                filter.name,
                &filter
                    .extensions
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>(),
            );
        }
    }
    Ok(dialog.blocking_pick_file().map(|p| p.to_string()))
}

// ========================================
// 文件系统
// ========================================

/// 在文件管理器中打开路径
#[tauri::command]
pub async fn open_in_explorer(path: String) -> Result<(), String> {
    log::info!("Opening in explorer: {path}");

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("打开文件管理器失败: {e}"))?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("打开 Finder 失败: {e}"))?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("打开文件管理器失败: {e}"))?;
    }

    Ok(())
}

/// 打开文件所在目录并选中文件
#[tauri::command(rename_all = "camelCase")]
pub async fn open_file_in_explorer(file_path: String) -> Result<(), String> {
    let path = PathBuf::from(&file_path);
    if !path.exists() {
        return Err(format!("文件不存在: {file_path}"));
    }

    #[cfg(target_os = "windows")]
    {
        let select_arg = format!("/select,{}", path.to_string_lossy());
        std::process::Command::new("explorer")
            .arg(&select_arg)
            .spawn()
            .map_err(|e| format!("打开文件管理器失败: {e}"))?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .args(["-R", &path.to_string_lossy()])
            .spawn()
            .map_err(|e| format!("打开 Finder 失败: {e}"))?;
    }
    #[cfg(target_os = "linux")]
    {
        if let Some(parent) = path.parent() {
            std::process::Command::new("xdg-open")
                .arg(parent)
                .spawn()
                .map_err(|e| format!("打开文件管理器失败: {e}"))?;
        }
    }

    Ok(())
}

/// 使用系统默认程序打开文件（如播放视频、查看文档）
#[tauri::command]
pub async fn open_file_with_default(app: AppHandle, path: String) -> Result<(), String> {
    log::info!("Opening file with default app: {path}");

    if !PathBuf::from(&path).exists() {
        return Err(format!("文件不存在: {path}"));
    }

    app.opener()
        .open_path(path, None::<&str>)
        .map_err(|e| format!("打开文件失败: {e}"))
}

/// 检查文件是否存在
#[tauri::command]
pub async fn file_exists(path: String) -> Result<bool, String> {
    Ok(PathBuf::from(&path).exists())
}

/// 删除文件或文件夹（文件夹递归删除）
#[tauri::command]
pub async fn delete_file_or_folder(path: String) -> Result<(), String> {
    log::info!("Deleting: {path}");
    let path = PathBuf::from(&path);
    if !path.exists() {
        return Err("文件或文件夹不存在".to_string());
    }
    if path.is_file() {
        std::fs::remove_file(&path).map_err(|e| format!("删除文件失败: {e}"))?;
    } else if path.is_dir() {
        std::fs::remove_dir_all(&path).map_err(|e| format!("删除文件夹失败: {e}"))?;
    }
    Ok(())
}

/// 获取数据库文件路径
#[tauri::command]
pub async fn get_db_path(app: AppHandle) -> Result<String, String> {
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("获取配置目录失败: {e}"))?;
    Ok(config_dir
        .join("streamgrab.db")
        .to_string_lossy()
        .to_string())
}

// ========================================
// 应用更新
// ========================================

/// 应用下载进度
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppDownloadProgress {
    pub status: String,
    pub downloaded: u64,
    pub total: u64,
    pub percent: f64,
}

/// 下载应用更新安装包
#[tauri::command(rename_all = "camelCase")]
pub async fn download_app_update(
    download_url: String,
    save_path: String,
    app: AppHandle,
) -> Result<String, String> {
    let save_path = PathBuf::from(&save_path);
    log::info!("[Update] 开始下载应用更新到: {:?}", save_path);

    if let Some(parent) = save_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
        }
    }

    let _ = app.emit(
        "app:update:start",
        &serde_json::json!({ "url": &download_url }),
    );

    let client = reqwest::Client::builder()
        .user_agent("StreamGrab-Updater")
        .timeout(Duration::from_secs(600))
        .connect_timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {e}"))?;

    let response = client
        .get(&download_url)
        .send()
        .await
        .map_err(|e| format!("下载请求失败: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("下载失败: HTTP {}", response.status()));
    }

    let total_size = response.content_length().unwrap_or(0);
    let _ = app.emit(
        "app:update:progress",
        &AppDownloadProgress {
            status: "downloading".into(),
            downloaded: 0,
            total: total_size,
            percent: 0.0,
        },
    );

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("下载文件内容失败: {e}"))?;
    let actual_size = bytes.len() as u64;

    if total_size > 0 && actual_size != total_size {
        return Err(format!(
            "下载不完整: 期望 {total_size} bytes, 实际 {actual_size} bytes"
        ));
    }

    // SHA-256 完整性校验（尝试获取 .sha256 伴随文件）
    let filename = download_url.rsplit('/').next().unwrap_or("installer");
    crate::infrastructure::fs::verify_download_integrity(&client, &download_url, filename, &bytes)
        .await?;

    use std::io::Write;
    let mut file = std::fs::File::create(&save_path).map_err(|e| format!("创建文件失败: {e}"))?;
    file.write_all(&bytes)
        .map_err(|e| format!("写入文件失败: {e}"))?;
    file.sync_all().map_err(|e| format!("同步文件失败: {e}"))?;

    let _ = app.emit(
        "app:update:progress",
        &AppDownloadProgress {
            status: "downloaded".into(),
            downloaded: actual_size,
            total: total_size,
            percent: 100.0,
        },
    );
    let _ = app.emit(
        "app:update:complete",
        &serde_json::json!({ "path": save_path.to_string_lossy() }),
    );

    Ok(save_path.to_string_lossy().to_string())
}

/// 运行安装程序
#[tauri::command(rename_all = "camelCase")]
pub async fn run_installer(installer_path: String) -> Result<(), String> {
    let path = PathBuf::from(&installer_path);
    if !path.exists() {
        return Err(format!("安装程序不存在: {installer_path}"));
    }
    log::info!("[Update] 运行安装程序: {:?}", path);

    #[cfg(target_os = "windows")]
    {
        let path_str = path.to_string_lossy().to_string();
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &path_str])
            .spawn()
            .map_err(|e| format!("运行安装程序失败: {e}"))?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("打开安装程序失败: {e}"))?;
    }
    #[cfg(target_os = "linux")]
    {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))
                .map_err(|e| format!("设置执行权限失败: {e}"))?;
        }
        std::process::Command::new(&path)
            .spawn()
            .map_err(|e| format!("运行安装程序失败: {e}"))?;
    }

    Ok(())
}
