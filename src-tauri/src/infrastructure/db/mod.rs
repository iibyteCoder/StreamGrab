//! 数据库模块
//!
//! 使用 SQLite 进行统一数据持久化

mod progress_repo;
pub mod repository;
mod schema;
mod settings;
pub mod task;

pub use progress_repo::*;
pub use repository::*;
pub use schema::*;
pub use settings::*;
pub use task::*;

use crate::infrastructure::config::AppConfig;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Arc;

/// 统一数据库管理器
///
/// 包含所有子模块的数据库操作
pub struct Database {
    pub settings: SettingsDb,
    pub tasks: TaskDb,
    pub config: ConfigRepository,
}

impl Database {
    /// 初始化数据库
    ///
    /// 创建数据库文件、表结构
    pub fn initialize(app_config_dir: &PathBuf) -> Result<Arc<Self>, String> {
        // 加载环境配置
        let config = AppConfig::load(app_config_dir).unwrap_or_else(|e| {
            log::warn!("Failed to load config, using defaults: {}", e);
            AppConfig::default()
        });

        // 确保配置目录存在
        std::fs::create_dir_all(app_config_dir)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;

        let db_path = config.get_database_path(app_config_dir);
        log::info!("Opening database at: {:?}", db_path);

        // 打开数据库连接
        let conn =
            Connection::open(&db_path).map_err(|e| format!("Failed to open database: {}", e))?;

        // 启用外键约束
        conn.execute_batch("PRAGMA foreign_keys = ON;")
            .map_err(|e| format!("Failed to enable foreign keys: {}", e))?;

        // 初始化表结构
        initialize_database(&conn)?;

        // 创建各模块管理器
        let settings = SettingsDb::new(
            Connection::open(&db_path)
                .map_err(|e| format!("Failed to open settings connection: {}", e))?,
        )?;

        let tasks = TaskDb::new(
            Connection::open(&db_path)
                .map_err(|e| format!("Failed to open tasks connection: {}", e))?,
        )?;

        let config_repo = ConfigRepository::new(
            Connection::open(&db_path)
                .map_err(|e| format!("Failed to open config connection: {}", e))?,
        )?;

        log::info!("Database initialized successfully");

        Ok(Arc::new(Self {
            settings,
            tasks,
            config: config_repo,
        }))
    }
}
