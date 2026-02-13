//! 任务表操作
//!
//! 管理下载任务状态

use rusqlite::{Connection, params, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

/// 任务记录（数据库存储格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRecord {
    pub id: String,
    pub url: String,
    pub file_name: String,
    pub save_dir: String,
    pub output_path: Option<String>,
    pub status: String,
    pub error: Option<String>,
    pub progress_json: String,
    pub config_json: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub was_interrupted: bool,
}

/// 任务数据库管理器
pub struct TaskDb {
    conn: Mutex<Connection>,
}

impl TaskDb {
    /// 创建任务管理器
    pub fn new(conn: Connection) -> Result<Self, String> {
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// 加载所有任务
    pub fn load_all(&self) -> Result<Vec<TaskRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare(
                "SELECT id, url, file_name, save_dir, output_path, status, error,
                        progress_json, config_json, created_at, updated_at, started_at,
                        completed_at, was_interrupted
                 FROM tasks
                 ORDER BY created_at DESC",
            )
            .map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let tasks = stmt
            .query_map([], |row| {
                Ok(TaskRecord {
                    id: row.get(0)?,
                    url: row.get(1)?,
                    file_name: row.get(2)?,
                    save_dir: row.get(3)?,
                    output_path: row.get(4)?,
                    status: row.get(5)?,
                    error: row.get(6)?,
                    progress_json: row.get(7)?,
                    config_json: row.get(8)?,
                    created_at: row.get(9)?,
                    updated_at: row.get(10)?,
                    started_at: row.get(11)?,
                    completed_at: row.get(12)?,
                    was_interrupted: row.get::<_, i64>(13)? != 0,
                })
            })
            .map_err(|e| format!("Failed to query tasks: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect tasks: {}", e))?;

        Ok(tasks)
    }

    /// 根据 ID 获取任务
    #[allow(dead_code)]
    pub fn get(&self, id: &str) -> Result<Option<TaskRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let result = conn
            .query_row(
                "SELECT id, url, file_name, save_dir, output_path, status, error,
                        progress_json, config_json, created_at, updated_at, started_at,
                        completed_at, was_interrupted
                 FROM tasks WHERE id = ?1",
                params![id],
                |row| {
                    Ok(TaskRecord {
                        id: row.get(0)?,
                        url: row.get(1)?,
                        file_name: row.get(2)?,
                        save_dir: row.get(3)?,
                        output_path: row.get(4)?,
                        status: row.get(5)?,
                        error: row.get(6)?,
                        progress_json: row.get(7)?,
                        config_json: row.get(8)?,
                        created_at: row.get(9)?,
                        updated_at: row.get(10)?,
                        started_at: row.get(11)?,
                        completed_at: row.get(12)?,
                        was_interrupted: row.get::<_, i64>(13)? != 0,
                    })
                },
            )
            .optional()
            .map_err(|e| format!("Failed to query task: {}", e))?;

        Ok(result)
    }

    /// 根据状态加载任务
    #[allow(dead_code)]
    pub fn load_by_status(&self, statuses: &[&str]) -> Result<Vec<TaskRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let placeholders: Vec<String> = statuses.iter().map(|_| "?".to_string()).collect();
        let sql = format!(
            "SELECT id, url, file_name, save_dir, output_path, status, error,
                    progress_json, config_json, created_at, updated_at, started_at,
                    completed_at, was_interrupted
             FROM tasks WHERE status IN ({}) ORDER BY created_at DESC",
            placeholders.join(", ")
        );

        let params: Vec<&dyn rusqlite::ToSql> = statuses.iter().map(|s| s as &dyn rusqlite::ToSql).collect();

        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let tasks = stmt
            .query_map(params.as_slice(), |row| {
                Ok(TaskRecord {
                    id: row.get(0)?,
                    url: row.get(1)?,
                    file_name: row.get(2)?,
                    save_dir: row.get(3)?,
                    output_path: row.get(4)?,
                    status: row.get(5)?,
                    error: row.get(6)?,
                    progress_json: row.get(7)?,
                    config_json: row.get(8)?,
                    created_at: row.get(9)?,
                    updated_at: row.get(10)?,
                    started_at: row.get(11)?,
                    completed_at: row.get(12)?,
                    was_interrupted: row.get::<_, i64>(13)? != 0,
                })
            })
            .map_err(|e| format!("Failed to query tasks: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect tasks: {}", e))?;

        Ok(tasks)
    }

    /// 加载可恢复的任务（被中断的下载）
    pub fn load_recoverable(&self) -> Result<Vec<TaskRecord>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare(
                "SELECT id, url, file_name, save_dir, output_path, status, error,
                        progress_json, config_json, created_at, updated_at, started_at,
                        completed_at, was_interrupted
                 FROM tasks
                 WHERE was_interrupted = 1 OR status IN ('downloading', 'paused', 'analyzing')
                 ORDER BY created_at DESC",
            )
            .map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let tasks = stmt
            .query_map([], |row| {
                Ok(TaskRecord {
                    id: row.get(0)?,
                    url: row.get(1)?,
                    file_name: row.get(2)?,
                    save_dir: row.get(3)?,
                    output_path: row.get(4)?,
                    status: row.get(5)?,
                    error: row.get(6)?,
                    progress_json: row.get(7)?,
                    config_json: row.get(8)?,
                    created_at: row.get(9)?,
                    updated_at: row.get(10)?,
                    started_at: row.get(11)?,
                    completed_at: row.get(12)?,
                    was_interrupted: row.get::<_, i64>(13)? != 0,
                })
            })
            .map_err(|e| format!("Failed to query recoverable tasks: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect tasks: {}", e))?;

        Ok(tasks)
    }

    /// 保存/更新任务（upsert）
    pub fn upsert(&self, task: &TaskRecord) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT INTO tasks (id, url, file_name, save_dir, output_path, status, error,
                              progress_json, config_json, created_at, updated_at, started_at,
                              completed_at, was_interrupted)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
             ON CONFLICT(id) DO UPDATE SET
                 url = excluded.url,
                 file_name = excluded.file_name,
                 save_dir = excluded.save_dir,
                 output_path = excluded.output_path,
                 status = excluded.status,
                 error = excluded.error,
                 progress_json = excluded.progress_json,
                 config_json = excluded.config_json,
                 updated_at = excluded.updated_at,
                 started_at = excluded.started_at,
                 completed_at = excluded.completed_at,
                 was_interrupted = excluded.was_interrupted",
            params![
                task.id,
                task.url,
                task.file_name,
                task.save_dir,
                task.output_path,
                task.status,
                task.error,
                task.progress_json,
                task.config_json,
                task.created_at,
                task.updated_at,
                task.started_at,
                task.completed_at,
                task.was_interrupted as i64,
            ],
        )
        .map_err(|e| format!("Failed to upsert task: {}", e))?;

        Ok(())
    }

    /// 更新任务状态
    pub fn update_status(&self, id: &str, status: &str, error: Option<&str>) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "UPDATE tasks SET status = ?1, error = ?2, updated_at = ?3 WHERE id = ?4",
            params![status, error, now, id],
        )
        .map_err(|e| format!("Failed to update task status: {}", e))?;

        Ok(())
    }

    /// 更新任务进度
    pub fn update_progress(&self, id: &str, progress_json: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "UPDATE tasks SET progress_json = ?1, updated_at = ?2 WHERE id = ?3",
            params![progress_json, now, id],
        )
        .map_err(|e| format!("Failed to update task progress: {}", e))?;

        Ok(())
    }

    /// 删除任务
    pub fn delete(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        conn.execute("DELETE FROM tasks WHERE id = ?1", params![id])
            .map_err(|e| format!("Failed to delete task: {}", e))?;

        Ok(())
    }

    /// 删除指定状态的任务
    pub fn delete_by_status(&self, statuses: &[&str]) -> Result<usize, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let placeholders: Vec<String> = statuses.iter().map(|_| "?".to_string()).collect();
        let sql = format!("DELETE FROM tasks WHERE status IN ({})", placeholders.join(", "));
        let params: Vec<&dyn rusqlite::ToSql> = statuses.iter().map(|s| s as &dyn rusqlite::ToSql).collect();

        let rows_affected = conn
            .execute(&sql, params.as_slice())
            .map_err(|e| format!("Failed to delete tasks by status: {}", e))?;

        Ok(rows_affected)
    }

    /// 清除已完成的任务
    pub fn clear_finished(&self) -> Result<usize, String> {
        self.delete_by_status(&["completed", "failed", "cancelled"])
    }

    /// 标记活跃任务为已中断
    pub fn mark_active_interrupted(&self) -> Result<usize, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let rows_affected = conn
            .execute(
                "UPDATE tasks SET was_interrupted = 1, status = 'paused'
                 WHERE status IN ('downloading', 'analyzing', 'merging', 'muxing')",
                [],
            )
            .map_err(|e| format!("Failed to mark tasks as interrupted: {}", e))?;

        Ok(rows_affected)
    }

    /// 清除所有任务
    pub fn clear_all(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        conn.execute("DELETE FROM tasks", [])
            .map_err(|e| format!("Failed to clear tasks: {}", e))?;

        Ok(())
    }
}
