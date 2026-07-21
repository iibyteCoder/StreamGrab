//! 历史记录仓储
//!
//! 任务终态快照，独立于任务表：清除任务不删除历史

use std::sync::{Arc, Mutex, MutexGuard};

use rusqlite::{params, Connection, Row};

use crate::domain::task::{HistoryRecord, TaskRecord, TaskStatus};
use crate::shared::{AppError, AppResult};

/// 历史记录仓储
#[derive(Clone)]
pub struct HistoryRepository {
    conn: Arc<Mutex<Connection>>,
}

fn lock(conn: &Mutex<Connection>) -> AppResult<MutexGuard<'_, Connection>> {
    conn.lock()
        .map_err(|e| AppError::database(format!("数据库锁获取失败: {e}")))
}

impl HistoryRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    fn map_row(row: &Row) -> rusqlite::Result<HistoryRecord> {
        let status_str: String = row.get("status")?;
        let overrides_json: Option<String> = row.get("overrides_json")?;
        Ok(HistoryRecord {
            id: row.get("id")?,
            task_id: row.get("task_id")?,
            url: row.get("url")?,
            file_name: row.get("file_name")?,
            save_dir: row.get("save_dir")?,
            output_path: row.get("output_path")?,
            file_size: row.get("file_size")?,
            status: TaskStatus::parse(&status_str).unwrap_or(TaskStatus::Completed),
            error: row.get("error")?,
            created_at: row.get("created_at")?,
            completed_at: row.get("completed_at")?,
            overrides: overrides_json
                .as_deref()
                .and_then(|j| serde_json::from_str(j).ok()),
        })
    }

    /// 从任务聚合记录写入历史快照
    ///
    /// 文件大小取进度数据中的 total_size（尽力而为）
    pub fn insert_from_task(&self, task: &TaskRecord) -> AppResult<()> {
        let conn = lock(&self.conn)?;
        let file_size = (task.progress.total_size > 0).then_some(task.progress.total_size);
        let completed_at = task.completed_at.clone().unwrap_or_else(TaskRecord::now);
        conn.execute(
            "INSERT INTO history (task_id, url, file_name, save_dir, output_path, file_size, status, error, created_at, completed_at, overrides_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                task.id,
                task.url,
                task.file_name,
                task.save_dir,
                task.output_path,
                file_size,
                task.status.as_str(),
                task.error,
                task.created_at,
                completed_at,
                task.overrides
                    .as_ref()
                    .map(serde_json::to_string)
                    .transpose()?,
            ],
        )?;
        Ok(())
    }

    /// 加载全部历史（按完成时间倒序）
    pub fn load_all(&self) -> AppResult<Vec<HistoryRecord>> {
        let conn = lock(&self.conn)?;
        let mut stmt = conn.prepare(
            "SELECT id, task_id, url, file_name, save_dir, output_path, file_size, status, error, created_at, completed_at, overrides_json
             FROM history ORDER BY completed_at DESC, id DESC",
        )?;
        let rows = stmt
            .query_map([], Self::map_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// 删除单条历史
    pub fn delete(&self, id: i64) -> AppResult<()> {
        let conn = lock(&self.conn)?;
        conn.execute("DELETE FROM history WHERE id = ?1", [id])?;
        Ok(())
    }

    /// 清空历史，返回删除数量
    pub fn clear(&self) -> AppResult<usize> {
        let conn = lock(&self.conn)?;
        Ok(conn.execute("DELETE FROM history", [])?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::task::ProgressData;
    use crate::infrastructure::db::schema;

    fn test_repo() -> HistoryRepository {
        let conn = Connection::open_in_memory().unwrap();
        schema::initialize(&conn).unwrap();
        HistoryRepository::new(Arc::new(Mutex::new(conn)))
    }

    fn finished_task(id: &str, status: TaskStatus) -> TaskRecord {
        TaskRecord {
            id: id.into(),
            url: format!("https://example.com/{id}.m3u8"),
            file_name: id.into(),
            save_dir: "D:/Videos".into(),
            output_path: Some(format!("D:/Videos/{id}.mp4")),
            status,
            error: None,
            was_interrupted: false,
            created_at: TaskRecord::now(),
            updated_at: TaskRecord::now(),
            started_at: Some(TaskRecord::now()),
            completed_at: Some(TaskRecord::now()),
            progress: ProgressData {
                total_size: 1024,
                ..Default::default()
            },
            media_info: None,
            overrides: None,
        }
    }

    #[test]
    fn insert_from_task_and_load() {
        let repo = test_repo();
        repo.insert_from_task(&finished_task("t1", TaskStatus::Completed))
            .unwrap();
        repo.insert_from_task(&finished_task("t2", TaskStatus::Failed))
            .unwrap();

        let all = repo.load_all().unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].file_size, Some(1024));
        assert_eq!(all[0].output_path.as_deref(), Some("D:/Videos/t2.mp4"));
        assert!(all.iter().any(|r| r.status == TaskStatus::Failed));
    }

    #[test]
    fn delete_and_clear() {
        let repo = test_repo();
        repo.insert_from_task(&finished_task("t1", TaskStatus::Completed))
            .unwrap();
        repo.insert_from_task(&finished_task("t2", TaskStatus::Completed))
            .unwrap();

        let id = repo.load_all().unwrap()[0].id;
        repo.delete(id).unwrap();
        assert_eq!(repo.load_all().unwrap().len(), 1);

        assert_eq!(repo.clear().unwrap(), 1);
        assert!(repo.load_all().unwrap().is_empty());
    }
}
