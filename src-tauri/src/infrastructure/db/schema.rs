//! 数据库 Schema 定义
//!
//! 采用结构化设计，每个配置项对应独立列

use rusqlite::Connection;

/// 当前 Schema 版本
pub const SCHEMA_VERSION: i32 = 3;

/// 初始化数据库
pub fn initialize_database(conn: &Connection) -> Result<(), String> {
    create_tables(conn)?;
    migrate_schema(conn)?;
    set_schema_version(conn)?;
    initialize_default_configs(conn)?;
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

/// 获取当前 Schema 版本
pub fn get_schema_version(conn: &Connection) -> i32 {
    conn.query_row(
        "SELECT value FROM schema_info WHERE key = 'version'",
        [],
        |row| row.get::<_, String>(0),
    )
    .ok()
    .and_then(|v| v.parse().ok())
    .unwrap_or(0)
}

/// 数据库迁移
fn migrate_schema(conn: &Connection) -> Result<(), String> {
    let version = get_schema_version(conn);

    // 版本 2 -> 3: 重构配置系统
    if version < 3 {
        log::info!("Migrating database schema from version {} to 3", version);
        // 迁移逻辑在 create_tables 中处理（使用 IF NOT EXISTS）
    }

    Ok(())
}

/// 初始化默认配置
fn initialize_default_configs(conn: &Connection) -> Result<(), String> {
    // 初始化应用配置（单例，id = 1）
    conn.execute("INSERT OR IGNORE INTO app_settings (id) VALUES (1)", [])
        .map_err(|e| format!("Failed to initialize app_settings: {}", e))?;

    // 初始化 M3U8DL 配置（单例，id = 1）
    conn.execute("INSERT OR IGNORE INTO m3u8dl_settings (id) VALUES (1)", [])
        .map_err(|e| format!("Failed to initialize m3u8dl_settings: {}", e))?;

    // 初始化 FFmpeg 配置（单例，id = 1）
    conn.execute("INSERT OR IGNORE INTO ffmpeg_settings (id) VALUES (1)", [])
        .map_err(|e| format!("Failed to initialize ffmpeg_settings: {}", e))?;

    // 初始化网络配置（单例，id = 1）
    conn.execute("INSERT OR IGNORE INTO network_settings (id) VALUES (1)", [])
        .map_err(|e| format!("Failed to initialize network_settings: {}", e))?;

    // 初始化解密配置（单例，id = 1）
    conn.execute(
        "INSERT OR IGNORE INTO decryption_settings (id) VALUES (1)",
        [],
    )
    .map_err(|e| format!("Failed to initialize decryption_settings: {}", e))?;

    log::info!("Default configurations initialized");
    Ok(())
}

/// 创建所有数据表
fn create_tables(conn: &Connection) -> Result<(), String> {
    // ========================================
    // Schema 版本信息表
    // ========================================
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_info (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )
    .map_err(|e| format!("Failed to create schema_info table: {}", e))?;

    // ========================================
    // 应用配置表 - 软件本身的行为设置
    // ========================================
    conn.execute(
        "CREATE TABLE IF NOT EXISTS app_settings (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            language TEXT NOT NULL DEFAULT 'zh-CN',
            auto_start_download INTEGER NOT NULL DEFAULT 1,
            minimize_to_tray INTEGER NOT NULL DEFAULT 0,
            check_update INTEGER NOT NULL DEFAULT 1,
            default_save_dir TEXT NOT NULL DEFAULT '',
            default_tmp_dir TEXT NOT NULL DEFAULT '',
            theme TEXT NOT NULL DEFAULT 'dark',
            show_notification INTEGER NOT NULL DEFAULT 1,
            clipboard_watch INTEGER NOT NULL DEFAULT 0,
            log_level TEXT NOT NULL DEFAULT 'INFO',
            log_file_path TEXT NOT NULL DEFAULT '',
            no_log INTEGER NOT NULL DEFAULT 0,
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )
    .map_err(|e| format!("Failed to create app_settings table: {}", e))?;

    // ========================================
    // M3U8DL 配置表 - 流媒体下载专用
    // ========================================
    conn.execute(
        "CREATE TABLE IF NOT EXISTS m3u8dl_settings (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            n_m3u8dl_path TEXT NOT NULL DEFAULT '',
            thread_count INTEGER NOT NULL DEFAULT 8,
            retry_count INTEGER NOT NULL DEFAULT 3,
            timeout INTEGER NOT NULL DEFAULT 100,
            max_speed TEXT NOT NULL DEFAULT '0',
            auto_select INTEGER NOT NULL DEFAULT 1,
            select_video TEXT,
            select_audio TEXT,
            select_subtitle TEXT,
            drop_video TEXT,
            drop_audio TEXT,
            drop_subtitle TEXT,
            check_segments_count INTEGER NOT NULL DEFAULT 1,
            del_after_done INTEGER NOT NULL DEFAULT 1,
            skip_merge INTEGER NOT NULL DEFAULT 0,
            write_meta_json INTEGER NOT NULL DEFAULT 0,
            binary_merge INTEGER NOT NULL DEFAULT 0,
            concurrent_download INTEGER NOT NULL DEFAULT 0,
            mux_format TEXT NOT NULL DEFAULT 'mp4',
            muxer TEXT NOT NULL DEFAULT 'ffmpeg',
            mux_bin_path TEXT,
            mux_skip_subtitles INTEGER NOT NULL DEFAULT 0,
            mux_keep_original INTEGER NOT NULL DEFAULT 0,
            sub_only INTEGER NOT NULL DEFAULT 0,
            sub_format TEXT NOT NULL DEFAULT 'SRT',
            auto_subtitle_fix INTEGER NOT NULL DEFAULT 1,
            live_perform_as_vod INTEGER NOT NULL DEFAULT 0,
            live_real_time_merge INTEGER NOT NULL DEFAULT 0,
            live_keep_segments INTEGER NOT NULL DEFAULT 1,
            live_pipe_mux INTEGER NOT NULL DEFAULT 0,
            live_fix_vtt_by_audio INTEGER NOT NULL DEFAULT 0,
            live_record_limit TEXT,
            live_wait_time INTEGER NOT NULL DEFAULT 0,
            live_take_count INTEGER NOT NULL DEFAULT 16,
            allow_hls_multi_ext_map INTEGER NOT NULL DEFAULT 0,
            url_processor_args TEXT,
            no_date_info INTEGER NOT NULL DEFAULT 0,
            use_ffmpeg_concat_demuxer INTEGER NOT NULL DEFAULT 0,
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )
    .map_err(|e| format!("Failed to create m3u8dl_settings table: {}", e))?;

    // ========================================
    // FFmpeg 配置表 - 直链下载专用
    // ========================================
    conn.execute(
        "CREATE TABLE IF NOT EXISTS ffmpeg_settings (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            ffmpeg_path TEXT NOT NULL DEFAULT '',
            ffprobe_path TEXT NOT NULL DEFAULT '',
            retry_count INTEGER NOT NULL DEFAULT 3,
            timeout INTEGER NOT NULL DEFAULT 60,
            max_speed TEXT NOT NULL DEFAULT '0',
            connection_timeout INTEGER NOT NULL DEFAULT 30,
            reconnect_attempts INTEGER NOT NULL DEFAULT 3,
            reconnect_delay INTEGER NOT NULL DEFAULT 5,
            overwrite_existing INTEGER NOT NULL DEFAULT 0,
            preserve_timestamps INTEGER NOT NULL DEFAULT 1,
            user_agent TEXT,
            referer TEXT,
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )
    .map_err(|e| format!("Failed to create ffmpeg_settings table: {}", e))?;

    // ========================================
    // 网络配置表 - 共用网络设置
    // ========================================
    conn.execute(
        "CREATE TABLE IF NOT EXISTS network_settings (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            use_system_proxy INTEGER NOT NULL DEFAULT 1,
            custom_proxy TEXT,
            base_url TEXT,
            append_url_params INTEGER NOT NULL DEFAULT 0,
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )
    .map_err(|e| format!("Failed to create network_settings table: {}", e))?;

    // 网络请求头表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS network_headers (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            value TEXT NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 1,
            sort_order INTEGER NOT NULL DEFAULT 0
        )",
        [],
    )
    .map_err(|e| format!("Failed to create network_headers table: {}", e))?;

    // ========================================
    // 解密配置表 - 共用解密设置
    // ========================================
    conn.execute(
        "CREATE TABLE IF NOT EXISTS decryption_settings (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            key_text_file TEXT,
            decryption_engine TEXT NOT NULL DEFAULT 'MP4DECRYPT',
            decryption_bin_path TEXT,
            real_time_decryption INTEGER NOT NULL DEFAULT 0,
            custom_hls_enabled INTEGER NOT NULL DEFAULT 0,
            custom_hls_method TEXT NOT NULL DEFAULT 'UNKNOWN',
            custom_hls_key_type TEXT NOT NULL DEFAULT 'hex',
            custom_hls_key_value TEXT,
            custom_hls_iv_type TEXT NOT NULL DEFAULT 'hex',
            custom_hls_iv_value TEXT,
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )
    .map_err(|e| format!("Failed to create decryption_settings table: {}", e))?;

    // 解密密钥表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS decryption_keys (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            kid TEXT,
            key TEXT NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0
        )",
        [],
    )
    .map_err(|e| format!("Failed to create decryption_keys table: {}", e))?;

    // ========================================
    // 广告过滤关键字表
    // ========================================
    conn.execute(
        "CREATE TABLE IF NOT EXISTS ad_filter_keywords (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            keyword TEXT NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 1,
            sort_order INTEGER NOT NULL DEFAULT 0
        )",
        [],
    )
    .map_err(|e| format!("Failed to create ad_filter_keywords table: {}", e))?;

    // ========================================
    // 混流导入表
    // ========================================
    conn.execute(
        "CREATE TABLE IF NOT EXISTS mux_imports (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT NOT NULL,
            lang TEXT,
            name TEXT,
            sort_order INTEGER NOT NULL DEFAULT 0
        )",
        [],
    )
    .map_err(|e| format!("Failed to create mux_imports table: {}", e))?;

    // ========================================
    // 配置模板表
    // ========================================
    conn.execute(
        "CREATE TABLE IF NOT EXISTS config_templates (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            is_preset INTEGER NOT NULL DEFAULT 0,
            downloader_type TEXT NOT NULL DEFAULT 'm3u8dl',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )",
        [],
    )
    .map_err(|e| format!("Failed to create config_templates table: {}", e))?;

    // 模板 M3U8DL 覆盖配置
    conn.execute(
        "CREATE TABLE IF NOT EXISTS template_m3u8dl_overrides (
            template_id TEXT PRIMARY KEY,
            n_m3u8dl_path TEXT,
            thread_count INTEGER,
            retry_count INTEGER,
            timeout INTEGER,
            max_speed TEXT,
            auto_select INTEGER,
            select_video TEXT,
            select_audio TEXT,
            select_subtitle TEXT,
            drop_video TEXT,
            drop_audio TEXT,
            drop_subtitle TEXT,
            check_segments_count INTEGER,
            del_after_done INTEGER,
            skip_merge INTEGER,
            write_meta_json INTEGER,
            binary_merge INTEGER,
            concurrent_download INTEGER,
            mux_format TEXT,
            muxer TEXT,
            mux_bin_path TEXT,
            mux_skip_subtitles INTEGER,
            mux_keep_original INTEGER,
            sub_only INTEGER,
            sub_format TEXT,
            auto_subtitle_fix INTEGER,
            live_perform_as_vod INTEGER,
            live_real_time_merge INTEGER,
            live_keep_segments INTEGER,
            live_pipe_mux INTEGER,
            live_fix_vtt_by_audio INTEGER,
            live_record_limit TEXT,
            live_wait_time INTEGER,
            live_take_count INTEGER,
            allow_hls_multi_ext_map INTEGER,
            url_processor_args TEXT,
            no_date_info INTEGER,
            use_ffmpeg_concat_demuxer INTEGER,
            FOREIGN KEY (template_id) REFERENCES config_templates(id) ON DELETE CASCADE
        )",
        [],
    )
    .map_err(|e| format!("Failed to create template_m3u8dl_overrides table: {}", e))?;

    // 模板 FFmpeg 覆盖配置
    conn.execute(
        "CREATE TABLE IF NOT EXISTS template_ffmpeg_overrides (
            template_id TEXT PRIMARY KEY,
            ffmpeg_path TEXT,
            ffprobe_path TEXT,
            retry_count INTEGER,
            timeout INTEGER,
            max_speed TEXT,
            connection_timeout INTEGER,
            reconnect_attempts INTEGER,
            reconnect_delay INTEGER,
            overwrite_existing INTEGER,
            preserve_timestamps INTEGER,
            user_agent TEXT,
            referer TEXT,
            FOREIGN KEY (template_id) REFERENCES config_templates(id) ON DELETE CASCADE
        )",
        [],
    )
    .map_err(|e| format!("Failed to create template_ffmpeg_overrides table: {}", e))?;

    // 模板网络配置覆盖
    conn.execute(
        "CREATE TABLE IF NOT EXISTS template_network_overrides (
            template_id TEXT PRIMARY KEY,
            use_system_proxy INTEGER,
            custom_proxy TEXT,
            base_url TEXT,
            append_url_params INTEGER,
            FOREIGN KEY (template_id) REFERENCES config_templates(id) ON DELETE CASCADE
        )",
        [],
    )
    .map_err(|e| format!("Failed to create template_network_overrides table: {}", e))?;

    // 模板解密配置覆盖
    conn.execute(
        "CREATE TABLE IF NOT EXISTS template_decryption_overrides (
            template_id TEXT PRIMARY KEY,
            key_text_file TEXT,
            decryption_engine TEXT,
            decryption_bin_path TEXT,
            real_time_decryption INTEGER,
            custom_hls_enabled INTEGER,
            custom_hls_method TEXT,
            custom_hls_key_type TEXT,
            custom_hls_key_value TEXT,
            custom_hls_iv_type TEXT,
            custom_hls_iv_value TEXT,
            FOREIGN KEY (template_id) REFERENCES config_templates(id) ON DELETE CASCADE
        )",
        [],
    )
    .map_err(|e| {
        format!(
            "Failed to create template_decryption_overrides table: {}",
            e
        )
    })?;

    // 模板专属请求头
    conn.execute(
        "CREATE TABLE IF NOT EXISTS template_network_headers (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            template_id TEXT NOT NULL,
            name TEXT NOT NULL,
            value TEXT NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 1,
            sort_order INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (template_id) REFERENCES config_templates(id) ON DELETE CASCADE
        )",
        [],
    )
    .map_err(|e| format!("Failed to create template_network_headers table: {}", e))?;

    // 模板专属解密密钥
    conn.execute(
        "CREATE TABLE IF NOT EXISTS template_decryption_keys (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            template_id TEXT NOT NULL,
            kid TEXT,
            key TEXT NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (template_id) REFERENCES config_templates(id) ON DELETE CASCADE
        )",
        [],
    )
    .map_err(|e| format!("Failed to create template_decryption_keys table: {}", e))?;

    // 模板专属广告过滤关键字
    conn.execute(
        "CREATE TABLE IF NOT EXISTS template_ad_filter_keywords (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            template_id TEXT NOT NULL,
            keyword TEXT NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 1,
            sort_order INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (template_id) REFERENCES config_templates(id) ON DELETE CASCADE
        )",
        [],
    )
    .map_err(|e| format!("Failed to create template_ad_filter_keywords table: {}", e))?;

    // 模板专属混流导入
    conn.execute(
        "CREATE TABLE IF NOT EXISTS template_mux_imports (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            template_id TEXT NOT NULL,
            path TEXT NOT NULL,
            lang TEXT,
            name TEXT,
            sort_order INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (template_id) REFERENCES config_templates(id) ON DELETE CASCADE
        )",
        [],
    )
    .map_err(|e| format!("Failed to create template_mux_imports table: {}", e))?;

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
            downloader_type TEXT NOT NULL DEFAULT 'm3u8dl',
            template_id TEXT REFERENCES config_templates(id),
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
    // 任务 M3U8DL 配置表 - 任务级配置覆盖
    // ========================================
    conn.execute(
        "CREATE TABLE IF NOT EXISTS task_m3u8dl_config (
            task_id TEXT PRIMARY KEY REFERENCES tasks(id) ON DELETE CASCADE,
            save_dir TEXT,
            save_name TEXT,
            save_pattern TEXT,
            thread_count INTEGER,
            retry_count INTEGER,
            timeout INTEGER,
            max_speed TEXT,
            auto_select INTEGER,
            select_video TEXT,
            select_audio TEXT,
            select_subtitle TEXT,
            drop_video TEXT,
            drop_audio TEXT,
            drop_subtitle TEXT,
            mux_format TEXT,
            mux_after_done INTEGER,
            skip_merge INTEGER,
            del_after_done INTEGER,
            check_segments_count INTEGER,
            custom_range TEXT,
            start_at TEXT
        )",
        [],
    )
    .map_err(|e| format!("Failed to create task_m3u8dl_config table: {}", e))?;

    // ========================================
    // 任务 FFmpeg 配置表 - 任务级配置覆盖
    // ========================================
    conn.execute(
        "CREATE TABLE IF NOT EXISTS task_ffmpeg_config (
            task_id TEXT PRIMARY KEY REFERENCES tasks(id) ON DELETE CASCADE,
            save_dir TEXT,
            save_name TEXT,
            retry_count INTEGER,
            timeout INTEGER,
            max_speed TEXT,
            connection_timeout INTEGER,
            reconnect_attempts INTEGER,
            start_at TEXT
        )",
        [],
    )
    .map_err(|e| format!("Failed to create task_ffmpeg_config table: {}", e))?;

    // ========================================
    // 任务网络请求头表
    // ========================================
    conn.execute(
        "CREATE TABLE IF NOT EXISTS task_network_headers (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            value TEXT NOT NULL,
            enabled INTEGER DEFAULT 1
        )",
        [],
    )
    .map_err(|e| format!("Failed to create task_network_headers table: {}", e))?;

    // ========================================
    // 历史记录表（保留旧表结构）
    // ========================================
    conn.execute(
        "CREATE TABLE IF NOT EXISTS history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            url TEXT NOT NULL,
            file_name TEXT NOT NULL,
            save_dir TEXT NOT NULL,
            output_path TEXT,
            file_size INTEGER,
            completed_at TEXT NOT NULL,
            duration REAL,
            resolution TEXT,
            video_codec TEXT,
            audio_codec TEXT
        )",
        [],
    )
    .map_err(|e| format!("Failed to create history table: {}", e))?;

    // ========================================
    // 旧配置表（保留兼容，后续删除）
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
    create_indexes(conn)?;

    log::info!("Database tables created successfully");
    Ok(())
}

/// 创建索引
fn create_indexes(conn: &Connection) -> Result<(), String> {
    let indexes = [
        "CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status)",
        "CREATE INDEX IF NOT EXISTS idx_tasks_created_at ON tasks(created_at DESC)",
        "CREATE INDEX IF NOT EXISTS idx_tasks_downloader_type ON tasks(downloader_type)",
        "CREATE INDEX IF NOT EXISTS idx_task_headers_task_id ON task_network_headers(task_id)",
        "CREATE INDEX IF NOT EXISTS idx_progress_history_task_id ON progress_history(task_id, timestamp)",
        "CREATE INDEX IF NOT EXISTS idx_network_headers_sort ON network_headers(sort_order)",
        "CREATE INDEX IF NOT EXISTS idx_decryption_keys_sort ON decryption_keys(sort_order)",
        "CREATE INDEX IF NOT EXISTS idx_ad_filter_keywords_sort ON ad_filter_keywords(sort_order)",
        "CREATE INDEX IF NOT EXISTS idx_mux_imports_sort ON mux_imports(sort_order)",
        "CREATE INDEX IF NOT EXISTS idx_config_templates_type ON config_templates(downloader_type)",
        "CREATE INDEX IF NOT EXISTS idx_template_network_headers ON template_network_headers(template_id, sort_order)",
        "CREATE INDEX IF NOT EXISTS idx_template_decryption_keys ON template_decryption_keys(template_id, sort_order)",
        "CREATE INDEX IF NOT EXISTS idx_template_ad_filter ON template_ad_filter_keywords(template_id, sort_order)",
        "CREATE INDEX IF NOT EXISTS idx_template_mux_imports ON template_mux_imports(template_id, sort_order)",
        "CREATE INDEX IF NOT EXISTS idx_history_completed_at ON history(completed_at DESC)",
    ];

    for idx in indexes {
        conn.execute(idx, [])
            .map_err(|e| format!("Failed to create index: {}", e))?;
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

        let result = initialize_database(&conn);
        assert!(result.is_ok());
        assert_eq!(get_schema_version(&conn), SCHEMA_VERSION);
    }
}
