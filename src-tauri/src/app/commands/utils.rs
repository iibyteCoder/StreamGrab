//! 命令模块公共工具
//!
//! 提供各命令共享的工具函数

use std::sync::Arc;
use tauri::{AppHandle, Manager};

use crate::infrastructure::db::Database;

/// 获取数据库实例
pub fn get_db(app: &AppHandle) -> Result<Arc<Database>, String> {
    app.try_state::<Arc<Database>>()
        .map(|s| s.inner().clone())
        .ok_or_else(|| "Database not initialized".to_string())
}

/// 工具路径配置
pub struct ToolPathsConfig {
    pub m3u8dl_path: Option<String>,
    pub ffmpeg_path: Option<String>,
    pub ffprobe_path: Option<String>,
}

/// 从数据库配置中获取工具路径
pub fn get_tool_paths_from_config(app: &AppHandle) -> ToolPathsConfig {
    let db = match get_db(app) {
        Ok(db) => db,
        Err(e) => {
            log::error!("Failed to get database for tool paths: {}", e);
            return ToolPathsConfig {
                m3u8dl_path: None,
                ffmpeg_path: None,
                ffprobe_path: None,
            };
        }
    };

    // 获取 M3U8DL 配置中的工具路径
    let m3u8dl_path = match db.config.get_m3u8dl_settings() {
        Ok(settings) if !settings.n_m3u8dl_path.is_empty() => Some(settings.n_m3u8dl_path.clone()),
        _ => None,
    };

    // 获取 FFmpeg 配置中的工具路径
    let (ffmpeg_path, ffprobe_path) = match db.config.get_ffmpeg_settings() {
        Ok(settings) => {
            let ffmpeg = if !settings.ffmpeg_path.is_empty() {
                Some(settings.ffmpeg_path.clone())
            } else {
                None
            };
            let ffprobe = if !settings.ffprobe_path.is_empty() {
                Some(settings.ffprobe_path.clone())
            } else {
                None
            };
            (ffmpeg, ffprobe)
        }
        Err(e) => {
            log::error!("Failed to load ffmpeg settings: {}", e);
            (None, None)
        }
    };

    log::debug!(
        "Tool paths from config: m3u8dl_path={:?}, ffmpeg_path={:?}, ffprobe_path={:?}",
        m3u8dl_path,
        ffmpeg_path,
        ffprobe_path
    );

    ToolPathsConfig {
        m3u8dl_path,
        ffmpeg_path,
        ffprobe_path,
    }
}

/// 获取 M3U8DL 可执行文件路径
pub fn get_m3u8dl_path(app: &AppHandle) -> Option<String> {
    get_tool_paths_from_config(app).m3u8dl_path
}

/// 获取 FFmpeg 可执行文件路径
pub fn get_ffmpeg_path(app: &AppHandle) -> Option<String> {
    get_tool_paths_from_config(app).ffmpeg_path
}

/// 获取 FFprobe 可执行文件路径
pub fn get_ffprobe_path(app: &AppHandle) -> Option<String> {
    get_tool_paths_from_config(app).ffprobe_path
}
