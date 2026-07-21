//! 进度历史仓储
//!
//! 下载速率曲线的时序数据（progress_history 表），独立于任务聚合

use std::sync::{Arc, Mutex, MutexGuard};

use rusqlite::{params, Connection};

use crate::domain::download::ProgressPoint;
use crate::shared::{AppError, AppResult};

/// 进度采样点（查询结果，推送给前端图表）
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressSample {
    pub percent: i32,
    pub speed: i64,
    pub downloaded_size: i64,
    /// Unix 毫秒时间戳
    pub recorded_at: i64,
}

/// 进度历史仓储
#[derive(Clone)]
pub struct ProgressHistoryRepository {
    conn: Arc<Mutex<Connection>>,
}

fn lock(conn: &Mutex<Connection>) -> AppResult<MutexGuard<'_, Connection>> {
    conn.lock()
        .map_err(|e| AppError::database(format!("数据库锁获取失败: {e}")))
}

impl ProgressHistoryRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// 批量保存采样点（事务批量插入）
    pub fn save_batch(&self, task_id: &str, points: &[ProgressPoint]) -> AppResult<()> {
        if points.is_empty() {
            return Ok(());
        }
        let mut conn = lock(&self.conn)?;
        let tx = conn.transaction()?;
        {
            let mut stmt = tx.prepare_cached(
                "INSERT INTO progress_history (task_id, percent, speed, downloaded_size, recorded_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
            )?;
            for p in points {
                stmt.execute(params![
                    task_id,
                    p.percent,
                    p.speed,
                    p.downloaded_size,
                    p.timestamp
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    /// 查询任务的进度历史（按时间升序，可选上限）
    pub fn query(&self, task_id: &str, limit: Option<usize>) -> AppResult<Vec<ProgressSample>> {
        let conn = lock(&self.conn)?;
        let limit = limit.unwrap_or(1000) as i64;
        let mut stmt = conn.prepare(
            "SELECT percent, speed, downloaded_size, recorded_at
             FROM progress_history WHERE task_id = ?1
             ORDER BY recorded_at ASC LIMIT ?2",
        )?;
        let rows = stmt
            .query_map(params![task_id, limit], |row| {
                Ok(ProgressSample {
                    percent: row.get(0)?,
                    speed: row.get(1)?,
                    downloaded_size: row.get(2)?,
                    recorded_at: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// 清除任务的进度历史
    pub fn clear(&self, task_id: &str) -> AppResult<()> {
        let conn = lock(&self.conn)?;
        conn.execute("DELETE FROM progress_history WHERE task_id = ?1", [task_id])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::db::schema;

    fn test_repo() -> ProgressHistoryRepository {
        let conn = Connection::open_in_memory().unwrap();
        schema::initialize(&conn).unwrap();
        ProgressHistoryRepository::new(Arc::new(Mutex::new(conn)))
    }

    #[test]
    fn save_batch_and_query_ordered() {
        let repo = test_repo();
        repo.save_batch(
            "t1",
            &[
                ProgressPoint {
                    percent: 10,
                    speed: 100,
                    downloaded_size: 1000,
                    timestamp: 300,
                },
                ProgressPoint {
                    percent: 20,
                    speed: 200,
                    downloaded_size: 2000,
                    timestamp: 100,
                },
            ],
        )
        .unwrap();
        // 空批次不报错
        repo.save_batch("t1", &[]).unwrap();

        let samples = repo.query("t1", None).unwrap();
        assert_eq!(samples.len(), 2);
        // 按时间升序
        assert_eq!(samples[0].recorded_at, 100);
        assert_eq!(samples[1].percent, 10);

        // limit 生效
        assert_eq!(repo.query("t1", Some(1)).unwrap().len(), 1);
        // 其他任务无数据
        assert!(repo.query("t2", None).unwrap().is_empty());
    }

    #[test]
    fn clear_removes_only_target_task() {
        let repo = test_repo();
        repo.save_batch(
            "t1",
            &[ProgressPoint {
                percent: 1,
                speed: 1,
                downloaded_size: 1,
                timestamp: 1,
            }],
        )
        .unwrap();
        repo.save_batch(
            "t2",
            &[ProgressPoint {
                percent: 2,
                speed: 2,
                downloaded_size: 2,
                timestamp: 2,
            }],
        )
        .unwrap();

        repo.clear("t1").unwrap();
        assert!(repo.query("t1", None).unwrap().is_empty());
        assert_eq!(repo.query("t2", None).unwrap().len(), 1);
    }
}
