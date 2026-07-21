//! 任务仓储
//!
//! `tasks` 单表聚合的 CRUD：基础列 + 三个 JSON 列（progress / media_info / overrides）

use std::sync::{Arc, Mutex, MutexGuard};

use rusqlite::{params, Connection, Row};

use crate::domain::media::MediaInfo;
use crate::domain::task::TaskOverrides;
use crate::domain::task::{ProgressData, TaskRecord, TaskStatus};
use crate::shared::{AppError, AppResult};

/// 任务仓储
#[derive(Clone)]
pub struct TaskRepository {
    conn: Arc<Mutex<Connection>>,
}

/// 获取连接（Mutex 中毒转为领域错误，避免级联 panic）
fn lock(conn: &Mutex<Connection>) -> AppResult<MutexGuard<'_, Connection>> {
    conn.lock()
        .map_err(|e| AppError::database(format!("数据库锁获取失败: {e}")))
}

impl TaskRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// 行 → 聚合记录（唯一映射点，消除旧版 3 处复制粘贴）
    fn map_row(row: &Row) -> rusqlite::Result<TaskRecord> {
        let progress_json: String = row.get("progress_json")?;
        let media_info_json: Option<String> = row.get("media_info_json")?;
        let overrides_json: Option<String> = row.get("overrides_json")?;
        let status_str: String = row.get("status")?;

        Ok(TaskRecord {
            id: row.get("id")?,
            url: row.get("url")?,
            file_name: row.get("file_name")?,
            save_dir: row.get("save_dir")?,
            output_path: row.get("output_path")?,
            status: TaskStatus::parse(&status_str).unwrap_or(TaskStatus::Pending),
            error: row.get("error")?,
            was_interrupted: row.get("was_interrupted")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
            started_at: row.get("started_at")?,
            completed_at: row.get("completed_at")?,
            progress: serde_json::from_str(&progress_json).unwrap_or_default(),
            media_info: media_info_json
                .as_deref()
                .and_then(|j| serde_json::from_str(j).ok()),
            overrides: overrides_json
                .as_deref()
                .and_then(|j| serde_json::from_str(j).ok()),
        })
    }

    const COLUMNS: &'static str = "id, url, file_name, save_dir, output_path, status, error, \
        was_interrupted, created_at, updated_at, started_at, completed_at, \
        progress_json, media_info_json, overrides_json";

    /// 创建任务
    pub fn create(&self, task: &TaskRecord) -> AppResult<()> {
        let conn = lock(&self.conn)?;
        conn.execute(
            &format!(
                "INSERT INTO tasks ({})
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
                Self::COLUMNS
            ),
            params![
                task.id,
                task.url,
                task.file_name,
                task.save_dir,
                task.output_path,
                task.status.as_str(),
                task.error,
                task.was_interrupted,
                task.created_at,
                task.updated_at,
                task.started_at,
                task.completed_at,
                serde_json::to_string(&task.progress)?,
                task.media_info
                    .as_ref()
                    .map(serde_json::to_string)
                    .transpose()?,
                task.overrides
                    .as_ref()
                    .map(serde_json::to_string)
                    .transpose()?,
            ],
        )?;
        Ok(())
    }

    /// 加载全部任务（按创建时间倒序）
    pub fn load_all(&self) -> AppResult<Vec<TaskRecord>> {
        let conn = lock(&self.conn)?;
        let mut stmt = conn.prepare(&format!(
            "SELECT {} FROM tasks ORDER BY created_at DESC",
            Self::COLUMNS
        ))?;
        let rows = stmt
            .query_map([], Self::map_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// 按 ID 获取
    pub fn get(&self, id: &str) -> AppResult<Option<TaskRecord>> {
        let conn = lock(&self.conn)?;
        let mut stmt = conn.prepare(&format!(
            "SELECT {} FROM tasks WHERE id = ?1",
            Self::COLUMNS
        ))?;
        let mut rows = stmt.query_map([id], Self::map_row)?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    /// 加载可恢复任务（被中断的 + 活跃状态的）
    pub fn load_recoverable(&self) -> AppResult<Vec<TaskRecord>> {
        let conn = lock(&self.conn)?;
        let mut stmt = conn.prepare(&format!(
            "SELECT {} FROM tasks
             WHERE was_interrupted = 1
                OR status IN ('analyzing', 'downloading', 'merging', 'muxing', 'paused')
             ORDER BY created_at DESC",
            Self::COLUMNS
        ))?;
        let rows = stmt
            .query_map([], Self::map_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// 更新状态
    ///
    /// 持久化规则：首次进入活跃态写 `started_at`；进入终态写 `completed_at`；
    /// `error` 原样写入（None 清空旧错误，支持重试场景）
    pub fn update_status(
        &self,
        id: &str,
        status: TaskStatus,
        error: Option<&str>,
    ) -> AppResult<()> {
        let conn = lock(&self.conn)?;
        let now = TaskRecord::now();

        let started_at_expr = if status.is_active() {
            "COALESCE(started_at, ?3)"
        } else {
            "started_at"
        };
        let completed_at_expr = if status.is_finished() {
            "?3"
        } else {
            "completed_at"
        };

        conn.execute(
            &format!(
                "UPDATE tasks SET status = ?2, error = ?4, was_interrupted = 0,
                     updated_at = ?3, started_at = {started_at_expr}, completed_at = {completed_at_expr}
                 WHERE id = ?1"
            ),
            params![id, status.as_str(), now, error],
        )?;
        Ok(())
    }

    /// 更新进度（JSON 列）
    pub fn update_progress(&self, id: &str, progress: &ProgressData) -> AppResult<()> {
        let conn = lock(&self.conn)?;
        conn.execute(
            "UPDATE tasks SET progress_json = ?2, updated_at = ?3 WHERE id = ?1",
            params![id, serde_json::to_string(progress)?, TaskRecord::now()],
        )?;
        Ok(())
    }

    /// 更新媒体信息（JSON 列）
    pub fn update_media_info(&self, id: &str, info: &MediaInfo) -> AppResult<()> {
        let conn = lock(&self.conn)?;
        conn.execute(
            "UPDATE tasks SET media_info_json = ?2, updated_at = ?3 WHERE id = ?1",
            params![id, serde_json::to_string(info)?, TaskRecord::now()],
        )?;
        Ok(())
    }

    /// 更新输出路径
    pub fn update_output_path(&self, id: &str, path: &str) -> AppResult<()> {
        let conn = lock(&self.conn)?;
        conn.execute(
            "UPDATE tasks SET output_path = ?2, updated_at = ?3 WHERE id = ?1",
            params![id, path, TaskRecord::now()],
        )?;
        Ok(())
    }

    /// 保存任务级覆盖（JSON 列）
    pub fn save_overrides(&self, id: &str, overrides: &TaskOverrides) -> AppResult<()> {
        let conn = lock(&self.conn)?;
        conn.execute(
            "UPDATE tasks SET overrides_json = ?2, updated_at = ?3 WHERE id = ?1",
            params![id, serde_json::to_string(overrides)?, TaskRecord::now()],
        )?;
        Ok(())
    }

    /// 删除任务
    pub fn delete(&self, id: &str) -> AppResult<()> {
        let conn = lock(&self.conn)?;
        conn.execute("DELETE FROM progress_history WHERE task_id = ?1", [id])?;
        conn.execute("DELETE FROM tasks WHERE id = ?1", [id])?;
        Ok(())
    }

    /// 按状态批量删除，返回删除数量
    pub fn delete_by_status(&self, statuses: &[TaskStatus]) -> AppResult<usize> {
        if statuses.is_empty() {
            return Ok(0);
        }
        let conn = lock(&self.conn)?;
        let placeholders = std::iter::repeat("?")
            .take(statuses.len())
            .collect::<Vec<_>>()
            .join(", ");
        let params: Vec<String> = statuses.iter().map(|s| s.as_str().to_string()).collect();
        let params: Vec<&dyn rusqlite::ToSql> =
            params.iter().map(|s| s as &dyn rusqlite::ToSql).collect();

        // 先清理关联的进度历史
        conn.execute(
            &format!(
                "DELETE FROM progress_history WHERE task_id IN (SELECT id FROM tasks WHERE status IN ({placeholders}))"
            ),
            params.as_slice(),
        )?;
        let deleted = conn.execute(
            &format!("DELETE FROM tasks WHERE status IN ({placeholders})"),
            params.as_slice(),
        )?;
        Ok(deleted)
    }

    /// 清除全部已结束任务，返回删除数量
    pub fn clear_finished(&self) -> AppResult<usize> {
        self.delete_by_status(&[
            TaskStatus::Completed,
            TaskStatus::Failed,
            TaskStatus::Cancelled,
        ])
    }

    /// 清空全部任务
    pub fn clear_all(&self) -> AppResult<()> {
        let conn = lock(&self.conn)?;
        conn.execute("DELETE FROM progress_history", [])?;
        conn.execute("DELETE FROM tasks", [])?;
        Ok(())
    }

    /// 将活跃任务标记为已中断（应用启动时调用），返回标记数量
    pub fn mark_active_interrupted(&self) -> AppResult<usize> {
        let conn = lock(&self.conn)?;
        let updated = conn.execute(
            "UPDATE tasks SET was_interrupted = 1, status = 'pending', updated_at = ?1
             WHERE status IN ('analyzing', 'downloading', 'merging', 'muxing')",
            [TaskRecord::now()],
        )?;
        Ok(updated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::db::schema;

    fn test_repo() -> TaskRepository {
        let conn = Connection::open_in_memory().unwrap();
        schema::initialize(&conn).unwrap();
        TaskRepository::new(Arc::new(Mutex::new(conn)))
    }

    fn sample_task(id: &str) -> TaskRecord {
        TaskRecord {
            id: id.into(),
            url: "https://example.com/index.m3u8".into(),
            file_name: "video".into(),
            save_dir: "D:/Videos".into(),
            output_path: None,
            status: TaskStatus::Pending,
            error: None,
            was_interrupted: false,
            created_at: TaskRecord::now(),
            updated_at: TaskRecord::now(),
            started_at: None,
            completed_at: None,
            progress: ProgressData::default(),
            media_info: None,
            overrides: Some(TaskOverrides {
                max_speed: Some("5M".into()),
                ..Default::default()
            }),
        }
    }

    #[test]
    fn create_and_get_round_trip() {
        let repo = test_repo();
        let task = sample_task("t1");
        repo.create(&task).unwrap();

        let loaded = repo.get("t1").unwrap().unwrap();
        assert_eq!(loaded.id, "t1");
        assert_eq!(loaded.url, task.url);
        assert_eq!(loaded.status, TaskStatus::Pending);
        assert_eq!(
            loaded.overrides.as_ref().unwrap().max_speed.as_deref(),
            Some("5M")
        );
        assert!(repo.get("nope").unwrap().is_none());
    }

    #[test]
    fn status_updates_manage_timestamps() {
        let repo = test_repo();
        repo.create(&sample_task("t1")).unwrap();

        repo.update_status("t1", TaskStatus::Downloading, None)
            .unwrap();
        let t = repo.get("t1").unwrap().unwrap();
        assert_eq!(t.status, TaskStatus::Downloading);
        assert!(t.started_at.is_some(), "进入活跃态应写 started_at");
        assert!(t.completed_at.is_none());

        repo.update_status("t1", TaskStatus::Failed, Some("网络错误"))
            .unwrap();
        let t = repo.get("t1").unwrap().unwrap();
        assert_eq!(t.status, TaskStatus::Failed);
        assert_eq!(t.error.as_deref(), Some("网络错误"));
        assert!(t.completed_at.is_some(), "进入终态应写 completed_at");

        // 重试：错误清空
        repo.update_status("t1", TaskStatus::Pending, None).unwrap();
        let t = repo.get("t1").unwrap().unwrap();
        assert!(t.error.is_none());
    }

    #[test]
    fn progress_and_media_info_persist_as_json() {
        let repo = test_repo();
        repo.create(&sample_task("t1")).unwrap();

        let progress = ProgressData {
            percent: 42,
            speed: 1024,
            ..Default::default()
        };
        repo.update_progress("t1", &progress).unwrap();

        let mut media = MediaInfo::default();
        media.resolution = Some("1920x1080".into());
        repo.update_media_info("t1", &media).unwrap();

        let t = repo.get("t1").unwrap().unwrap();
        assert_eq!(t.progress.percent, 42);
        assert_eq!(
            t.media_info.unwrap().resolution.as_deref(),
            Some("1920x1080")
        );
    }

    #[test]
    fn recoverable_and_cleanup_queries() {
        let repo = test_repo();
        for id in ["t1", "t2", "t3"] {
            repo.create(&sample_task(id)).unwrap();
        }
        repo.update_status("t1", TaskStatus::Downloading, None)
            .unwrap();
        repo.update_status("t2", TaskStatus::Completed, None)
            .unwrap();
        // t3 pending

        // 模拟中断标记：活跃任务 → pending + was_interrupted
        let marked = repo.mark_active_interrupted().unwrap();
        assert_eq!(marked, 1);
        let recoverable = repo.load_recoverable().unwrap();
        assert_eq!(recoverable.len(), 1);
        assert_eq!(recoverable[0].id, "t1");
        assert!(recoverable[0].was_interrupted);

        // 清除已结束
        let cleared = repo.clear_finished().unwrap();
        assert_eq!(cleared, 1);
        assert_eq!(repo.load_all().unwrap().len(), 2);

        // 清空
        repo.clear_all().unwrap();
        assert!(repo.load_all().unwrap().is_empty());
    }

    #[test]
    fn delete_removes_progress_history() {
        let repo = test_repo();
        repo.create(&sample_task("t1")).unwrap();
        // 直接插入进度历史行验证级联清理
        {
            let conn = lock(&repo.conn).unwrap();
            conn.execute(
                "INSERT INTO progress_history (task_id, percent, speed, downloaded_size, recorded_at) VALUES ('t1', 10, 100, 1000, 123)",
                [],
            ).unwrap();
        }
        repo.delete("t1").unwrap();
        let conn = lock(&repo.conn).unwrap();
        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM progress_history WHERE task_id = 't1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);
    }
}
