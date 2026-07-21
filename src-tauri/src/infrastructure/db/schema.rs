//! 数据库 Schema（v4）
//!
//! v4 采用单表聚合模型（tasks 含三个 JSON 列）+ 通用工具配置表，
//! 与旧版多表结构不兼容：版本低于 4 时直接重建全部表
//!（用户已确认无需向后兼容，旧数据不保留）。

use rusqlite::Connection;

use crate::shared::AppResult;

/// 当前 schema 版本
pub const SCHEMA_VERSION: i32 = 4;

/// 全量 DDL（幂等，`IF NOT EXISTS`）
const DDL: &str = r#"
CREATE TABLE IF NOT EXISTS schema_info (
    version INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS tasks (
    id TEXT PRIMARY KEY,
    url TEXT NOT NULL,
    file_name TEXT NOT NULL,
    save_dir TEXT NOT NULL,
    output_path TEXT,
    status TEXT NOT NULL DEFAULT 'pending',
    error TEXT,
    was_interrupted INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    started_at TEXT,
    completed_at TEXT,
    progress_json TEXT NOT NULL DEFAULT '{}',
    media_info_json TEXT,
    overrides_json TEXT
);
CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
CREATE INDEX IF NOT EXISTS idx_tasks_created_at ON tasks(created_at);

CREATE TABLE IF NOT EXISTS progress_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id TEXT NOT NULL,
    percent INTEGER NOT NULL,
    speed INTEGER NOT NULL,
    downloaded_size INTEGER NOT NULL,
    recorded_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_progress_history_task ON progress_history(task_id);

CREATE TABLE IF NOT EXISTS history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id TEXT,
    url TEXT NOT NULL,
    file_name TEXT NOT NULL,
    save_dir TEXT NOT NULL,
    output_path TEXT,
    file_size INTEGER,
    status TEXT NOT NULL,
    error TEXT,
    created_at TEXT NOT NULL,
    completed_at TEXT NOT NULL,
    overrides_json TEXT
);
CREATE INDEX IF NOT EXISTS idx_history_completed_at ON history(completed_at);

CREATE TABLE IF NOT EXISTS app_settings (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    settings_json TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS tool_settings (
    tool_id TEXT PRIMARY KEY,
    config_json TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS task_presets (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    icon TEXT,
    description TEXT,
    overrides_json TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
"#;

/// 需要清理的表（含历史版本遗留表名）
const ALL_TABLES: [&str; 14] = [
    "tasks",
    "task_progress",
    "task_media_info",
    "task_config",
    "progress_history",
    "history",
    "settings",
    "app_settings",
    "m3u8dl_settings",
    "ffmpeg_settings",
    "network_headers",
    "decryption_keys",
    "config_templates",
    "task_presets",
];

/// 初始化 schema：设置 PRAGMA，必要时迁移到当前版本
pub fn initialize(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA foreign_keys = ON;
         PRAGMA busy_timeout = 5000;",
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_info (version INTEGER NOT NULL)",
        [],
    )?;

    let version: i32 = conn
        .query_row("SELECT version FROM schema_info LIMIT 1", [], |r| r.get(0))
        .unwrap_or(0);

    if version < SCHEMA_VERSION {
        rebuild(conn)?;
        log::info!("Database schema initialized at v{SCHEMA_VERSION}");
    }

    Ok(())
}

/// 全量重建：drop 所有表 → DDL → 写入版本号
fn rebuild(conn: &Connection) -> AppResult<()> {
    for table in ALL_TABLES {
        conn.execute_batch(&format!("DROP TABLE IF EXISTS {table};"))?;
    }
    conn.execute_batch(DDL)?;
    conn.execute("DELETE FROM schema_info", [])?;
    conn.execute(
        "INSERT INTO schema_info (version) VALUES (?1)",
        [SCHEMA_VERSION],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_db_initializes_to_current_version() {
        let conn = Connection::open_in_memory().unwrap();
        initialize(&conn).unwrap();
        let version: i32 = conn
            .query_row("SELECT version FROM schema_info LIMIT 1", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version, SCHEMA_VERSION);

        // 所有 v4 表存在
        for table in [
            "tasks",
            "progress_history",
            "history",
            "app_settings",
            "tool_settings",
            "task_presets",
        ] {
            let count: i32 = conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                    [table],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(count, 1, "表 {table} 应存在");
        }
    }

    #[test]
    fn initialize_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        initialize(&conn).unwrap();
        conn.execute("INSERT INTO tasks (id, url, file_name, save_dir, status, created_at, updated_at) VALUES ('t1','u','f','d','pending','now','now')", []).unwrap();
        // 再次初始化不重建（版本相同），数据保留
        initialize(&conn).unwrap();
        let count: i32 = conn
            .query_row("SELECT COUNT(*) FROM tasks", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn migrates_from_old_version_by_rebuild() {
        let conn = Connection::open_in_memory().unwrap();
        // 模拟旧版 v3：只有 schema_info 与一张遗留表
        conn.execute_batch(
            "CREATE TABLE schema_info (version INTEGER NOT NULL);
             INSERT INTO schema_info (version) VALUES (3);
             CREATE TABLE settings (module TEXT PRIMARY KEY, data TEXT);
             INSERT INTO settings (module, data) VALUES ('general', '{}');",
        )
        .unwrap();

        initialize(&conn).unwrap();

        let version: i32 = conn
            .query_row("SELECT version FROM schema_info LIMIT 1", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version, SCHEMA_VERSION);
        // 旧表已删除
        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='settings'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);
    }
}
