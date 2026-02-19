//! 数据库 Schema 定义
//!
//! 采用多表结构化设计，便于维护和扩展

use rusqlite::Connection;

/// 数据库文件名
pub const DB_FILE_NAME: &str = "streamgrab.db";

/// 当前 Schema 版本
pub const SCHEMA_VERSION: i32 = 2;

/// 初始化数据库
pub fn initialize_database(conn: &Connection) -> Result<(), String> {
    create_tables(conn)?;
    set_schema_version(conn)?;
    Ok(())
}

/// 设置 Schema 版本
fn set_schema_version(conn: &Connection) -> Result<(), String> {
    conn.execute(
        "INSERT OR REPLACE INTO schema_info (key, value) VALUES ('version', ?1)",
        [SCHEMA_VERSION.to_string()],
    )
    .map_err(|e| format!("Failed to set schema version: {}", e))?;
    Ok(())
}

/// 创建所有数据表
fn create_tables(conn: &Connection) -> Result<(), String> {
    // Schema 版本信息表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_info (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )
    .map_err(|e| format!("Failed to create schema_info table: {}", e))?;

    // ========================================
    // 任务表 - 核心任务信息
    // ========================================
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tasks (
            id TEXT PRIMARY KEY,
            url TEXT NOT NULL,
            file_name TEXT NOT NULL,
            save_dir TEXT NOT NULL,
            output_path TEXT,
            status TEXT NOT NULL DEFAULT 'pending'
                CHECK (status IN ('pending', 'analyzing', 'downloading', 'paused', 'merging', 'muxing', 'completed', 'failed', 'cancelled')),
            error TEXT,
            was_interrupted INTEGER DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            started_at TEXT,
            completed_at TEXT
        )",
        [],
    )
    .map_err(|e| format!("Failed to create tasks table: {}", e))?;

    // ========================================
    // 任务进度表 - 下载进度信息
    // ========================================
    conn.execute(
        "CREATE TABLE IF NOT EXISTS task_progress (
            task_id TEXT PRIMARY KEY REFERENCES tasks(id) ON DELETE CASCADE,
            percent INTEGER DEFAULT 0,
            speed INTEGER DEFAULT 0,
            downloaded_size INTEGER DEFAULT 0,
            total_size INTEGER DEFAULT 0,
            downloaded_segments INTEGER DEFAULT 0,
            total_segments INTEGER DEFAULT 0,
            eta INTEGER DEFAULT 0,
            current_action TEXT DEFAULT '',
            updated_at TEXT NOT NULL
        )",
        [],
    )
    .map_err(|e| format!("Failed to create task_progress table: {}", e))?;

    // ========================================
    // 进度历史表 - 用于绘制进度曲线图
    // ========================================
    conn.execute(
        "CREATE TABLE IF NOT EXISTS progress_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
            timestamp TEXT NOT NULL,
            percent INTEGER NOT NULL,
            speed INTEGER NOT NULL,
            downloaded_size INTEGER NOT NULL
        )",
        [],
    )
    .map_err(|e| format!("Failed to create progress_history table: {}", e))?;

    // ========================================
    // 媒体信息表 - 视频音频信息
    // ========================================
    conn.execute(
        "CREATE TABLE IF NOT EXISTS task_media_info (
            task_id TEXT PRIMARY KEY REFERENCES tasks(id) ON DELETE CASCADE,
            resolution TEXT,
            width INTEGER,
            height INTEGER,
            frame_rate REAL,
            video_codec TEXT,
            video_range TEXT,
            audio_codec TEXT,
            audio_channels TEXT,
            audio_language TEXT,
            duration REAL,
            segment_count INTEGER,
            is_live INTEGER DEFAULT 0,
            is_encrypted INTEGER DEFAULT 0,
            file_format TEXT
        )",
        [],
    )
    .map_err(|e| format!("Failed to create task_media_info table: {}", e))?;

    // ========================================
    // 任务配置表 - 下载配置
    // ========================================
    conn.execute(
        "CREATE TABLE IF NOT EXISTS task_config (
            task_id TEXT PRIMARY KEY REFERENCES tasks(id) ON DELETE CASCADE,
            thread_count INTEGER DEFAULT 16,
            retry_count INTEGER DEFAULT 3,
            timeout INTEGER DEFAULT 30,
            max_speed TEXT DEFAULT '',
            auto_select INTEGER DEFAULT 1,
            select_video TEXT,
            select_audio TEXT,
            select_subtitle TEXT,
            drop_video TEXT,
            drop_audio TEXT,
            drop_subtitle TEXT,
            mux_format TEXT DEFAULT 'mp4',
            mux_after_done INTEGER DEFAULT 1,
            skip_merge INTEGER DEFAULT 0,
            del_after_done INTEGER DEFAULT 0,
            check_segments_count INTEGER DEFAULT 1,
            custom_range TEXT,
            key TEXT,
            proxy TEXT
        )",
        [],
    )
    .map_err(|e| format!("Failed to create task_config table: {}", e))?;

    // ========================================
    // 任务请求头表 - 自定义 Headers
    // ========================================
    conn.execute(
        "CREATE TABLE IF NOT EXISTS task_headers (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            value TEXT NOT NULL,
            enabled INTEGER DEFAULT 1
        )",
        [],
    )
    .map_err(|e| format!("Failed to create task_headers table: {}", e))?;

    // ========================================
    // 配置表 - 应用设置
    // ========================================
    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            module TEXT PRIMARY KEY,
            data TEXT NOT NULL
        )",
        [],
    )
    .map_err(|e| format!("Failed to create settings table: {}", e))?;

    // ========================================
    // 索引
    // ========================================
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

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_task_headers_task_id ON task_headers(task_id)",
        [],
    )
    .map_err(|e| format!("Failed to create task_headers index: {}", e))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_progress_history_task_id ON progress_history(task_id, timestamp)",
        [],
    )
    .map_err(|e| format!("Failed to create progress_history index: {}", e))?;

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
