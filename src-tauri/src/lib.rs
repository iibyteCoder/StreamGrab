//! StreamGrab - M3U8 视频流下载器
//!
//! 基于 Tauri 2.0 的现代视频流下载器 GUI 应用

mod commands;
mod db;
mod process;
mod tray;
mod types;

use serde::Deserialize;
use tauri::Manager;

use commands::{config::*, dialog::*, download::*, fs::*, task::*};
use db::Database;

/// 通用设置结构（用于解析 minimizeToTray）
#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GeneralSettings {
    #[serde(default = "default_minimize_to_tray")]
    minimizeToTray: bool,
}

fn default_minimize_to_tray() -> bool {
    true
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            // 开发模式下启用日志
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Debug)
                        .build(),
                )?;
            }

            // 初始化配置目录
            let config_dir = app
                .path()
                .app_config_dir()
                .expect("Failed to get config directory");

            // 初始化统一数据库
            let database =
                Database::initialize(&config_dir).expect("Failed to initialize database");
            app.manage(database.clone());

            // 创建系统托盘
            let _tray = tray::create_tray(app.handle())
                .map_err(|e| log::error!("Failed to create tray: {}", e));

            // 监听窗口关闭事件，根据设置决定是最小化到托盘还是退出
            if let Some(window) = app.get_webview_window("main") {
                let app_handle = app.handle().clone();
                let db_for_close = database.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        // 从数据库读取 minimizeToTray 设置
                        let should_minimize = db_for_close
                            .settings
                            .load("general")
                            .ok()
                            .and_then(|value| serde_json::from_value::<GeneralSettings>(value).ok())
                            .map(|s| s.minimizeToTray)
                            .unwrap_or(true);

                        if should_minimize {
                            // 最小化到托盘
                            api.prevent_close();
                            if let Some(win) = app_handle.get_webview_window("main") {
                                let _ = win.hide();
                            }
                        }
                        // 如果 minimizeToTray 为 false，允许窗口正常关闭（退出应用）
                    }
                });
            }

            log::info!("StreamGrab starting...");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // === 下载命令 ===
            start_download,
            stop_download,
            pause_download,
            resume_download,
            parse_url,
            get_n_m3u8dl_version,
            get_file_info,
            detect_url_type,
            start_http_video_download,
            // === 配置命令 ===
            load_settings,
            save_setting,
            save_settings,
            reset_setting,
            reset_all_settings,
            export_config,
            import_config,
            // === 文件系统命令 ===
            get_db_path,
            open_in_explorer,
            file_exists,
            delete_file_or_folder,
            // === 对话框命令 ===
            select_directory,
            select_file,
            // === 任务命令 ===
            load_all_tasks,
            load_recoverable_tasks,
            create_task,
            update_task_status,
            update_task_output_path,
            update_task_progress,
            update_task_media_info,
            delete_task,
            clear_finished_tasks,
            mark_active_tasks_interrupted,
            clear_all_tasks,
            get_progress_history,
            clear_progress_history,
            save_progress_history,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
