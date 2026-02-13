//! StreamGrab - M3U8 视频流下载器
//!
//! 基于 Tauri 2.0 的现代视频流下载器 GUI 应用

mod commands;
mod db;
mod process;

use std::sync::Arc;
use tauri::Manager;

use commands::{config::*, download::*};
use db::HistoryDb;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
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
            std::fs::create_dir_all(&config_dir).expect("Failed to create config directory");

            // 初始化历史记录数据库
            let history_db = Arc::new(
                HistoryDb::new(&config_dir).expect("Failed to initialize history database"),
            );
            app.manage(history_db);

            log::info!("StreamGrab starting...");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 下载命令
            start_download,
            stop_download,
            pause_download,
            resume_download,
            parse_url,
            get_n_m3u8dl_version,
            // 配置命令
            load_config,
            save_config,
            export_config,
            import_config,
            get_config_path_cmd,
            open_in_explorer,
            file_exists,
            select_directory,
            select_file,
            // 历史记录命令
            load_history,
            save_history,
            add_history_record,
            clear_history,
            delete_history_record,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
