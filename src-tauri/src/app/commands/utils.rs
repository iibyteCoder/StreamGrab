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
    pub downloader_dir: Option<String>,
    pub ffmpeg_dir: Option<String>,
}

/// 从数据库配置中获取工具路径
pub fn get_tool_paths_from_config(app: &AppHandle) -> ToolPathsConfig {
    let db = match get_db(app) {
        Ok(db) => db,
        Err(e) => {
            log::error!("Failed to get database for tool paths: {}", e);
            return ToolPathsConfig {
                downloader_dir: None,
                ffmpeg_dir: None,
            };
        }
    };

    // 加载 advanced 配置
    let advanced = match db.settings.load("advanced") {
        Ok(config) => config,
        Err(e) => {
            log::error!("Failed to load advanced settings: {}", e);
            return ToolPathsConfig {
                downloader_dir: None,
                ffmpeg_dir: None,
            };
        }
    };

    // 提取工具路径
    let downloader_dir = advanced
        .get("n_m3u8dlPath")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    let ffmpeg_dir = advanced
        .get("ffmpegPath")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    log::debug!(
        "Tool paths from config: downloader_dir={:?}, ffmpeg_dir={:?}",
        downloader_dir,
        ffmpeg_dir
    );

    ToolPathsConfig {
        downloader_dir,
        ffmpeg_dir,
    }
}
