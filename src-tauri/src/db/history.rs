//! 历史记录数据库操作

use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

/// 历史记录项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryRecord {
    pub id: String,
    pub url: String,
    pub file_name: String,
    pub save_path: String,
    pub file_size: u64,
    pub duration: f64,
    pub completed_at: String,
    pub task_id: Option<String>,
}

/// 历史记录数据库
pub struct HistoryDb {
    conn: Mutex<Connection>,
}

impl HistoryDb {
    /// 创建历史记录管理器
    ///
    /// 注意：表结构由 schema.rs 统一创建
    pub fn new(conn: Connection) -> Result<Self, String> {
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// 加载所有历史记录
    pub fn load_all(&self) -> Result<Vec<HistoryRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare(
                "SELECT id, url, file_name, save_path, file_size, duration, completed_at, task_id
                 FROM history
                 ORDER BY completed_at DESC",
            )
            .map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let records = stmt
            .query_map([], |row| {
                Ok(HistoryRecord {
                    id: row.get(0)?,
                    url: row.get(1)?,
                    file_name: row.get(2)?,
                    save_path: row.get(3)?,
                    file_size: row.get::<_, i64>(4)? as u64,
                    duration: row.get(5)?,
                    completed_at: row.get(6)?,
                    task_id: row.get(7)?,
                })
            })
            .map_err(|e| format!("Failed to query history: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect records: {}", e))?;

        Ok(records)
    }

    /// 添加历史记录
    pub fn add(&self, record: &HistoryRecord) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT INTO history (id, url, file_name, save_path, file_size, duration, completed_at, task_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                record.id,
                record.url,
                record.file_name,
                record.save_path,
                record.file_size as i64,
                record.duration,
                record.completed_at,
                record.task_id
            ],
        )
        .map_err(|e| format!("Failed to insert history record: {}", e))?;

        // 限制最多保存 100 条记录
        self.limit_records(&conn, 100)?;

        Ok(())
    }

    /// 清除所有历史记录
    pub fn clear(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        conn.execute("DELETE FROM history", [])
            .map_err(|e| format!("Failed to clear history: {}", e))?;

        Ok(())
    }

    /// 删除指定记录
    pub fn delete(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        conn.execute("DELETE FROM history WHERE id = ?1", params![id])
            .map_err(|e| format!("Failed to delete history record: {}", e))?;

        Ok(())
    }

    /// 限制记录数量（保留最新的 N 条）
    fn limit_records(&self, conn: &Connection, max_count: usize) -> Result<(), String> {
        conn.execute(
            "DELETE FROM history WHERE id IN (
                SELECT id FROM history ORDER BY completed_at DESC LIMIT -1 OFFSET ?1
            )",
            params![max_count as i64],
        )
        .map_err(|e| format!("Failed to limit records: {}", e))?;

        Ok(())
    }
}
