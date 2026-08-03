//! 下载命令
//!
//! 引擎分派（策略模式）：URL 类型 → 引擎 → 构建参数 → 启动进程 → 事件推送。
//! 前端不再构建任何 CLI 参数——工具知识全部内聚于后端引擎模块。

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::Mutex as TokioMutex;

use super::api;
use crate::domain::config::{FfmpegConfig, ToolConfigs};
use crate::domain::download::{
    flush_progress, record_progress, EngineEvent, EngineRegistry, EngineSession, StreamInfo,
    ToolId, UrlType,
};
use crate::domain::media::{MediaAnalyzer, MediaInfo};
use crate::infrastructure::fs as fs_util;
use crate::infrastructure::media::ffprobe;
use crate::infrastructure::process::manager::ProcessManager;
use crate::infrastructure::tools::{
    get_downloader_exe_path, get_ffmpeg_exe_path, get_ffprobe_exe_path,
};
use crate::infrastructure::Database;
use crate::shared::{AppError, AppResult, ResolvedPath};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
use crate::infrastructure::platform::CREATE_NO_WINDOW;

/// 空字符串视为未配置
fn none_if_empty(s: Option<&str>) -> Option<&str> {
    s.filter(|s| !s.is_empty())
}

/// 解析 ffprobe 二进制路径（ffprobe_path 优先，回退到 ffmpeg 目录）
fn resolve_ffprobe_bin(ffmpeg: &FfmpegConfig) -> Option<PathBuf> {
    get_ffprobe_exe_path(none_if_empty(Some(ffmpeg.ffprobe_path.as_str())))
        .or_else(|| get_ffprobe_exe_path(none_if_empty(Some(ffmpeg.ffmpeg_path.as_str()))))
}

/// 解析默认保存目录（用户既无任务覆盖也无全局默认时使用），并确保目录存在。
///
/// 优先系统「下载」目录下的 `StreamGrab` 子目录，失败则回退应用数据目录。
/// 返回绝对路径，避免 N_m3u8DL-RE 以自身 CWD 解释空/相对路径而把文件落到工程目录。
fn default_save_dir(app: &AppHandle) -> AppResult<String> {
    let dir = app
        .path()
        .download_dir()
        .map(|d| d.join("StreamGrab"))
        .or_else(|_| app.path().app_data_dir().map(|d| d.join("downloads")))
        .map_err(|e| AppError::other(format!("获取默认保存目录失败: {e}")))?;
    std::fs::create_dir_all(&dir)
        .map_err(|e| AppError::other(format!("创建默认保存目录失败: {e}")))?;
    log::info!("未配置保存目录，回退默认: {}", dir.display());
    Ok(dir.to_string_lossy().into_owned())
}

/// 加载全部工具配置
fn load_tool_configs(db: &Database) -> AppResult<ToolConfigs> {
    Ok(ToolConfigs {
        nm3u8dl: db.settings.load_tool_config(ToolId::Nm3u8dl)?,
        ffmpeg: db.settings.load_tool_config(ToolId::Ffmpeg)?,
    })
}

/// 同步执行命令并捕获输出（解析模式使用）
///
/// `working_dir`：子进程 CWD。解析模式下 N_m3u8DL-RE 即使 `--skip-download`
/// 也会在 CWD 创建元数据目录（raw.m3u8 / meta.json），必须指向系统临时目录
/// 而非工程目录，避免开发时污染 src-tauri/。
fn run_command_capture(
    program: &ResolvedPath,
    args: &[String],
    working_dir: Option<&std::path::Path>,
) -> AppResult<std::process::Output> {
    let mut cmd = std::process::Command::new(program.as_path());
    cmd.args(args);
    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd.output()
        .map_err(|e| AppError::process(format!("执行命令失败: {e}")))
}

/// 开始下载（引擎自动分派）
#[tauri::command(rename_all = "camelCase")]
pub async fn start_download(
    task_id: String,
    db: State<'_, Database>,
    manager: State<'_, Arc<TokioMutex<ProcessManager>>>,
    engines: State<'_, EngineRegistry>,
    app: AppHandle,
) -> Result<(), String> {
    api(start_download_inner(task_id, &db, &manager, &engines, app).await)
}

async fn start_download_inner(
    task_id: String,
    db: &Database,
    manager: &Arc<TokioMutex<ProcessManager>>,
    engines: &EngineRegistry,
    app: AppHandle,
) -> AppResult<()> {
    let task = db
        .tasks
        .get(&task_id)?
        .ok_or_else(|| AppError::other(format!("任务不存在: {task_id}")))?;
    let mut spec = task.spec();

    let app_settings = db.settings.load_app_settings()?;
    let tools = load_tool_configs(db)?;

    // 保存目录解析：任务覆盖 > 全局默认 > 系统下载目录。
    // 空目录会让 N_m3u8DL-RE 落到自身 CWD，且完成回调 find_output_file 找不到输出文件。
    if spec.save_dir.trim().is_empty() {
        spec.save_dir = app_settings.default_save_dir.clone();
    }
    if spec.save_dir.trim().is_empty() {
        spec.save_dir = default_save_dir(&app)?;
    }

    let engine = engines
        .for_url(spec.url_type)
        .ok_or_else(|| AppError::other("无可用下载引擎"))?;

    // 解析工具二进制 + 引擎构建参数（ResolvedPath 保证非空+绝对+存在）
    let program_path = match engine.id() {
        ToolId::Nm3u8dl => get_downloader_exe_path(none_if_empty(Some(
            tools.nm3u8dl.path.as_str(),
        )))
        .ok_or_else(|| {
            AppError::tool_not_found(
                "N_m3u8DL-RE 未找到。请在设置中配置工具目录路径，或使用【下载】按钮自动下载。",
            )
        })?,
        ToolId::Ffmpeg => get_ffmpeg_exe_path(none_if_empty(Some(
            tools.ffmpeg.ffmpeg_path.as_str(),
        )))
        .ok_or_else(|| {
            AppError::tool_not_found(
                "FFmpeg 未找到。请在设置中配置工具目录路径，或使用【下载】按钮自动下载。",
            )
        })?,
    };
    let program = ResolvedPath::from_path(program_path)?;
    let args = engine.build_download_args(&spec, &tools, &app_settings);

    log::info!(
        "Starting download: task_id={}, engine={}",
        task_id,
        engine.id()
    );

    // 逐任务解析会话（跨行状态）
    let session: Arc<Mutex<Box<dyn EngineSession>>> = Arc::new(Mutex::new(engine.new_session()));

    // 事件分派：Tauri 事件推送 + 进度采样持久化（on_output 与 on_complete 共用）
    let task_id_out = task_id.clone();
    let app_out = app.clone();
    let dispatch: Arc<dyn Fn(EngineEvent) + Send + Sync> = Arc::new(move |event| match event {
        EngineEvent::Log { level, message } => {
            let _ = app_out.emit(
                &format!("download:log:{task_id_out}"),
                serde_json::json!({ "level": level, "message": message }),
            );
        }
        EngineEvent::Progress { data } => {
            record_progress(
                &task_id_out,
                data.overall_percent,
                data.speed,
                data.downloaded_size,
            );
            let _ = app_out.emit(&format!("download:progress:{task_id_out}"), &data);
        }
        EngineEvent::Status { action } => {
            let _ = app_out.emit(
                &format!("download:status:{task_id_out}"),
                serde_json::json!({ "action": action }),
            );
        }
    });

    // 输出回调：会话解析 → 分派
    let dispatch_out = Arc::clone(&dispatch);
    let session_done = Arc::clone(&session);
    let on_output = move |line: String| {
        let events = session
            .lock()
            .ok()
            .map(|mut s| s.parse_chunk(&line))
            .unwrap_or_default();
        for event in events {
            dispatch_out(event);
        }
    };

    // 完成回调：冲刷会话残余 → 刷新进度历史 → 查找输出文件 → 事件通知。
    // 排序依赖 ProcessManager 的保证：本回调在输出读取线程 EOF 排空之后触发。
    // N_m3u8DL-RE 非 TTY 下将全部进度帧积压到退出瞬间一次性输出（无换行粘连块），
    // 必须由 finalize 兜底解析——否则完成事件抢跑、前端订阅读者注销后进度数据全部丢失。
    let task_id_done = task_id.clone();
    let app_done = app.clone();
    let save_dir_done = spec.save_dir.clone();
    let save_name_done = spec.file_name.clone();
    let engine_id = engine.id();
    let mux_format = spec.overrides.mux_format.unwrap_or(tools.ffmpeg.mux_format);
    let on_complete = move |success: bool, error_msg: Option<String>| {
        // 冲刷会话缓冲中无 `\n` 结尾的残余（退出倾泻的进度块）
        let residual = session_done
            .lock()
            .ok()
            .map(|mut s| s.finalize())
            .unwrap_or_default();
        for event in residual {
            dispatch(event);
        }
        // finalize 的进度点已入采样缓冲，此时刷新才能落库
        flush_progress(&task_id_done);
        if success {
            let output_path = match engine_id {
                // 直链下载输出路径确定
                ToolId::Ffmpeg => Some(
                    PathBuf::from(&save_dir_done)
                        .join(&save_name_done)
                        .to_string_lossy()
                        .to_string(),
                ),
                // 流媒体下载：下载器可能改动文件名，需要查找
                ToolId::Nm3u8dl => fs_util::find_output_file(
                    &save_dir_done,
                    Some(&save_name_done),
                    Some(mux_format),
                ),
            };
            let _ = app_done.emit(
                &format!("download:complete:{task_id_done}"),
                serde_json::json!({ "outputPath": output_path }),
            );
        } else {
            let _ = app_done.emit(
                &format!("download:error:{task_id_done}"),
                serde_json::json!({
                    "message": error_msg.unwrap_or_else(|| "未知错误".to_string())
                }),
            );
        }
    };

    // 确保保存目录存在（用户可能配置了尚未创建的路径）
    let save_dir_path = PathBuf::from(&spec.save_dir);
    if !save_dir_path.exists() {
        std::fs::create_dir_all(&save_dir_path).map_err(|e| {
            AppError::config(format!(
                "保存目录不存在且无法创建: {} ({e})",
                save_dir_path.display()
            ))
        })?;
        log::info!("已创建保存目录: {}", save_dir_path.display());
    }
    // ResolvedPath 编译期保证：非空 + 绝对 + 存在
    let save_dir_resolved = ResolvedPath::new(&spec.save_dir)?;

    manager.lock().await.start_process(
        task_id,
        &program,
        args,
        Some(&save_dir_resolved),
        on_output,
        on_complete,
    )
}

/// 停止下载
#[tauri::command(rename_all = "camelCase")]
pub async fn stop_download(
    task_id: String,
    manager: State<'_, Arc<TokioMutex<ProcessManager>>>,
) -> Result<(), String> {
    log::info!("Stopping download: {task_id}");
    manager.lock().await.stop_process(&task_id);
    Ok(())
}

/// 暂停下载
///
/// N_m3u8DL-RE 不支持真暂停：实现为终止进程，前端保留任务状态，
/// 恢复时经 `resume_download` 重新启动。
#[tauri::command(rename_all = "camelCase")]
pub async fn pause_download(
    task_id: String,
    manager: State<'_, Arc<TokioMutex<ProcessManager>>>,
) -> Result<(), String> {
    log::info!("Pausing download: {task_id}");
    manager.lock().await.stop_process(&task_id);
    Ok(())
}

/// 恢复下载（任务级覆盖已持久化，引擎重新构建参数）
#[tauri::command(rename_all = "camelCase")]
pub async fn resume_download(
    task_id: String,
    db: State<'_, Database>,
    manager: State<'_, Arc<TokioMutex<ProcessManager>>>,
    engines: State<'_, EngineRegistry>,
    app: AppHandle,
) -> Result<(), String> {
    log::info!("Resuming download: {task_id}");
    api(start_download_inner(task_id, &db, &manager, &engines, app).await)
}

/// 解析 URL 获取流信息（引擎自动分派：流媒体走 RE，直链走 ffprobe）
#[tauri::command(rename_all = "camelCase")]
pub async fn parse_url(
    url: String,
    db: State<'_, Database>,
    engines: State<'_, EngineRegistry>,
) -> Result<StreamInfo, String> {
    api(parse_url_inner(&db, &engines, &url).await)
}

async fn parse_url_inner(
    db: &Database,
    engines: &EngineRegistry,
    url: &str,
) -> AppResult<StreamInfo> {
    let url_type = UrlType::detect(url);
    log::info!("Parsing URL: {url} (type: {url_type:?})");

    let app_settings = db.settings.load_app_settings()?;
    let tools = load_tool_configs(db)?;
    let engine = engines
        .for_url(url_type)
        .ok_or_else(|| AppError::other("无可用下载引擎"))?;

    match engine.id() {
        ToolId::Nm3u8dl => {
            let program_path =
                get_downloader_exe_path(none_if_empty(Some(tools.nm3u8dl.path.as_str())))
                    .ok_or_else(|| {
                        AppError::tool_not_found(
                    "N_m3u8DL-RE 未找到。请在设置中配置工具目录路径，或使用【下载】按钮自动下载。",
                )
                    })?;
            let program = ResolvedPath::from_path(program_path)?;
            let args = engine.build_parse_args(url, &tools, &app_settings);

            // 解析模式下 N_m3u8DL-RE 会在 CWD 创建元数据目录，
            // 必须指向系统临时目录，避免污染工程目录（开发时 CWD = src-tauri/）
            let parse_cwd = std::env::temp_dir();
            log::info!(
                "Running parse command: {program} {args:?} (cwd: {})",
                parse_cwd.display()
            );
            let output = tokio::task::spawn_blocking(move || {
                run_command_capture(&program, &args, Some(&parse_cwd))
            })
            .await
            .map_err(|e| AppError::process(format!("解析任务中断: {e}")))??;

            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            if !output.status.success() {
                return Err(AppError::process(format!(
                    "解析失败: {}",
                    if stderr.is_empty() { &stdout } else { &stderr }
                )));
            }

            let info = engine.parse_streams(&stdout);
            log::info!(
                "Parsed {} videos, {} audios",
                info.videos.len(),
                info.audios.len()
            );
            Ok(info)
        }
        ToolId::Ffmpeg => {
            let bin = resolve_ffprobe_bin(&tools.ffmpeg)
                .ok_or_else(|| {
                    AppError::tool_not_found(
                        "FFprobe 未找到。请在设置中配置 FFmpeg 目录路径，或使用【下载】按钮自动下载。",
                    )
                })?
                .to_string_lossy()
                .to_string();
            let url_owned = url.to_string();
            let json = tokio::task::spawn_blocking(move || ffprobe::run_ffprobe(&bin, &url_owned))
                .await
                .map_err(|e| AppError::process(format!("探测任务中断: {e}")))?;
            ffprobe::stream_info_from_json(&json?)
        }
    }
}

/// 检测 URL 类型（前端类型徽章与引擎分派提示）
#[tauri::command(rename_all = "camelCase")]
pub async fn detect_url_type(url: String) -> Result<String, String> {
    let url_type = UrlType::detect(&url);
    Ok(serde_json::to_value(url_type)
        .ok()
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(|| "unknown".into()))
}

/// 获取文件信息
#[tauri::command(rename_all = "camelCase")]
pub async fn get_file_info(path: String) -> Result<fs_util::FileInfo, String> {
    api(fs_util::file_info(&path))
}

/// 分析媒体文件（ffprobe）
#[tauri::command(rename_all = "camelCase")]
pub async fn analyze_media_file(
    file_path: String,
    db: State<'_, Database>,
) -> Result<MediaInfo, String> {
    api((|| {
        if !std::path::Path::new(&file_path).exists() {
            return Err(AppError::other(format!("文件不存在: {file_path}")));
        }
        let ffmpeg: FfmpegConfig = db.settings.load_tool_config(ToolId::Ffmpeg)?;
        let bin = resolve_ffprobe_bin(&ffmpeg).ok_or_else(|| {
            AppError::tool_not_found(
                "FFprobe 未找到。请在设置中配置 FFmpeg 目录路径，或使用【下载】按钮自动下载。",
            )
        })?;
        log::info!("Analyzing media file: {file_path}");
        ffprobe::FfprobeAnalyzer::new(bin.to_string_lossy().to_string()).analyze(&file_path)
    })())
}
