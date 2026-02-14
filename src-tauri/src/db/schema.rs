//! 数据库 Schema 定义
//!
//! 统一使用 SQLite 存储所有应用数据

use rusqlite::Connection;

/// 数据库文件名
pub const DB_FILE_NAME: &str = "streamgrab.db";

/// 初始化数据库
///
/// 创建所有表
pub fn initialize_database(conn: &Connection) -> Result<(), String> {
    create_tables(conn)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_create_tables() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = Connection::open(&db_path).unwrap();

        let result = initialize_database(&conn);
        assert!(result.is_ok());
    }
}
