//! 命令模块公共工具
//!
//! 提供各命令共享的工具函数

use std::sync::Arc;
use tauri::{AppHandle, Manager};

use crate::db::Database;

/// 获取数据库实例
pub fn get_db(app: &AppHandle) -> Result<Arc<Database>, String> {
    app.try_state::<Arc<Database>>()
        .map(|s| s.inner().clone())
        .ok_or_else(|| "Database not initialized".to_string())
}
