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
use tauri::{Emitter, Manager};
use tokio::sync::Mutex;

use app::commands::{
    download::*, history::*, presets::*, settings::*, system::*, tasks::*, tools::*,
};
use infrastructure::process::manager::ProcessManager;

/// 应用日志级别设置 → `log::LevelFilter`（驱动 tauri_plugin_log）
fn level_filter(level: crate::domain::config::LogLevel) -> log::LevelFilter {
    use crate::domain::config::LogLevel;
    match level {
        LogLevel::Debug => log::LevelFilter::Debug,
        LogLevel::Info => log::LevelFilter::Info,
        LogLevel::Warn => log::LevelFilter::Warn,
        LogLevel::Error => log::LevelFilter::Error,
        LogLevel::Off => log::LevelFilter::Off,
    }
}

/// 生产入口
///
/// 构建完整应用（真实 assets + 配置窗口）并运行事件循环。
/// 测试用 `build_app()` + `mock_context(noop_assets())` 构造 MockRuntime 实例。
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = build_app::<tauri::Wry>()
        .build(tauri::generate_context!())
        .expect("error while building tauri application");
    app.run(run_event_loop);
}

/// 可复用应用构建器
///
/// 生产 `run()` 用 `generate_context!()`；集成测试用 `mock_context(noop_assets())`。
/// 插桩、命令注册与 setup 均在此，保证两条路径行为一致。
pub fn build_app<R: tauri::Runtime>() -> tauri::Builder<R> {
    tauri::Builder::<R>::new()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_notification::init())
        .setup(setup_app)
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
            open_file_with_default,
            file_exists,
            delete_file_or_folder,
            get_db_path,
            get_tray_status,
            download_app_update,
            run_installer,
        ])
}

/// 应用初始化：DB + 日志插件 + 状态注入 + 托盘 + 关闭拦截注册
///
/// `create_tray` 在 MockRuntime 下无真实托盘实现，测试应改用
/// `setup_app_with_db(app, db)`（跳过托盘与日志插件，注入内存 DB）。
fn setup_app<R: tauri::Runtime>(app: &mut tauri::App<R>) -> Result<(), Box<dyn std::error::Error>> {
    // 初始化配置目录与统一数据库（schema v4）
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("获取配置目录失败: {e}"))?;
    let database =
        Database::initialize(&config_dir).map_err(|e| format!("初始化数据库失败: {e}"))?;

    setup_app_with_db(app, database, true)
}

/// 应用初始化（可注入数据库，供测试复用；`create_tray` 控制是否创建真实托盘）
///
/// - `create_tray = true`：生产路径，创建系统托盘（失败记录 TrayStatus 供前端提示）。
/// - `create_tray = false`：测试路径（MockRuntime 无真实托盘），仅记录状态。
pub fn setup_app_with_db<R: tauri::Runtime>(
    app: &mut tauri::App<R>,
    database: Database,
    create_tray: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // 应用自身日志级别由设置驱动（log_level / log_file_path / no_log），
    // 与 N_m3u8DL-RE 子进程日志（经 --log-level 传入）分开配置。
    // 读库失败回退 Info（与默认设置一致）。
    let (app_log_level, app_log_file_path) = database
        .settings
        .load_app_settings()
        .map(|s| (s.log_level, s.log_file_path))
        .unwrap_or((crate::domain::config::LogLevel::Info, String::new()));
    let mut log_builder = tauri_plugin_log::Builder::default().level(level_filter(app_log_level));
    // 配置了日志文件路径且父目录存在时，追加文件输出（尽力而为）
    let log_file = std::path::Path::new(&app_log_file_path);
    if !app_log_file_path.is_empty() && log_file.parent().is_some_and(|p| p.exists()) {
        let file_name = log_file
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "streamgrab.log".into());
        log_builder = log_builder.targets([
            tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
            tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Folder {
                path: log_file.parent().expect("parent 已校验存在").to_path_buf(),
                file_name: Some(file_name),
            }),
        ]);
    }
    app.handle().plugin(log_builder.build())?;

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
    // 托盘创建失败不阻塞启动，但记录状态供前端提示——
    // 否则「关闭时最小化到托盘」会把窗口隐藏，却无图标可恢复，应用像「消失」一样。
    let tray_status = if create_tray {
        match crate::app::create_tray(app.handle()) {
            Ok(tray) => {
                // 显式持活 TrayIcon（官方推荐模式）：即使资源表已持有克隆，
                // 也避免任何平台差异导致托盘图标被回收
                app.manage(tray);
                crate::app::TrayStatus {
                    created: true,
                    error: None,
                }
            }
            Err(e) => {
                log::warn!("Failed to create tray: {e}");
                crate::app::TrayStatus {
                    created: false,
                    error: Some(e.to_string()),
                }
            }
        }
    } else {
        crate::app::TrayStatus {
            created: false,
            error: Some("测试环境不创建真实托盘".into()),
        }
    };
    app.manage(tray_status);

    // 监听窗口关闭事件：按 minimize_to_tray 设置决定行为
    register_close_handler(app, &database);

    log::info!("StreamGrab starting...");

    Ok(())
}

/// 注册窗口关闭拦截：`minimize_to_tray` 为 true 时隐藏到托盘而非退出
///
/// MockRuntime 下 `on_window_event` 回调为空操作（tauri `test` 模块实现），
/// 端到端关闭行为回归见 `tests/close_behavior.rs`。
fn register_close_handler<R: tauri::Runtime>(app: &tauri::App<R>, database: &Database) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    let app_handle = app.handle().clone();
    let db_for_close = database.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            // 每次关闭实时读 DB（单行索引读，亚毫秒）；读库失败回退隐藏（防误退出）。
            // 决策委托 resolve_close_behavior（domain 层纯函数，单测锁定）。
            let behavior = db_for_close
                .settings
                .load_app_settings()
                .map(|s| crate::domain::config::resolve_close_behavior(&s))
                .unwrap_or_else(|e| {
                    log::warn!("CloseRequested: 读取 minimize_to_tray 失败，回退隐藏: {e}");
                    crate::domain::config::CloseBehavior::Minimize
                });
            log::info!("CloseRequested: behavior={:?}", behavior);

            if behavior == crate::domain::config::CloseBehavior::Minimize {
                // 安全网：托盘不可用时绝不隐藏窗口——否则窗口消失且无图标可恢复，
                // 应用像「消失」一样。此时改为正常退出。
                let tray_created = app_handle
                    .try_state::<crate::app::TrayStatus>()
                    .map(|s| s.created)
                    .unwrap_or(false);
                if !tray_created {
                    log::warn!("CloseRequested: minimize_to_tray 开启但托盘不可用，改为正常退出");
                    return;
                }
                api.prevent_close();
                if let Some(win) = app_handle.get_webview_window("main") {
                    match win.hide() {
                        Ok(()) => {
                            log::info!("CloseRequested: 窗口已隐藏到托盘");
                            // 通知前端展示「已最小化到托盘」，避免用户误以为应用退出
                            let _ = app_handle.emit("app:minimized-to-tray", ());
                        }
                        Err(e) => log::warn!("CloseRequested: 隐藏窗口失败: {e}"),
                    }
                }
            }
        }
    });
}

/// 应用事件循环回调
///
/// - `RunEvent::Exit`：同步清理所有子进程（Drop 之外的第二道保险）。
pub fn run_event_loop<R: tauri::Runtime>(app: &tauri::AppHandle<R>, event: tauri::RunEvent) {
    if let tauri::RunEvent::Exit = event {
        if let Some(manager) = app.try_state::<Arc<Mutex<ProcessManager>>>() {
            if let Ok(mut guard) = manager.try_lock() {
                guard.stop_all_sync();
            }
        }
    }
}
