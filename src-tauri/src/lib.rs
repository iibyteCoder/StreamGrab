//! StreamGrab - M3U8 视频流下载器
//!
//! 基于 Tauri 2.0 的现代视频流下载器 GUI 应用

mod commands;
mod process;

use tauri::Manager;

use commands::{config::*, download::*};

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
            if let Some(config_dir) = app.path().app_config_dir().ok() {
                std::fs::create_dir_all(&config_dir).ok();
            }

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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
