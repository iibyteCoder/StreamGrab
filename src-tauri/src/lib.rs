//! StreamGrab - M3U8 视频流下载器
//!
//! 基于 Tauri 2.0 的现代视频流下载器 GUI 应用

mod commands;
mod db;
mod process;

use tauri::Manager;

use commands::{config::*, download::*, task::*, keys::*};
use db::Database;

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

            // 初始化统一数据库
            let database = Database::initialize(&config_dir)
                .expect("Failed to initialize database");
            app.manage(database);

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
            // 配置命令（SQLite）
            load_settings,
            save_setting,
            save_settings,
            reset_setting,
            reset_all_settings,
            export_config,
            import_config,
            get_db_path,
            open_in_explorer,
            file_exists,
            select_directory,
            select_file,
            // 历史记录命令
            load_history,
            add_history_record,
            clear_history,
            delete_history_record,
            // 任务命令
            load_all_tasks,
            load_recoverable_tasks,
            save_task,
            save_tasks,
            update_task_status,
            update_task_progress,
            delete_task,
            clear_finished_tasks,
            mark_active_tasks_interrupted,
            clear_all_tasks,
            // 密钥命令
            load_keys,
            add_key,
            update_key,
            delete_key,
            clear_keys,
            record_key_usage,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
