//! 数据库 Schema（v4）
//!
//! v4 采用单表聚合模型（tasks 含三个 JSON 列）+ 通用工具配置表。
//! **不做任何数据迁移**：现有文件版本与 v4 不符（或不可读/遗留结构）时，
//! 直接删除整个数据库文件（含 -wal/-shm）并重建。

use std::path::Path;

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

/// 打开数据库
///
/// 现有文件版本与 v4 不符（或不可读）时删除整个文件并重建——不做数据迁移；
/// 版本相符时原样使用（跨重启正常持久化）。
pub fn open_or_recreate(db_path: &Path) -> AppResult<Connection> {
    if db_path.exists() && !is_current_version(db_path) {
        log::warn!(
            "数据库文件版本不符或不可读，删除重建: {}",
            db_path.display()
        );
        remove_db_files(db_path)?;
    }
    let conn = Connection::open(db_path)?;
    initialize(&conn)?;
    Ok(conn)
}

/// 检查数据库文件是否已处于当前版本（任何失败都视为不符）
fn is_current_version(db_path: &Path) -> bool {
    Connection::open(db_path)
        .ok()
        .and_then(|conn| {
            conn.query_row("SELECT version FROM schema_info LIMIT 1", [], |r| {
                r.get::<_, i32>(0)
            })
            .ok()
        })
        .is_some_and(|v| v == SCHEMA_VERSION)
}

/// 删除数据库文件及其 WAL/SHM 附属文件
fn remove_db_files(db_path: &Path) -> AppResult<()> {
    let mut targets = vec![db_path.to_path_buf()];
    for suffix in ["-wal", "-shm"] {
        let mut name = db_path.as_os_str().to_os_string();
        name.push(suffix);
        targets.push(name.into());
    }
    for target in targets {
        if target.exists() {
            std::fs::remove_file(&target)?;
        }
    }
    Ok(())
}

/// 应用 v4 表结构（幂等；缺失时补写版本行）
///
/// 供新文件与内存测试库使用
pub fn initialize(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA foreign_keys = ON;
         PRAGMA busy_timeout = 5000;",
    )?;
    conn.execute_batch(DDL)?;
    conn.execute(
        "INSERT INTO schema_info (version)
         SELECT ?1 WHERE NOT EXISTS (SELECT 1 FROM schema_info)",
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
        // 再次初始化不丢数据（幂等）
        initialize(&conn).unwrap();
        let count: i32 = conn
            .query_row("SELECT COUNT(*) FROM tasks", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn recreates_file_when_legacy_schema_info_lacks_version_column() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("streamgrab.db");

        // 历史开发版本遗留：schema_info 没有 version 列
        let legacy = Connection::open(&db_path).unwrap();
        legacy
            .execute_batch(
                "CREATE TABLE schema_info (id INTEGER PRIMARY KEY, schema_version INTEGER);
                 INSERT INTO schema_info (schema_version) VALUES (3);
                 CREATE TABLE tasks (id TEXT PRIMARY KEY, url TEXT);",
            )
            .unwrap();
        drop(legacy);

        let conn = open_or_recreate(&db_path).unwrap();
        let version: i32 = conn
            .query_row("SELECT version FROM schema_info LIMIT 1", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version, SCHEMA_VERSION);
        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='task_presets'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn recreates_file_when_version_mismatch() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("streamgrab.db");

        let old = Connection::open(&db_path).unwrap();
        old.execute_batch(
            "CREATE TABLE schema_info (version INTEGER NOT NULL);
             INSERT INTO schema_info (version) VALUES (3);
             CREATE TABLE tasks (id TEXT PRIMARY KEY, url TEXT);
             INSERT INTO tasks (id, url) VALUES ('legacy', 'u');",
        )
        .unwrap();
        drop(old);

        let conn = open_or_recreate(&db_path).unwrap();
        // 旧数据不保留
        let count: i32 = conn
            .query_row("SELECT COUNT(*) FROM tasks", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
        let version: i32 = conn
            .query_row("SELECT version FROM schema_info LIMIT 1", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version, SCHEMA_VERSION);
    }

    #[test]
    fn preserves_data_when_version_current() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("streamgrab.db");

        {
            let conn = open_or_recreate(&db_path).unwrap();
            conn.execute("INSERT INTO tasks (id, url, file_name, save_dir, status, created_at, updated_at) VALUES ('t1','u','f','d','pending','now','now')", []).unwrap();
        }

        // 重新打开：版本相符，数据保留
        let conn = open_or_recreate(&db_path).unwrap();
        let count: i32 = conn
            .query_row("SELECT COUNT(*) FROM tasks", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }
}
