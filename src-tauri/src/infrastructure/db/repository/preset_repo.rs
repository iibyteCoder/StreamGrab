//! 任务预设仓储
//!
//! 预设 = 命名的 TaskOverrides 组合，持久化于 `task_presets` 表

use std::sync::{Arc, Mutex, MutexGuard};

use rusqlite::{params, Connection, Row};

use crate::domain::task::TaskPreset;
use crate::shared::{AppError, AppResult};

/// 预设仓储
#[derive(Clone)]
pub struct PresetRepository {
    conn: Arc<Mutex<Connection>>,
}

fn lock(conn: &Mutex<Connection>) -> AppResult<MutexGuard<'_, Connection>> {
    conn.lock()
        .map_err(|e| AppError::database(format!("数据库锁获取失败: {e}")))
}

impl PresetRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    fn map_row(row: &Row) -> rusqlite::Result<TaskPreset> {
        let overrides_json: String = row.get("overrides_json")?;
        Ok(TaskPreset {
            id: row.get("id")?,
            name: row.get("name")?,
            icon: row.get("icon")?,
            description: row.get("description")?,
            overrides: serde_json::from_str(&overrides_json).unwrap_or_default(),
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }

    /// 加载全部预设（按创建时间升序）
    pub fn load_all(&self) -> AppResult<Vec<TaskPreset>> {
        let conn = lock(&self.conn)?;
        let mut stmt = conn.prepare(
            "SELECT id, name, icon, description, overrides_json, created_at, updated_at
             FROM task_presets ORDER BY created_at ASC",
        )?;
        let rows = stmt
            .query_map([], Self::map_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// 保存预设（按 ID upsert）
    pub fn save(&self, preset: &TaskPreset) -> AppResult<()> {
        let conn = lock(&self.conn)?;
        conn.execute(
            "INSERT INTO task_presets (id, name, icon, description, overrides_json, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(id) DO UPDATE SET
                 name = excluded.name,
                 icon = excluded.icon,
                 description = excluded.description,
                 overrides_json = excluded.overrides_json,
                 updated_at = excluded.updated_at",
            params![
                preset.id,
                preset.name,
                preset.icon,
                preset.description,
                serde_json::to_string(&preset.overrides)?,
                preset.created_at,
                preset.updated_at,
            ],
        )?;
        Ok(())
    }

    /// 删除预设
    pub fn delete(&self, id: &str) -> AppResult<()> {
        let conn = lock(&self.conn)?;
        conn.execute("DELETE FROM task_presets WHERE id = ?1", [id])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::task::{TaskOverrides, TaskRecord};
    use crate::infrastructure::db::schema;

    fn test_repo() -> PresetRepository {
        let conn = Connection::open_in_memory().unwrap();
        schema::initialize(&conn).unwrap();
        PresetRepository::new(Arc::new(Mutex::new(conn)))
    }

    fn sample_preset(id: &str, name: &str) -> TaskPreset {
        TaskPreset {
            id: id.into(),
            name: name.into(),
            icon: Some("Zap".into()),
            description: None,
            overrides: TaskOverrides {
                max_speed: Some("10M".into()),
                ..Default::default()
            },
            created_at: TaskRecord::now(),
            updated_at: TaskRecord::now(),
        }
    }

    #[test]
    fn save_load_delete_round_trip() {
        let repo = test_repo();
        repo.save(&sample_preset("p1", "最佳质量")).unwrap();
        repo.save(&sample_preset("p2", "极速")).unwrap();

        let all = repo.load_all().unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].name, "最佳质量");
        assert_eq!(all[0].overrides.max_speed.as_deref(), Some("10M"));

        repo.delete("p1").unwrap();
        let all = repo.load_all().unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].id, "p2");
    }

    #[test]
    fn save_upserts_by_id() {
        let repo = test_repo();
        repo.save(&sample_preset("p1", "旧名")).unwrap();
        let mut updated = sample_preset("p1", "新名");
        updated.overrides.max_speed = Some("1M".into());
        repo.save(&updated).unwrap();

        let all = repo.load_all().unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].name, "新名");
        assert_eq!(all[0].overrides.max_speed.as_deref(), Some("1M"));
    }
}
