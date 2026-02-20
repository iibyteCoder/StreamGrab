//! 进度历史仓储实现
//!
//! 实现 domain 层的 ProgressRepository trait

use std::sync::Arc;

use super::Database;
use crate::domain::download::{ProgressPoint, ProgressRepository};

/// 数据库进度仓储
pub struct DbProgressRepository {
    db: Arc<Database>,
}

impl DbProgressRepository {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
}

impl ProgressRepository for DbProgressRepository {
    fn save(&self, task_id: &str, points: &[ProgressPoint]) -> Result<(), String> {
        for point in points {
            self.db.tasks.add_progress_history(
                task_id,
                point.percent,
                point.speed,
                point.downloaded_size,
            )?;
        }
        Ok(())
    }
}
