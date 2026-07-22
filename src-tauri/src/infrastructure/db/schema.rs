//! 数据库 Schema（v4）
//!
//! v4 采用单表聚合模型（tasks 含三个 JSON 列）+ 通用工具配置表。
//! 版本不符时先备份旧文件（`.bak.<timestamp>`）再重建空库——用户数据不丢失。

use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::shared::{AppError, AppResult};

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
/// 现有文件版本与 v4 不符（或不可读）时，先将旧文件重命名为带时间戳的备份
/// （`streamgrab.db.bak.<YYYYMMDD_HHMMSS>`），再重建空库。用户数据不会丢失。
/// 版本相符时原样使用（跨重启正常持久化）。
pub fn open_or_recreate(db_path: &Path) -> AppResult<Connection> {
    if db_path.exists() && !is_current_version(db_path) {
        let backup_path = backup_db_files(db_path)?;
        log::warn!(
            "数据库版本不符或不可读，已备份到 {} 并重建空库: {}",
            backup_path.display(),
            db_path.display()
        );
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

/// 将数据库文件及其 WAL/SHM 附属文件重命名为带时间戳的备份，返回主备份路径
///
/// 备份命名: `streamgrab.db.bak.20260723_001500`
fn backup_db_files(db_path: &Path) -> AppResult<PathBuf> {
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let backup_name = format!(
        "{}.bak.{timestamp}",
        db_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("streamgrab.db")
    );
    let backup_path = db_path.with_file_name(&backup_name);

    // 主文件必须存在才需要备份
    if db_path.exists() {
        std::fs::rename(db_path, &backup_path).map_err(|e| {
            AppError::database(format!(
                "备份数据库失败 ({} → {}): {e}",
                db_path.display(),
                backup_path.display()
            ))
        })?;
    }

    // WAL/SHM 附属文件一并重命名（可能不存在）
    for suffix in ["-wal", "-shm"] {
        let mut src_name = db_path.as_os_str().to_os_string();
        src_name.push(suffix);
        let src: PathBuf = src_name.into();
        if src.exists() {
            let mut dst_name = backup_path.as_os_str().to_os_string();
            dst_name.push(suffix);
            let dst: PathBuf = dst_name.into();
            let _ = std::fs::rename(&src, &dst); // 附属文件失败不阻塞
        }
    }

    Ok(backup_path)
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
    fn backs_up_when_legacy_schema_info_lacks_version_column() {
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
    fn backs_up_and_recreates_on_version_mismatch() {
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
        // 新库无旧数据
        let count: i32 = conn
            .query_row("SELECT COUNT(*) FROM tasks", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
        let version: i32 = conn
            .query_row("SELECT version FROM schema_info LIMIT 1", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version, SCHEMA_VERSION);

        // 备份文件存在且包含旧数据
        let backups: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains(".bak."))
            .collect();
        assert_eq!(backups.len(), 1, "应恰好有一个备份文件");

        let backup_conn = Connection::open(backups[0].path()).unwrap();
        let old_count: i32 = backup_conn
            .query_row("SELECT COUNT(*) FROM tasks", [], |r| r.get(0))
            .unwrap();
        assert_eq!(old_count, 1, "备份应保留旧数据");
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
