//! 配置仓库
//!
//! 提供配置数据的 CRUD 操作，支持字段级别更新

use rusqlite::{params, Connection};
use std::sync::Mutex;

// ========================================
// 应用配置
// ========================================

/// 应用配置实体
#[derive(Debug, Clone)]
pub struct AppSettings {
    pub language: String,
    pub auto_start_download: bool,
    pub minimize_to_tray: bool,
    pub check_update: bool,
    pub default_save_dir: String,
    pub default_tmp_dir: String,
    pub theme: String,
    pub show_notification: bool,
    pub clipboard_watch: bool,
    pub log_level: String,
    pub log_file_path: String,
    pub no_log: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: "zh-CN".to_string(),
            auto_start_download: true,
            minimize_to_tray: false,
            check_update: true,
            default_save_dir: String::new(),
            default_tmp_dir: String::new(),
            theme: "dark".to_string(),
            show_notification: true,
            clipboard_watch: false,
            log_level: "INFO".to_string(),
            log_file_path: String::new(),
            no_log: false,
        }
    }
}

// ========================================
// M3U8DL 配置
// ========================================

/// M3U8DL 配置实体
#[derive(Debug, Clone, Default)]
pub struct M3U8DLSettings {
    pub n_m3u8dl_path: String,
    pub thread_count: i32,
    pub retry_count: i32,
    pub timeout: i32,
    pub max_speed: String,
    pub auto_select: bool,
    pub select_video: Option<String>,
    pub select_audio: Option<String>,
    pub select_subtitle: Option<String>,
    pub drop_video: Option<String>,
    pub drop_audio: Option<String>,
    pub drop_subtitle: Option<String>,
    pub check_segments_count: bool,
    pub del_after_done: bool,
    pub skip_merge: bool,
    pub write_meta_json: bool,
    pub binary_merge: bool,
    pub concurrent_download: bool,
    pub mux_format: String,
    pub muxer: String,
    pub mux_bin_path: Option<String>,
    pub mux_skip_subtitles: bool,
    pub mux_keep_original: bool,
    pub sub_only: bool,
    pub sub_format: String,
    pub auto_subtitle_fix: bool,
    pub live_perform_as_vod: bool,
    pub live_real_time_merge: bool,
    pub live_keep_segments: bool,
    pub live_pipe_mux: bool,
    pub live_fix_vtt_by_audio: bool,
    pub live_record_limit: Option<String>,
    pub live_wait_time: i32,
    pub live_take_count: i32,
    pub allow_hls_multi_ext_map: bool,
    pub url_processor_args: Option<String>,
    pub no_date_info: bool,
    pub use_ffmpeg_concat_demuxer: bool,
}

// ========================================
// FFmpeg 配置
// ========================================

/// FFmpeg 配置实体
#[derive(Debug, Clone, Default)]
pub struct FFmpegSettings {
    pub ffmpeg_path: String,
    pub ffprobe_path: String,
    pub retry_count: i32,
    pub timeout: i32,
    pub max_speed: String,
    pub connection_timeout: i32,
    pub reconnect_attempts: i32,
    pub reconnect_delay: i32,
    pub overwrite_existing: bool,
    pub preserve_timestamps: bool,
    pub user_agent: Option<String>,
    pub referer: Option<String>,
}

// ========================================
// 网络配置
// ========================================

/// 网络配置实体
#[derive(Debug, Clone, Default)]
pub struct NetworkSettings {
    pub use_system_proxy: bool,
    pub custom_proxy: Option<String>,
    pub base_url: Option<String>,
    pub append_url_params: bool,
}

/// 网络请求头
#[derive(Debug, Clone)]
pub struct NetworkHeader {
    pub id: i64,
    pub name: String,
    pub value: String,
    pub enabled: bool,
    pub sort_order: i32,
}

// ========================================
// 解密配置
// ========================================

/// 解密配置实体
#[derive(Debug, Clone, Default)]
pub struct DecryptionSettings {
    pub key_text_file: Option<String>,
    pub decryption_engine: String,
    pub decryption_bin_path: Option<String>,
    pub real_time_decryption: bool,
    pub custom_hls_enabled: bool,
    pub custom_hls_method: String,
    pub custom_hls_key_type: String,
    pub custom_hls_key_value: Option<String>,
    pub custom_hls_iv_type: String,
    pub custom_hls_iv_value: Option<String>,
}

/// 解密密钥
#[derive(Debug, Clone)]
pub struct DecryptionKey {
    pub id: i64,
    pub kid: Option<String>,
    pub key: String,
    pub sort_order: i32,
}

// ========================================
// 配置仓库
// ========================================

/// 配置仓库
pub struct ConfigRepository {
    conn: Mutex<Connection>,
}

impl ConfigRepository {
    /// 创建配置仓库
    pub fn new(conn: Connection) -> Result<Self, String> {
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    // ========================================
    // 应用配置
    // ========================================

    /// 获取应用配置
    pub fn get_app_settings(&self) -> Result<AppSettings, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        conn.query_row(
            "SELECT language, auto_start_download, minimize_to_tray, check_update,
                    default_save_dir, default_tmp_dir, theme, show_notification, clipboard_watch,
                    log_level, log_file_path, no_log
             FROM app_settings WHERE id = 1",
            [],
            |row| {
                Ok(AppSettings {
                    language: row.get(0)?,
                    auto_start_download: row.get::<_, i32>(1)? != 0,
                    minimize_to_tray: row.get::<_, i32>(2)? != 0,
                    check_update: row.get::<_, i32>(3)? != 0,
                    default_save_dir: row.get(4)?,
                    default_tmp_dir: row.get(5)?,
                    theme: row.get(6)?,
                    show_notification: row.get::<_, i32>(7)? != 0,
                    clipboard_watch: row.get::<_, i32>(8)? != 0,
                    log_level: row.get(9)?,
                    log_file_path: row.get(10)?,
                    no_log: row.get::<_, i32>(11)? != 0,
                })
            },
        )
        .map_err(|e| format!("Failed to get app settings: {}", e))
    }

    /// 更新应用配置字段
    pub fn update_app_setting_field(&self, field: &str, value: &str) -> Result<(), String> {
        let allowed_fields = [
            "language",
            "auto_start_download",
            "minimize_to_tray",
            "check_update",
            "default_save_dir",
            "default_tmp_dir",
            "theme",
            "show_notification",
            "clipboard_watch",
            "log_level",
            "log_file_path",
            "no_log",
        ];

        if !allowed_fields.contains(&field) {
            return Err(format!("Invalid field: {}", field));
        }

        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let now = chrono::Utc::now().to_rfc3339();
        let sql = format!(
            "UPDATE app_settings SET {} = ?1, updated_at = ?2 WHERE id = 1",
            field
        );

        conn.execute(&sql, params![value, now])
            .map_err(|e| format!("Failed to update {}: {}", field, e))?;

        Ok(())
    }

    // ========================================
    // M3U8DL 配置
    // ========================================

    /// 获取 M3U8DL 配置
    pub fn get_m3u8dl_settings(&self) -> Result<M3U8DLSettings, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        conn.query_row(
            "SELECT n_m3u8dl_path, thread_count, retry_count, timeout, max_speed,
                    auto_select, select_video, select_audio, select_subtitle,
                    drop_video, drop_audio, drop_subtitle, check_segments_count,
                    del_after_done, skip_merge, write_meta_json, binary_merge,
                    concurrent_download, mux_format, muxer, mux_bin_path,
                    mux_skip_subtitles, mux_keep_original, sub_only, sub_format,
                    auto_subtitle_fix, live_perform_as_vod, live_real_time_merge,
                    live_keep_segments, live_pipe_mux, live_fix_vtt_by_audio,
                    live_record_limit, live_wait_time, live_take_count,
                    allow_hls_multi_ext_map, url_processor_args, no_date_info,
                    use_ffmpeg_concat_demuxer
             FROM m3u8dl_settings WHERE id = 1",
            [],
            |row| {
                Ok(M3U8DLSettings {
                    n_m3u8dl_path: row.get(0)?,
                    thread_count: row.get(1)?,
                    retry_count: row.get(2)?,
                    timeout: row.get(3)?,
                    max_speed: row.get(4)?,
                    auto_select: row.get::<_, i32>(5)? != 0,
                    select_video: row.get(6)?,
                    select_audio: row.get(7)?,
                    select_subtitle: row.get(8)?,
                    drop_video: row.get(9)?,
                    drop_audio: row.get(10)?,
                    drop_subtitle: row.get(11)?,
                    check_segments_count: row.get::<_, i32>(12)? != 0,
                    del_after_done: row.get::<_, i32>(13)? != 0,
                    skip_merge: row.get::<_, i32>(14)? != 0,
                    write_meta_json: row.get::<_, i32>(15)? != 0,
                    binary_merge: row.get::<_, i32>(16)? != 0,
                    concurrent_download: row.get::<_, i32>(17)? != 0,
                    mux_format: row.get(18)?,
                    muxer: row.get(19)?,
                    mux_bin_path: row.get(20)?,
                    mux_skip_subtitles: row.get::<_, i32>(21)? != 0,
                    mux_keep_original: row.get::<_, i32>(22)? != 0,
                    sub_only: row.get::<_, i32>(23)? != 0,
                    sub_format: row.get(24)?,
                    auto_subtitle_fix: row.get::<_, i32>(25)? != 0,
                    live_perform_as_vod: row.get::<_, i32>(26)? != 0,
                    live_real_time_merge: row.get::<_, i32>(27)? != 0,
                    live_keep_segments: row.get::<_, i32>(28)? != 0,
                    live_pipe_mux: row.get::<_, i32>(29)? != 0,
                    live_fix_vtt_by_audio: row.get::<_, i32>(30)? != 0,
                    live_record_limit: row.get(31)?,
                    live_wait_time: row.get(32)?,
                    live_take_count: row.get(33)?,
                    allow_hls_multi_ext_map: row.get::<_, i32>(34)? != 0,
                    url_processor_args: row.get(35)?,
                    no_date_info: row.get::<_, i32>(36)? != 0,
                    use_ffmpeg_concat_demuxer: row.get::<_, i32>(37)? != 0,
                })
            },
        )
        .map_err(|e| format!("Failed to get m3u8dl settings: {}", e))
    }

    /// 更新 M3U8DL 配置字段
    pub fn update_m3u8dl_setting_field(&self, field: &str, value: &str) -> Result<(), String> {
        let allowed_fields = [
            "n_m3u8dl_path",
            "thread_count",
            "retry_count",
            "timeout",
            "max_speed",
            "auto_select",
            "select_video",
            "select_audio",
            "select_subtitle",
            "drop_video",
            "drop_audio",
            "drop_subtitle",
            "check_segments_count",
            "del_after_done",
            "skip_merge",
            "write_meta_json",
            "binary_merge",
            "concurrent_download",
            "mux_format",
            "muxer",
            "mux_bin_path",
            "mux_skip_subtitles",
            "mux_keep_original",
            "sub_only",
            "sub_format",
            "auto_subtitle_fix",
            "live_perform_as_vod",
            "live_real_time_merge",
            "live_keep_segments",
            "live_pipe_mux",
            "live_fix_vtt_by_audio",
            "live_record_limit",
            "live_wait_time",
            "live_take_count",
            "allow_hls_multi_ext_map",
            "url_processor_args",
            "no_date_info",
            "use_ffmpeg_concat_demuxer",
        ];

        if !allowed_fields.contains(&field) {
            return Err(format!("Invalid field: {}", field));
        }

        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let now = chrono::Utc::now().to_rfc3339();
        let sql = format!(
            "UPDATE m3u8dl_settings SET {} = ?1, updated_at = ?2 WHERE id = 1",
            field
        );

        conn.execute(&sql, params![value, now])
            .map_err(|e| format!("Failed to update {}: {}", field, e))?;

        Ok(())
    }

    // ========================================
    // FFmpeg 配置
    // ========================================

    /// 获取 FFmpeg 配置
    pub fn get_ffmpeg_settings(&self) -> Result<FFmpegSettings, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        conn.query_row(
            "SELECT ffmpeg_path, ffprobe_path, retry_count, timeout, max_speed,
                    connection_timeout, reconnect_attempts, reconnect_delay,
                    overwrite_existing, preserve_timestamps, user_agent, referer
             FROM ffmpeg_settings WHERE id = 1",
            [],
            |row| {
                Ok(FFmpegSettings {
                    ffmpeg_path: row.get(0)?,
                    ffprobe_path: row.get(1)?,
                    retry_count: row.get(2)?,
                    timeout: row.get(3)?,
                    max_speed: row.get(4)?,
                    connection_timeout: row.get(5)?,
                    reconnect_attempts: row.get(6)?,
                    reconnect_delay: row.get(7)?,
                    overwrite_existing: row.get::<_, i32>(8)? != 0,
                    preserve_timestamps: row.get::<_, i32>(9)? != 0,
                    user_agent: row.get(10)?,
                    referer: row.get(11)?,
                })
            },
        )
        .map_err(|e| format!("Failed to get ffmpeg settings: {}", e))
    }

    /// 更新 FFmpeg 配置字段
    pub fn update_ffmpeg_setting_field(&self, field: &str, value: &str) -> Result<(), String> {
        let allowed_fields = [
            "ffmpeg_path",
            "ffprobe_path",
            "retry_count",
            "timeout",
            "max_speed",
            "connection_timeout",
            "reconnect_attempts",
            "reconnect_delay",
            "overwrite_existing",
            "preserve_timestamps",
            "user_agent",
            "referer",
        ];

        if !allowed_fields.contains(&field) {
            return Err(format!("Invalid field: {}", field));
        }

        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let now = chrono::Utc::now().to_rfc3339();
        let sql = format!(
            "UPDATE ffmpeg_settings SET {} = ?1, updated_at = ?2 WHERE id = 1",
            field
        );

        conn.execute(&sql, params![value, now])
            .map_err(|e| format!("Failed to update {}: {}", field, e))?;

        Ok(())
    }

    // ========================================
    // 网络配置
    // ========================================

    /// 获取网络配置
    pub fn get_network_settings(&self) -> Result<NetworkSettings, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        conn.query_row(
            "SELECT use_system_proxy, custom_proxy, base_url, append_url_params
             FROM network_settings WHERE id = 1",
            [],
            |row| {
                Ok(NetworkSettings {
                    use_system_proxy: row.get::<_, i32>(0)? != 0,
                    custom_proxy: row.get(1)?,
                    base_url: row.get(2)?,
                    append_url_params: row.get::<_, i32>(3)? != 0,
                })
            },
        )
        .map_err(|e| format!("Failed to get network settings: {}", e))
    }

    /// 更新网络配置字段
    pub fn update_network_setting_field(&self, field: &str, value: &str) -> Result<(), String> {
        let allowed_fields = [
            "use_system_proxy",
            "custom_proxy",
            "base_url",
            "append_url_params",
        ];

        if !allowed_fields.contains(&field) {
            return Err(format!("Invalid field: {}", field));
        }

        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let now = chrono::Utc::now().to_rfc3339();
        let sql = format!(
            "UPDATE network_settings SET {} = ?1, updated_at = ?2 WHERE id = 1",
            field
        );

        conn.execute(&sql, params![value, now])
            .map_err(|e| format!("Failed to update {}: {}", field, e))?;

        Ok(())
    }

    /// 获取所有网络请求头
    pub fn get_network_headers(&self) -> Result<Vec<NetworkHeader>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare(
                "SELECT id, name, value, enabled, sort_order
                 FROM network_headers
                 ORDER BY sort_order",
            )
            .map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(NetworkHeader {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    value: row.get(2)?,
                    enabled: row.get::<_, i32>(3)? != 0,
                    sort_order: row.get(4)?,
                })
            })
            .map_err(|e| format!("Failed to query headers: {}", e))?;

        let mut headers = Vec::new();
        for row in rows {
            headers.push(row.map_err(|e| format!("Failed to read row: {}", e))?);
        }

        Ok(headers)
    }

    /// 添加网络请求头
    pub fn add_network_header(&self, name: &str, value: &str) -> Result<i64, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        // 获取最大排序号
        let max_order: i32 = conn
            .query_row(
                "SELECT COALESCE(MAX(sort_order), -1) FROM network_headers",
                [],
                |row| row.get(0),
            )
            .unwrap_or(-1);

        conn.execute(
            "INSERT INTO network_headers (name, value, enabled, sort_order) VALUES (?1, ?2, 1, ?3)",
            params![name, value, max_order + 1],
        )
        .map_err(|e| format!("Failed to add header: {}", e))?;

        Ok(conn.last_insert_rowid())
    }

    /// 更新网络请求头
    pub fn update_network_header(
        &self,
        id: i64,
        name: &str,
        value: &str,
        enabled: bool,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        conn.execute(
            "UPDATE network_headers SET name = ?1, value = ?2, enabled = ?3 WHERE id = ?4",
            params![name, value, enabled as i32, id],
        )
        .map_err(|e| format!("Failed to update header: {}", e))?;

        Ok(())
    }

    /// 删除网络请求头
    pub fn delete_network_header(&self, id: i64) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        conn.execute("DELETE FROM network_headers WHERE id = ?1", params![id])
            .map_err(|e| format!("Failed to delete header: {}", e))?;

        Ok(())
    }

    // ========================================
    // 解密配置
    // ========================================

    /// 获取解密配置
    pub fn get_decryption_settings(&self) -> Result<DecryptionSettings, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        conn.query_row(
            "SELECT key_text_file, decryption_engine, decryption_bin_path,
                    real_time_decryption, custom_hls_enabled, custom_hls_method,
                    custom_hls_key_type, custom_hls_key_value, custom_hls_iv_type,
                    custom_hls_iv_value
             FROM decryption_settings WHERE id = 1",
            [],
            |row| {
                Ok(DecryptionSettings {
                    key_text_file: row.get(0)?,
                    decryption_engine: row.get(1)?,
                    decryption_bin_path: row.get(2)?,
                    real_time_decryption: row.get::<_, i32>(3)? != 0,
                    custom_hls_enabled: row.get::<_, i32>(4)? != 0,
                    custom_hls_method: row.get(5)?,
                    custom_hls_key_type: row.get(6)?,
                    custom_hls_key_value: row.get(7)?,
                    custom_hls_iv_type: row.get(8)?,
                    custom_hls_iv_value: row.get(9)?,
                })
            },
        )
        .map_err(|e| format!("Failed to get decryption settings: {}", e))
    }

    /// 更新解密配置字段
    pub fn update_decryption_setting_field(&self, field: &str, value: &str) -> Result<(), String> {
        let allowed_fields = [
            "key_text_file",
            "decryption_engine",
            "decryption_bin_path",
            "real_time_decryption",
            "custom_hls_enabled",
            "custom_hls_method",
            "custom_hls_key_type",
            "custom_hls_key_value",
            "custom_hls_iv_type",
            "custom_hls_iv_value",
        ];

        if !allowed_fields.contains(&field) {
            return Err(format!("Invalid field: {}", field));
        }

        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let now = chrono::Utc::now().to_rfc3339();
        let sql = format!(
            "UPDATE decryption_settings SET {} = ?1, updated_at = ?2 WHERE id = 1",
            field
        );

        conn.execute(&sql, params![value, now])
            .map_err(|e| format!("Failed to update {}: {}", field, e))?;

        Ok(())
    }

    /// 获取所有解密密钥
    pub fn get_decryption_keys(&self) -> Result<Vec<DecryptionKey>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare(
                "SELECT id, kid, key, sort_order
                 FROM decryption_keys
                 ORDER BY sort_order",
            )
            .map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(DecryptionKey {
                    id: row.get(0)?,
                    kid: row.get(1)?,
                    key: row.get(2)?,
                    sort_order: row.get(3)?,
                })
            })
            .map_err(|e| format!("Failed to query keys: {}", e))?;

        let mut keys = Vec::new();
        for row in rows {
            keys.push(row.map_err(|e| format!("Failed to read row: {}", e))?);
        }

        Ok(keys)
    }

    /// 添加解密密钥
    pub fn add_decryption_key(&self, kid: Option<&str>, key: &str) -> Result<i64, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        // 获取最大排序号
        let max_order: i32 = conn
            .query_row(
                "SELECT COALESCE(MAX(sort_order), -1) FROM decryption_keys",
                [],
                |row| row.get(0),
            )
            .unwrap_or(-1);

        conn.execute(
            "INSERT INTO decryption_keys (kid, key, sort_order) VALUES (?1, ?2, ?3)",
            params![kid, key, max_order + 1],
        )
        .map_err(|e| format!("Failed to add key: {}", e))?;

        Ok(conn.last_insert_rowid())
    }

    /// 删除解密密钥
    pub fn delete_decryption_key(&self, id: i64) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        conn.execute("DELETE FROM decryption_keys WHERE id = ?1", params![id])
            .map_err(|e| format!("Failed to delete key: {}", e))?;

        Ok(())
    }
}
