//! 应用更新命令
//!
//! 提供应用更新的下载功能

use std::path::PathBuf;
use std::time::Duration;

use log::info;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

/// 应用下载进度
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppDownloadProgress {
    pub status: String,
    pub downloaded: u64,
    pub total: u64,
    pub percent: f64,
}

/// 下载应用更新
///
/// @param download_url 下载链接
/// @param save_path 保存路径
/// @return 保存的文件路径
#[tauri::command(rename_all = "camelCase")]
pub async fn download_app_update(
    download_url: String,
    save_path: String,
    app: AppHandle,
) -> Result<String, String> {
    let save_path = PathBuf::from(&save_path);

    info!("[Update] 开始下载应用更新到: {:?}", save_path);

    // 确保目标目录存在
    if let Some(parent) = save_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
        }
    }

    // 发送开始事件
    let _ = app.emit(
        "app:update:start",
        &serde_json::json!({ "url": &download_url }),
    );

    // 创建 HTTP 客户端
    let client = reqwest::Client::builder()
        .user_agent("StreamGrab-Updater")
        .timeout(Duration::from_secs(600)) // 10 分钟超时
        .connect_timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    info!("[Update] 开始下载文件: {}", download_url);

    // 下载
    let response = client
        .get(&download_url)
        .send()
        .await
        .map_err(|e| format!("下载请求失败: {}", e))?;

    info!("[Update] 下载响应状态: {}", response.status());

    if !response.status().is_success() {
        return Err(format!("下载失败: HTTP {}", response.status()));
    }

    // 获取响应内容长度
    let total_size = response.content_length().unwrap_or(0);

    info!("[Update] 文件大小: {} bytes", total_size);

    // 发送下载中进度事件
    let _ = app.emit(
        "app:update:progress",
        &AppDownloadProgress {
            status: "downloading".to_string(),
            downloaded: 0,
            total: total_size,
            percent: 0.0,
        },
    );

    // 下载整个文件到内存
    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("下载文件内容失败: {}", e))?;

    let actual_size = bytes.len() as u64;
    info!("[Update] 实际下载大小: {} bytes", actual_size);

    // 验证下载完整性
    if total_size > 0 && actual_size != total_size {
        return Err(format!(
            "下载不完整: 期望 {} bytes, 实际 {} bytes",
            total_size, actual_size
        ));
    }

    // 写入文件
    use std::io::Write;
    let mut file = std::fs::File::create(&save_path).map_err(|e| format!("创建文件失败: {}", e))?;
    file.write_all(&bytes)
        .map_err(|e| format!("写入文件失败: {}", e))?;
    file.sync_all()
        .map_err(|e| format!("同步文件失败: {}", e))?;

    info!("[Update] 下载完成，大小: {} bytes", actual_size);

    // 发送完成事件
    let _ = app.emit(
        "app:update:progress",
        &AppDownloadProgress {
            status: "downloaded".to_string(),
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

/// 打开文件所在目录并选中文件
#[tauri::command(rename_all = "camelCase")]
pub async fn open_file_in_explorer(file_path: String) -> Result<(), String> {
    let path = PathBuf::from(&file_path);

    if !path.exists() {
        return Err(format!("文件不存在: {}", file_path));
    }

    info!("[Update] 打开文件管理器: {:?}", path);

    #[cfg(target_os = "windows")]
    {
        // Windows: 使用 explorer 选中文件，注意参数格式
        let path_str = path.to_string_lossy();
        let select_arg = format!("/select,{}", path_str);
        std::process::Command::new("explorer")
            .arg(&select_arg)
            .spawn()
            .map_err(|e| format!("打开文件管理器失败: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        // macOS: 使用 open 命令
        std::process::Command::new("open")
            .args(["-R", &path.to_string_lossy()])
            .spawn()
            .map_err(|e| format!("打开 Finder 失败: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        // Linux: 尝试使用 dbus 打开文件管理器
        if let Some(parent) = path.parent() {
            std::process::Command::new("xdg-open")
                .arg(parent)
                .spawn()
                .map_err(|e| format!("打开文件管理器失败: {}", e))?;
        }
    }

    Ok(())
}

/// 运行安装程序
#[tauri::command(rename_all = "camelCase")]
pub async fn run_installer(installer_path: String) -> Result<(), String> {
    let path = PathBuf::from(&installer_path);

    if !path.exists() {
        return Err(format!("安装程序不存在: {}", installer_path));
    }

    info!("[Update] 运行安装程序: {:?}", path);

    #[cfg(target_os = "windows")]
    {
        // Windows: 使用 cmd 启动安装程序（更好的兼容性）
        let path_str = path.to_string_lossy().to_string();
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &path_str])
            .spawn()
            .map_err(|e| format!("运行安装程序失败: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        // macOS: 使用 open 命令打开 dmg
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("打开安装程序失败: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        // Linux: 添加执行权限并运行
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))
                .map_err(|e| format!("设置执行权限失败: {}", e))?;
        }

        std::process::Command::new(&path)
            .spawn()
            .map_err(|e| format!("运行安装程序失败: {}", e))?;
    }

    Ok(())
}
