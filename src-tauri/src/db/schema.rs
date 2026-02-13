//! 数据库 Schema 定义和迁移
//!
//! 统一使用 SQLite 存储所有应用数据

use rusqlite::Connection;
use std::path::PathBuf;
use std::fs;

/// 数据库文件名
pub const DB_FILE_NAME: &str = "streamgrab.db";

/// 旧数据库文件名（用于迁移）
pub const OLD_DB_FILE_NAME: &str = "history.db";

/// 旧配置文件名（用于迁移）
pub const OLD_CONFIG_FILE_NAME: &str = "settings.json";

/// 初始化数据库
///
/// 创建所有表，执行数据迁移
pub fn initialize_database(conn: &Connection, app_config_dir: &PathBuf) -> Result<(), String> {
    // 创建所有表
    create_tables(conn)?;

    // 执行数据迁移
    migrate_from_legacy(conn, app_config_dir)?;

    Ok(())
}

/// 创建所有数据表
fn create_tables(conn: &Connection) -> Result<(), String> {
    // 配置表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )
    .map_err(|e| format!("Failed to create settings table: {}", e))?;

    // 密钥库表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS keys (
            id TEXT PRIMARY KEY,
            kid TEXT,
            key TEXT NOT NULL,
            name TEXT,
            created_at TEXT NOT NULL,
            last_used_at TEXT
        )",
        [],
    )
    .map_err(|e| format!("Failed to create keys table: {}", e))?;

    // 任务表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tasks (
            id TEXT PRIMARY KEY,
            url TEXT NOT NULL,
            file_name TEXT NOT NULL,
            save_dir TEXT NOT NULL,
            output_path TEXT,
            status TEXT NOT NULL DEFAULT 'pending',
            error TEXT,
            progress_json TEXT NOT NULL DEFAULT '{}',
            config_json TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            started_at TEXT,
            completed_at TEXT,
            was_interrupted INTEGER DEFAULT 0
        )",
        [],
    )
    .map_err(|e| format!("Failed to create tasks table: {}", e))?;

    // 任务表索引
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status)",
        [],
    )
    .map_err(|e| format!("Failed to create tasks status index: {}", e))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_tasks_created_at ON tasks(created_at DESC)",
        [],
    )
    .map_err(|e| format!("Failed to create tasks created_at index: {}", e))?;

    // 历史记录表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS history (
            id TEXT PRIMARY KEY,
            url TEXT NOT NULL,
            file_name TEXT NOT NULL,
            save_path TEXT NOT NULL,
            file_size INTEGER NOT NULL,
            duration REAL NOT NULL,
            completed_at TEXT NOT NULL,
            task_id TEXT REFERENCES tasks(id) ON DELETE SET NULL
        )",
        [],
    )
    .map_err(|e| format!("Failed to create history table: {}", e))?;

    // 历史记录索引
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_history_completed_at ON history(completed_at DESC)",
        [],
    )
    .map_err(|e| format!("Failed to create history completed_at index: {}", e))?;

    // 配置模板表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS config_templates (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            settings_json TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )",
        [],
    )
    .map_err(|e| format!("Failed to create config_templates table: {}", e))?;

    // 定时任务表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS scheduled_tasks (
            id TEXT PRIMARY KEY,
            task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
            scheduled_time TEXT NOT NULL,
            repeat TEXT NOT NULL DEFAULT 'none',
            enabled INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL
        )",
        [],
    )
    .map_err(|e| format!("Failed to create scheduled_tasks table: {}", e))?;

    // 定时任务索引
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_scheduled_tasks_time ON scheduled_tasks(scheduled_time)",
        [],
    )
    .map_err(|e| format!("Failed to create scheduled_tasks time index: {}", e))?;

    log::info!("Database tables created successfully");
    Ok(())
}

/// 从旧版本迁移数据
fn migrate_from_legacy(conn: &Connection, app_config_dir: &PathBuf) -> Result<(), String> {
    let old_db_path = app_config_dir.join(OLD_DB_FILE_NAME);
    let old_config_path = app_config_dir.join(OLD_CONFIG_FILE_NAME);
    let new_db_path = app_config_dir.join(DB_FILE_NAME);

    // 迁移旧的历史数据库
    if old_db_path.exists() && old_db_path != new_db_path {
        migrate_history_db(conn, &old_db_path)?;
    }

    // 迁移旧的 JSON 配置
    if old_config_path.exists() {
        migrate_settings_json(conn, &old_config_path)?;
    }

    Ok(())
}

/// 迁移旧的 history.db 数据
fn migrate_history_db(conn: &Connection, old_db_path: &PathBuf) -> Result<(), String> {
    log::info!("Migrating history from {:?}", old_db_path);

    let old_conn = Connection::open(old_db_path)
        .map_err(|e| format!("Failed to open old history database: {}", e))?;

    // 读取旧数据
    let mut stmt = old_conn
        .prepare(
            "SELECT id, url, file_name, save_path, file_size, duration, completed_at FROM history",
        )
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let records = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, f64>(5)?,
                row.get::<_, String>(6)?,
            ))
        })
        .map_err(|e| format!("Failed to query old history: {}", e))?;

    let mut count = 0;
    for record in records {
        let (id, url, file_name, save_path, file_size, duration, completed_at) =
            record.map_err(|e| format!("Failed to read record: {}", e))?;

        // 插入到新数据库（忽略重复）
        let result = conn.execute(
            "INSERT OR IGNORE INTO history (id, url, file_name, save_path, file_size, duration, completed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![id, url, file_name, save_path, file_size, duration, completed_at],
        );

        if let Ok(_) = result {
            count += 1;
        }
    }

    log::info!("Migrated {} history records", count);

    // 重命名旧文件为备份
    let backup_path = old_db_path.with_extension("db.bak");
    if let Err(e) = fs::rename(old_db_path, &backup_path) {
        log::warn!("Failed to rename old history.db: {}", e);
    } else {
        log::info!("Renamed old history.db to {:?}", backup_path);
    }

    Ok(())
}

/// 迁移旧的 settings.json 配置
fn migrate_settings_json(conn: &Connection, old_config_path: &PathBuf) -> Result<(), String> {
    log::info!("Migrating settings from {:?}", old_config_path);

    let content = fs::read_to_string(old_config_path)
        .map_err(|e| format!("Failed to read settings.json: {}", e))?;

    let settings: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse settings.json: {}", e))?;

    // 遍历所有配置模块并插入
    if let serde_json::Value::Object(map) = settings {
        let mut count = 0;
        for (key, value) in map {
            let value_str = serde_json::to_string(&value).unwrap_or_else(|_| "{}".to_string());

            let result = conn.execute(
                "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
                rusqlite::params![key, value_str],
            );

            if let Ok(_) = result {
                count += 1;
            }
        }
        log::info!("Migrated {} settings sections", count);
    }

    // 重命名旧文件为备份
    let backup_path = old_config_path.with_extension("json.bak");
    if let Err(e) = fs::rename(old_config_path, &backup_path) {
        log::warn!("Failed to rename old settings.json: {}", e);
    } else {
        log::info!("Renamed old settings.json to {:?}", backup_path);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_create_tables() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = Connection::open(&db_path).unwrap();

        let result = initialize_database(&conn, &dir.path().to_path_buf());
        assert!(result.is_ok());
    }
}
