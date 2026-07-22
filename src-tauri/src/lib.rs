//! StreamGrab - M3U8 视频流下载器
//!
//! 基于 Tauri 2.0 的现代视频流下载器 GUI 应用

// 模块声明 - 分层架构
pub mod app;
pub mod domain;
pub mod infrastructure;
pub mod shared;

// 常用类型重新导出
pub use infrastructure::{Database, DbProgressRepository};
pub use shared::{AppError, AppResult};

use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

use app::commands::{
    download::*, history::*, presets::*, settings::*, system::*, tasks::*, tools::*,
};
use infrastructure::process::manager::ProcessManager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            // 开发模式下启用日志
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Debug)
                        .build(),
                )?;
            }

            // 初始化配置目录与统一数据库（schema v4）
            let config_dir = app
                .path()
                .app_config_dir()
                .map_err(|e| format!("获取配置目录失败: {e}"))?;
            let database =
                Database::initialize(&config_dir).map_err(|e| format!("初始化数据库失败: {e}"))?;

            // 进度跟踪器：领域层采样缓冲 → 数据库持久化（观察者模式的持久化端）
            let progress_repo = Arc::new(DbProgressRepository::new(database.connection()));
            domain::download::init_progress_tracker(progress_repo);
            log::info!("Progress tracker initialized");

            app.manage(database.clone());

            // 进程管理器（State 注入，替代全局 static；退出清理由 Drop + Exit hook 双保险）
            app.manage(Arc::new(Mutex::new(ProcessManager::new())));

            // 下载引擎注册表（策略模式：按 URL 类型分派）
            app.manage(infrastructure::engines::default_registry());

            // 创建系统托盘
            let _tray = crate::app::create_tray(app.handle())
                .map_err(|e| log::error!("Failed to create tray: {e}"));

            // 监听窗口关闭事件：按 minimize_to_tray 设置决定行为
            if let Some(window) = app.get_webview_window("main") {
                let app_handle = app.handle().clone();
                let db_for_close = database.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        let should_minimize = db_for_close
                            .settings
                            .load_app_settings()
                            .map(|s| s.minimize_to_tray)
                            .unwrap_or(true);

                        if should_minimize {
                            api.prevent_close();
                            if let Some(win) = app_handle.get_webview_window("main") {
                                let _ = win.hide();
                            }
                        }
                    }
                });
            }

            log::info!("StreamGrab starting...");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // === 任务 ===
            load_all_tasks,
            load_recoverable_tasks,
            get_task,
            create_task,
            update_task_status,
            update_task_output_path,
            update_task_progress,
            update_task_media_info,
            save_task_overrides,
            delete_task,
            clear_finished_tasks,
            clear_all_tasks,
            mark_active_tasks_interrupted,
            get_progress_history,
            clear_progress_history,
            // === 下载（引擎自动分派）===
            start_download,
            stop_download,
            pause_download,
            resume_download,
            parse_url,
            detect_url_type,
            get_file_info,
            analyze_media_file,
            // === 设置（应用 + 按工具分离）===
            get_app_settings,
            save_app_settings,
            patch_app_settings,
            get_tool_settings,
            save_tool_settings,
            patch_tool_settings,
            export_config,
            import_config,
            // === 任务预设 ===
            load_presets,
            save_preset,
            delete_preset,
            // === 历史记录 ===
            load_history,
            delete_history_record,
            clear_history,
            // === 工具管理 ===
            get_nm3u8dl_info,
            get_ffmpeg_info,
            get_ffprobe_info,
            get_nm3u8dl_latest_release,
            get_ffmpeg_latest_release,
            download_tool,
            // === 系统（对话框/文件系统/应用更新）===
            select_directory,
            select_file,
            open_in_explorer,
            open_file_in_explorer,
            file_exists,
            delete_file_or_folder,
            get_db_path,
            download_app_update,
            run_installer,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app_handle, event| {
        // 应用退出：同步清理所有子进程（Drop 之外的第二道保险）
        if let tauri::RunEvent::Exit = event {
            if let Some(manager) = app_handle.try_state::<Arc<Mutex<ProcessManager>>>() {
                if let Ok(mut guard) = manager.try_lock() {
                    guard.stop_all_sync();
                }
            }
        }
    });
}
