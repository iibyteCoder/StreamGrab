//! 配置模板仓库
//!
//! 管理配置模板的 CRUD 操作

use rusqlite::{params, Connection};
use std::sync::Mutex;

/// 下载器类型
#[derive(Debug, Clone, PartialEq, Default)]
pub enum DownloaderType {
    #[default]
    M3U8DL,
    FFmpeg,
}

impl std::fmt::Display for DownloaderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::M3U8DL => write!(f, "m3u8dl"),
            Self::FFmpeg => write!(f, "ffmpeg"),
        }
    }
}

impl std::str::FromStr for DownloaderType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "m3u8dl" | "m3u8" => Ok(Self::M3U8DL),
            "ffmpeg" => Ok(Self::FFmpeg),
            _ => Err(format!("Unknown downloader type: {}", s)),
        }
    }
}

/// 配置模板
#[derive(Debug, Clone)]
pub struct ConfigTemplate {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub is_preset: bool,
    pub downloader_type: DownloaderType,
    pub created_at: String,
    pub updated_at: String,
}

/// 模板 M3U8DL 覆盖配置
#[derive(Debug, Clone, Default)]
pub struct TemplateM3U8DL {
    pub thread_count: Option<i32>,
    pub retry_count: Option<i32>,
    pub timeout: Option<i32>,
    pub max_speed: Option<String>,
    pub auto_select: Option<bool>,
    pub select_video: Option<String>,
    pub select_audio: Option<String>,
    pub select_subtitle: Option<String>,
    pub drop_video: Option<String>,
    pub drop_audio: Option<String>,
    pub drop_subtitle: Option<String>,
    pub check_segments_count: Option<bool>,
    pub del_after_done: Option<bool>,
    pub skip_merge: Option<bool>,
    pub write_meta_json: Option<bool>,
    pub binary_merge: Option<bool>,
    pub concurrent_download: Option<bool>,
    pub mux_format: Option<String>,
    pub muxer: Option<String>,
    pub mux_bin_path: Option<String>,
    pub mux_skip_subtitles: Option<bool>,
    pub mux_keep_original: Option<bool>,
    pub sub_only: Option<bool>,
    pub sub_format: Option<String>,
    pub auto_subtitle_fix: Option<bool>,
    pub live_perform_as_vod: Option<bool>,
    pub live_real_time_merge: Option<bool>,
    pub live_keep_segments: Option<bool>,
    pub live_pipe_mux: Option<bool>,
    pub live_fix_vtt_by_audio: Option<bool>,
    pub live_record_limit: Option<String>,
    pub live_wait_time: Option<i32>,
    pub live_take_count: Option<i32>,
    pub allow_hls_multi_ext_map: Option<bool>,
    pub url_processor_args: Option<String>,
    pub no_date_info: Option<bool>,
    pub use_ffmpeg_concat_demuxer: Option<bool>,
}

/// 模板 FFmpeg 覆盖配置
#[derive(Debug, Clone, Default)]
pub struct TemplateFFmpeg {
    pub ffmpeg_path: Option<String>,
    pub ffprobe_path: Option<String>,
    pub retry_count: Option<i32>,
    pub timeout: Option<i32>,
    pub max_speed: Option<String>,
    pub connection_timeout: Option<i32>,
    pub reconnect_attempts: Option<i32>,
    pub reconnect_delay: Option<i32>,
    pub overwrite_existing: Option<bool>,
    pub preserve_timestamps: Option<bool>,
    pub user_agent: Option<String>,
    pub referer: Option<String>,
}

/// 模板网络覆盖配置
#[derive(Debug, Clone, Default)]
pub struct TemplateNetwork {
    pub use_system_proxy: Option<bool>,
    pub custom_proxy: Option<String>,
    pub base_url: Option<String>,
    pub append_url_params: Option<bool>,
}

/// 模板解密覆盖配置
#[derive(Debug, Clone, Default)]
pub struct TemplateDecryption {
    pub key_text_file: Option<String>,
    pub decryption_engine: Option<String>,
    pub decryption_bin_path: Option<String>,
    pub real_time_decryption: Option<bool>,
    pub custom_hls_enabled: Option<bool>,
    pub custom_hls_method: Option<String>,
    pub custom_hls_key_type: Option<String>,
    pub custom_hls_key_value: Option<String>,
    pub custom_hls_iv_type: Option<String>,
    pub custom_hls_iv_value: Option<String>,
}

/// 完整的模板配置（包含所有覆盖）
#[derive(Debug, Clone)]
pub struct FullTemplateConfig {
    pub template: ConfigTemplate,
    pub m3u8dl: Option<TemplateM3U8DL>,
    pub ffmpeg: Option<TemplateFFmpeg>,
    pub network: Option<TemplateNetwork>,
    pub decryption: Option<TemplateDecryption>,
    pub network_headers: Vec<TemplateHeader>,
    pub decryption_keys: Vec<TemplateKey>,
    pub ad_filter_keywords: Vec<TemplateKeyword>,
    pub mux_imports: Vec<TemplateMuxImport>,
}

/// 模板请求头
#[derive(Debug, Clone)]
pub struct TemplateHeader {
    pub id: i64,
    pub template_id: String,
    pub name: String,
    pub value: String,
    pub enabled: bool,
    pub sort_order: i32,
}

/// 模板解密密钥
#[derive(Debug, Clone)]
pub struct TemplateKey {
    pub id: i64,
    pub template_id: String,
    pub kid: Option<String>,
    pub key: String,
    pub sort_order: i32,
}

/// 模板广告过滤关键字
#[derive(Debug, Clone)]
pub struct TemplateKeyword {
    pub id: i64,
    pub template_id: String,
    pub keyword: String,
    pub enabled: bool,
    pub sort_order: i32,
}

/// 模板混流导入
#[derive(Debug, Clone)]
pub struct TemplateMuxImport {
    pub id: i64,
    pub template_id: String,
    pub path: String,
    pub lang: Option<String>,
    pub name: Option<String>,
    pub sort_order: i32,
}

/// 配置模板仓库
pub struct TemplateRepository {
    conn: Mutex<Connection>,
}

impl TemplateRepository {
    /// 创建模板仓库
    pub fn new(conn: Connection) -> Result<Self, String> {
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    // ========================================
    // 模板基本操作
    // ========================================

    /// 获取所有模板
    pub fn get_all_templates(&self) -> Result<Vec<ConfigTemplate>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare(
                "SELECT id, name, description, is_preset, downloader_type, created_at, updated_at
                 FROM config_templates
                 ORDER BY is_preset DESC, name ASC",
            )
            .map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(ConfigTemplate {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    is_preset: row.get::<_, i32>(3)? != 0,
                    downloader_type: row.get::<_, String>(4)?.parse().unwrap_or_default(),
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })
            .map_err(|e| format!("Failed to query templates: {}", e))?;

        let mut templates = Vec::new();
        for row in rows {
            templates.push(row.map_err(|e| format!("Failed to read row: {}", e))?);
        }

        Ok(templates)
    }

    /// 获取单个模板
    pub fn get_template(&self, id: &str) -> Result<Option<ConfigTemplate>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let result = conn.query_row(
            "SELECT id, name, description, is_preset, downloader_type, created_at, updated_at
             FROM config_templates WHERE id = ?1",
            params![id],
            |row| {
                Ok(ConfigTemplate {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    is_preset: row.get::<_, i32>(3)? != 0,
                    downloader_type: row.get::<_, String>(4)?.parse().unwrap_or_default(),
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            },
        );

        match result {
            Ok(template) => Ok(Some(template)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(format!("Failed to get template: {}", e)),
        }
    }

    /// 创建模板
    pub fn create_template(&self, template: &ConfigTemplate) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT INTO config_templates (id, name, description, is_preset, downloader_type, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                template.id,
                template.name,
                template.description,
                template.is_preset as i32,
                template.downloader_type.to_string(),
                template.created_at,
                template.updated_at,
            ],
        )
        .map_err(|e| format!("Failed to create template: {}", e))?;

        Ok(())
    }

    /// 更新模板基本信息
    pub fn update_template(
        &self,
        id: &str,
        name: &str,
        description: Option<&str>,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "UPDATE config_templates SET name = ?1, description = ?2, updated_at = ?3 WHERE id = ?4",
            params![name, description, now, id],
        )
        .map_err(|e| format!("Failed to update template: {}", e))?;

        Ok(())
    }

    /// 删除模板（级联删除所有关联数据）
    pub fn delete_template(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        conn.execute("DELETE FROM config_templates WHERE id = ?1", params![id])
            .map_err(|e| format!("Failed to delete template: {}", e))?;

        Ok(())
    }

    // ========================================
    // 模板 M3U8DL 覆盖配置
    // ========================================

    /// 获取模板 M3U8DL 覆盖配置
    pub fn get_template_m3u8dl(&self, template_id: &str) -> Result<Option<TemplateM3U8DL>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let result = conn.query_row(
            "SELECT thread_count, retry_count, timeout, max_speed, auto_select,
                    select_video, select_audio, select_subtitle, drop_video, drop_audio, drop_subtitle,
                    check_segments_count, del_after_done, skip_merge, write_meta_json, binary_merge,
                    concurrent_download, mux_format, muxer, mux_bin_path, mux_skip_subtitles, mux_keep_original,
                    sub_only, sub_format, auto_subtitle_fix, live_perform_as_vod, live_real_time_merge,
                    live_keep_segments, live_pipe_mux, live_fix_vtt_by_audio, live_record_limit,
                    live_wait_time, live_take_count, allow_hls_multi_ext_map, url_processor_args,
                    no_date_info, use_ffmpeg_concat_demuxer
             FROM template_m3u8dl_overrides WHERE template_id = ?1",
            params![template_id],
            |row| {
                Ok(TemplateM3U8DL {
                    thread_count: row.get(0)?,
                    retry_count: row.get(1)?,
                    timeout: row.get(2)?,
                    max_speed: row.get(3)?,
                    auto_select: row.get::<_, Option<i32>>(4)?.map(|v| v != 0),
                    select_video: row.get(5)?,
                    select_audio: row.get(6)?,
                    select_subtitle: row.get(7)?,
                    drop_video: row.get(8)?,
                    drop_audio: row.get(9)?,
                    drop_subtitle: row.get(10)?,
                    check_segments_count: row.get::<_, Option<i32>>(11)?.map(|v| v != 0),
                    del_after_done: row.get::<_, Option<i32>>(12)?.map(|v| v != 0),
                    skip_merge: row.get::<_, Option<i32>>(13)?.map(|v| v != 0),
                    write_meta_json: row.get::<_, Option<i32>>(14)?.map(|v| v != 0),
                    binary_merge: row.get::<_, Option<i32>>(15)?.map(|v| v != 0),
                    concurrent_download: row.get::<_, Option<i32>>(16)?.map(|v| v != 0),
                    mux_format: row.get(17)?,
                    muxer: row.get(18)?,
                    mux_bin_path: row.get(19)?,
                    mux_skip_subtitles: row.get::<_, Option<i32>>(20)?.map(|v| v != 0),
                    mux_keep_original: row.get::<_, Option<i32>>(21)?.map(|v| v != 0),
                    sub_only: row.get::<_, Option<i32>>(22)?.map(|v| v != 0),
                    sub_format: row.get(23)?,
                    auto_subtitle_fix: row.get::<_, Option<i32>>(24)?.map(|v| v != 0),
                    live_perform_as_vod: row.get::<_, Option<i32>>(25)?.map(|v| v != 0),
                    live_real_time_merge: row.get::<_, Option<i32>>(26)?.map(|v| v != 0),
                    live_keep_segments: row.get::<_, Option<i32>>(27)?.map(|v| v != 0),
                    live_pipe_mux: row.get::<_, Option<i32>>(28)?.map(|v| v != 0),
                    live_fix_vtt_by_audio: row.get::<_, Option<i32>>(29)?.map(|v| v != 0),
                    live_record_limit: row.get(30)?,
                    live_wait_time: row.get(31)?,
                    live_take_count: row.get(32)?,
                    allow_hls_multi_ext_map: row.get::<_, Option<i32>>(33)?.map(|v| v != 0),
                    url_processor_args: row.get(34)?,
                    no_date_info: row.get::<_, Option<i32>>(35)?.map(|v| v != 0),
                    use_ffmpeg_concat_demuxer: row.get::<_, Option<i32>>(36)?.map(|v| v != 0),
                })
            },
        );

        match result {
            Ok(config) => Ok(Some(config)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(format!("Failed to get template m3u8dl config: {}", e)),
        }
    }

    /// 保存模板 M3U8DL 覆盖配置
    pub fn save_template_m3u8dl(
        &self,
        template_id: &str,
        config: &TemplateM3U8DL,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT OR REPLACE INTO template_m3u8dl_overrides (
                template_id, thread_count, retry_count, timeout, max_speed, auto_select,
                select_video, select_audio, select_subtitle, drop_video, drop_audio, drop_subtitle,
                check_segments_count, del_after_done, skip_merge, write_meta_json, binary_merge,
                concurrent_download, mux_format, muxer, mux_bin_path, mux_skip_subtitles, mux_keep_original,
                sub_only, sub_format, auto_subtitle_fix, live_perform_as_vod, live_real_time_merge,
                live_keep_segments, live_pipe_mux, live_fix_vtt_by_audio, live_record_limit,
                live_wait_time, live_take_count, allow_hls_multi_ext_map, url_processor_args,
                no_date_info, use_ffmpeg_concat_demuxer
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29, ?30, ?31, ?32, ?33, ?34, ?35, ?36, ?37, ?38)",
            params![
                template_id,
                config.thread_count,
                config.retry_count,
                config.timeout,
                config.max_speed,
                config.auto_select.map(|v| v as i32),
                &config.select_video,
                &config.select_audio,
                &config.select_subtitle,
                &config.drop_video,
                &config.drop_audio,
                &config.drop_subtitle,
                config.check_segments_count.map(|v| v as i32),
                config.del_after_done.map(|v| v as i32),
                config.skip_merge.map(|v| v as i32),
                config.write_meta_json.map(|v| v as i32),
                config.binary_merge.map(|v| v as i32),
                config.concurrent_download.map(|v| v as i32),
                &config.mux_format,
                &config.muxer,
                &config.mux_bin_path,
                config.mux_skip_subtitles.map(|v| v as i32),
                config.mux_keep_original.map(|v| v as i32),
                config.sub_only.map(|v| v as i32),
                &config.sub_format,
                config.auto_subtitle_fix.map(|v| v as i32),
                config.live_perform_as_vod.map(|v| v as i32),
                config.live_real_time_merge.map(|v| v as i32),
                config.live_keep_segments.map(|v| v as i32),
                config.live_pipe_mux.map(|v| v as i32),
                config.live_fix_vtt_by_audio.map(|v| v as i32),
                &config.live_record_limit,
                config.live_wait_time,
                config.live_take_count,
                config.allow_hls_multi_ext_map.map(|v| v as i32),
                &config.url_processor_args,
                config.no_date_info.map(|v| v as i32),
                config.use_ffmpeg_concat_demuxer.map(|v| v as i32),
            ],
        )
        .map_err(|e| format!("Failed to save template m3u8dl config: {}", e))?;

        Ok(())
    }

    // ========================================
    // 模板网络请求头
    // ========================================

    /// 获取模板网络请求头
    pub fn get_template_headers(&self, template_id: &str) -> Result<Vec<TemplateHeader>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare(
                "SELECT id, template_id, name, value, enabled, sort_order
                 FROM template_network_headers
                 WHERE template_id = ?1
                 ORDER BY sort_order",
            )
            .map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let rows = stmt
            .query_map(params![template_id], |row| {
                Ok(TemplateHeader {
                    id: row.get(0)?,
                    template_id: row.get(1)?,
                    name: row.get(2)?,
                    value: row.get(3)?,
                    enabled: row.get::<_, i32>(4)? != 0,
                    sort_order: row.get(5)?,
                })
            })
            .map_err(|e| format!("Failed to query headers: {}", e))?;

        let mut headers = Vec::new();
        for row in rows {
            headers.push(row.map_err(|e| format!("Failed to read row: {}", e))?);
        }

        Ok(headers)
    }

    /// 添加模板请求头
    pub fn add_template_header(
        &self,
        template_id: &str,
        name: &str,
        value: &str,
    ) -> Result<i64, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        // 获取最大排序号
        let max_order: i32 = conn
            .query_row(
                "SELECT COALESCE(MAX(sort_order), -1) FROM template_network_headers WHERE template_id = ?1",
                params![template_id],
                |row| row.get(0),
            )
            .unwrap_or(-1);

        conn.execute(
            "INSERT INTO template_network_headers (template_id, name, value, enabled, sort_order)
             VALUES (?1, ?2, ?3, 1, ?4)",
            params![template_id, name, value, max_order + 1],
        )
        .map_err(|e| format!("Failed to add header: {}", e))?;

        Ok(conn.last_insert_rowid())
    }

    /// 删除模板请求头
    pub fn delete_template_header(&self, id: i64) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        conn.execute(
            "DELETE FROM template_network_headers WHERE id = ?1",
            params![id],
        )
        .map_err(|e| format!("Failed to delete header: {}", e))?;

        Ok(())
    }
}
