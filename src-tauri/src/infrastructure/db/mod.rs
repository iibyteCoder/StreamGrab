//! 数据库子系统
//!
//! 单一 SQLite 连接 + `Arc<Mutex<Connection>>` 共享给各仓储；
//! schema v4 单表聚合模型，详见 [`schema`]

pub mod repository;
pub mod schema;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use rusqlite::Connection;

use crate::domain::download::{ProgressPoint, ProgressRepository};
use crate::shared::{AppError, AppResult};
use repository::{
    HistoryRepository, PresetRepository, ProgressHistoryRepository, SettingsRepository,
    TaskRepository,
};

/// 统一数据库
///
/// 各仓储字段为轻量句柄（内部是共享连接的 Arc 克隆）
#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
    db_path: PathBuf,
    /// 任务聚合仓储
    pub tasks: TaskRepository,
    /// 应用与工具配置仓储
    pub settings: SettingsRepository,
    /// 任务预设仓储
    pub presets: PresetRepository,
    /// 历史记录仓储
    pub history: HistoryRepository,
    /// 进度历史仓储
    pub progress: ProgressHistoryRepository,
}

impl Database {
    /// 打开（或创建）配置目录下的数据库并初始化 schema
    pub fn initialize(config_dir: &Path) -> AppResult<Self> {
        std::fs::create_dir_all(config_dir)
            .map_err(|e| AppError::database(format!("创建配置目录失败: {e}")))?;

        let db_path = config_dir.join("streamgrab.db");
        // 版本不符的旧文件直接删除重建（不做数据迁移）
        let conn = schema::open_or_recreate(&db_path)?;

        let conn = Arc::new(Mutex::new(conn));
        Ok(Self {
            tasks: TaskRepository::new(Arc::clone(&conn)),
            settings: SettingsRepository::new(Arc::clone(&conn)),
            presets: PresetRepository::new(Arc::clone(&conn)),
            history: HistoryRepository::new(Arc::clone(&conn)),
            progress: ProgressHistoryRepository::new(Arc::clone(&conn)),
            db_path,
            conn,
        })
    }

    /// 打开内存数据库并初始化 schema（集成测试用，无磁盘副作用）
    pub fn in_memory() -> AppResult<Self> {
        let conn = Connection::open_in_memory()?;
        schema::initialize(&conn)?;
        let conn = Arc::new(Mutex::new(conn));
        Ok(Self {
            tasks: TaskRepository::new(Arc::clone(&conn)),
            settings: SettingsRepository::new(Arc::clone(&conn)),
            presets: PresetRepository::new(Arc::clone(&conn)),
            history: HistoryRepository::new(Arc::clone(&conn)),
            progress: ProgressHistoryRepository::new(Arc::clone(&conn)),
            db_path: PathBuf::from(":memory:"),
            conn,
        })
    }

    /// 数据库文件路径
    pub fn path(&self) -> &Path {
        &self.db_path
    }

    /// 共享连接句柄（供进度跟踪器适配器等特殊场景）
    pub fn connection(&self) -> Arc<Mutex<Connection>> {
        Arc::clone(&self.conn)
    }
}

/// 领域层 `ProgressRepository` 的数据库适配器
///
/// 供 `ProgressTracker`（领域层）持久化采样点，解耦领域与基础设施
pub struct DbProgressRepository {
    inner: ProgressHistoryRepository,
}

impl DbProgressRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self {
            inner: ProgressHistoryRepository::new(conn),
        }
    }
}

impl ProgressRepository for DbProgressRepository {
    fn save(&self, task_id: &str, points: &[ProgressPoint]) -> AppResult<()> {
        self.inner.save_batch(task_id, points)
    }
}
