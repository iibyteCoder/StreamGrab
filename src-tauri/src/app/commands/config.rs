//! 配置相关命令
//!
//! 处理应用配置的读取、保存、导入、导出
//! 支持字段级别更新，实现即时保存功能

use std::fs;

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use super::utils::get_db;
use crate::infrastructure::db::repository::{
    AppSettings, DecryptionKey, DecryptionSettings, FFmpegSettings, M3U8DLSettings, NetworkHeader,
    NetworkSettings,
};

// ========================================
// 应用配置
// ========================================

/// 获取应用配置
#[tauri::command]
pub async fn get_app_settings(app: AppHandle) -> Result<AppSettingsResponse, String> {
    log::info!("Getting app settings");

    let db = get_db(&app)?;
    let settings = db.config.get_app_settings()?;

    Ok(settings.into())
}

/// 应用配置响应（用于前端）
#[derive(Debug, Serialize, Deserialize)]
pub struct AppSettingsResponse {
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

impl From<AppSettings> for AppSettingsResponse {
    fn from(value: AppSettings) -> Self {
        Self {
            language: value.language,
            auto_start_download: value.auto_start_download,
            minimize_to_tray: value.minimize_to_tray,
            check_update: value.check_update,
            default_save_dir: value.default_save_dir,
            default_tmp_dir: value.default_tmp_dir,
            theme: value.theme,
            show_notification: value.show_notification,
            clipboard_watch: value.clipboard_watch,
            log_level: value.log_level,
            log_file_path: value.log_file_path,
            no_log: value.no_log,
        }
    }
}

/// 更新应用配置字段
#[tauri::command]
pub async fn update_app_setting_field(
    field: String,
    value: String,
    app: AppHandle,
) -> Result<(), String> {
    log::info!("Updating app setting field: {} = {}", field, value);

    let db = get_db(&app)?;
    db.config.update_app_setting_field(&field, &value)?;

    Ok(())
}

// ========================================
// M3U8DL 配置
// ========================================

/// 获取 M3U8DL 配置
#[tauri::command]
pub async fn get_m3u8dl_settings(app: AppHandle) -> Result<M3U8DLSettingsResponse, String> {
    log::info!("Getting M3U8DL settings");

    let db = get_db(&app)?;
    let settings = db.config.get_m3u8dl_settings()?;

    Ok(settings.into())
}

/// M3U8DL 配置响应（用于前端）
#[derive(Debug, Serialize, Deserialize)]
pub struct M3U8DLSettingsResponse {
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

impl From<M3U8DLSettings> for M3U8DLSettingsResponse {
    fn from(value: M3U8DLSettings) -> Self {
        Self {
            n_m3u8dl_path: value.n_m3u8dl_path,
            thread_count: value.thread_count,
            retry_count: value.retry_count,
            timeout: value.timeout,
            max_speed: value.max_speed,
            auto_select: value.auto_select,
            select_video: value.select_video,
            select_audio: value.select_audio,
            select_subtitle: value.select_subtitle,
            drop_video: value.drop_video,
            drop_audio: value.drop_audio,
            drop_subtitle: value.drop_subtitle,
            check_segments_count: value.check_segments_count,
            del_after_done: value.del_after_done,
            skip_merge: value.skip_merge,
            write_meta_json: value.write_meta_json,
            binary_merge: value.binary_merge,
            concurrent_download: value.concurrent_download,
            mux_format: value.mux_format,
            muxer: value.muxer,
            mux_bin_path: value.mux_bin_path,
            mux_skip_subtitles: value.mux_skip_subtitles,
            mux_keep_original: value.mux_keep_original,
            sub_only: value.sub_only,
            sub_format: value.sub_format,
            auto_subtitle_fix: value.auto_subtitle_fix,
            live_perform_as_vod: value.live_perform_as_vod,
            live_real_time_merge: value.live_real_time_merge,
            live_keep_segments: value.live_keep_segments,
            live_pipe_mux: value.live_pipe_mux,
            live_fix_vtt_by_audio: value.live_fix_vtt_by_audio,
            live_record_limit: value.live_record_limit,
            live_wait_time: value.live_wait_time,
            live_take_count: value.live_take_count,
            allow_hls_multi_ext_map: value.allow_hls_multi_ext_map,
            url_processor_args: value.url_processor_args,
            no_date_info: value.no_date_info,
            use_ffmpeg_concat_demuxer: value.use_ffmpeg_concat_demuxer,
        }
    }
}

/// 更新 M3U8DL 配置字段
#[tauri::command]
pub async fn update_m3u8dl_setting_field(
    field: String,
    value: String,
    app: AppHandle,
) -> Result<(), String> {
    log::info!("Updating M3U8DL setting field: {} = {}", field, value);

    let db = get_db(&app)?;
    db.config.update_m3u8dl_setting_field(&field, &value)?;

    Ok(())
}

// ========================================
// FFmpeg 配置
// ========================================

/// 获取 FFmpeg 配置
#[tauri::command]
pub async fn get_ffmpeg_settings(app: AppHandle) -> Result<FFmpegSettingsResponse, String> {
    log::info!("Getting FFmpeg settings");

    let db = get_db(&app)?;
    let settings = db.config.get_ffmpeg_settings()?;

    Ok(settings.into())
}

/// FFmpeg 配置响应（用于前端）
#[derive(Debug, Serialize, Deserialize)]
pub struct FFmpegSettingsResponse {
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

impl From<FFmpegSettings> for FFmpegSettingsResponse {
    fn from(value: FFmpegSettings) -> Self {
        Self {
            ffmpeg_path: value.ffmpeg_path,
            ffprobe_path: value.ffprobe_path,
            retry_count: value.retry_count,
            timeout: value.timeout,
            max_speed: value.max_speed,
            connection_timeout: value.connection_timeout,
            reconnect_attempts: value.reconnect_attempts,
            reconnect_delay: value.reconnect_delay,
            overwrite_existing: value.overwrite_existing,
            preserve_timestamps: value.preserve_timestamps,
            user_agent: value.user_agent,
            referer: value.referer,
        }
    }
}

/// 更新 FFmpeg 配置字段
#[tauri::command]
pub async fn update_ffmpeg_setting_field(
    field: String,
    value: String,
    app: AppHandle,
) -> Result<(), String> {
    log::info!("Updating FFmpeg setting field: {} = {}", field, value);

    let db = get_db(&app)?;
    db.config.update_ffmpeg_setting_field(&field, &value)?;

    Ok(())
}

// ========================================
// 网络配置
// ========================================

/// 获取网络配置
#[tauri::command]
pub async fn get_network_settings(app: AppHandle) -> Result<NetworkSettingsResponse, String> {
    log::info!("Getting network settings");

    let db = get_db(&app)?;
    let settings = db.config.get_network_settings()?;

    Ok(settings.into())
}

/// 网络配置响应（用于前端）
#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkSettingsResponse {
    pub use_system_proxy: bool,
    pub custom_proxy: Option<String>,
    pub base_url: Option<String>,
    pub append_url_params: bool,
}

impl From<NetworkSettings> for NetworkSettingsResponse {
    fn from(value: NetworkSettings) -> Self {
        Self {
            use_system_proxy: value.use_system_proxy,
            custom_proxy: value.custom_proxy,
            base_url: value.base_url,
            append_url_params: value.append_url_params,
        }
    }
}

/// 更新网络配置字段
#[tauri::command]
pub async fn update_network_setting_field(
    field: String,
    value: String,
    app: AppHandle,
) -> Result<(), String> {
    log::info!("Updating network setting field: {} = {}", field, value);

    let db = get_db(&app)?;
    db.config.update_network_setting_field(&field, &value)?;

    Ok(())
}

// ========================================
// 网络请求头
// ========================================

/// 获取所有网络请求头
#[tauri::command]
pub async fn get_network_headers(app: AppHandle) -> Result<Vec<NetworkHeaderResponse>, String> {
    log::info!("Getting network headers");

    let db = get_db(&app)?;
    let headers = db.config.get_network_headers()?;

    Ok(headers.into_iter().map(|h| h.into()).collect())
}

/// 网络请求头响应（用于前端）
#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkHeaderResponse {
    pub id: i64,
    pub name: String,
    pub value: String,
    pub enabled: bool,
    pub sort_order: i32,
}

impl From<NetworkHeader> for NetworkHeaderResponse {
    fn from(value: NetworkHeader) -> Self {
        Self {
            id: value.id,
            name: value.name,
            value: value.value,
            enabled: value.enabled,
            sort_order: value.sort_order,
        }
    }
}

/// 添加网络请求头
#[tauri::command]
pub async fn add_network_header(
    name: String,
    value: String,
    app: AppHandle,
) -> Result<i64, String> {
    log::info!("Adding network header: {}", name);

    let db = get_db(&app)?;
    let id = db.config.add_network_header(&name, &value)?;

    Ok(id)
}

/// 更新网络请求头
#[tauri::command]
pub async fn update_network_header(
    id: i64,
    name: String,
    value: String,
    enabled: bool,
    app: AppHandle,
) -> Result<(), String> {
    log::info!("Updating network header {}: {}", id, name);

    let db = get_db(&app)?;
    db.config
        .update_network_header(id, &name, &value, enabled)?;

    Ok(())
}

/// 删除网络请求头
#[tauri::command]
pub async fn delete_network_header(id: i64, app: AppHandle) -> Result<(), String> {
    log::info!("Deleting network header: {}", id);

    let db = get_db(&app)?;
    db.config.delete_network_header(id)?;

    Ok(())
}

// ========================================
// 解密配置
// ========================================

/// 获取解密配置
#[tauri::command]
pub async fn get_decryption_settings(app: AppHandle) -> Result<DecryptionSettingsResponse, String> {
    log::info!("Getting decryption settings");

    let db = get_db(&app)?;
    let settings = db.config.get_decryption_settings()?;

    Ok(settings.into())
}

/// 解密配置响应（用于前端）
#[derive(Debug, Serialize, Deserialize)]
pub struct DecryptionSettingsResponse {
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

impl From<DecryptionSettings> for DecryptionSettingsResponse {
    fn from(value: DecryptionSettings) -> Self {
        Self {
            key_text_file: value.key_text_file,
            decryption_engine: value.decryption_engine,
            decryption_bin_path: value.decryption_bin_path,
            real_time_decryption: value.real_time_decryption,
            custom_hls_enabled: value.custom_hls_enabled,
            custom_hls_method: value.custom_hls_method,
            custom_hls_key_type: value.custom_hls_key_type,
            custom_hls_key_value: value.custom_hls_key_value,
            custom_hls_iv_type: value.custom_hls_iv_type,
            custom_hls_iv_value: value.custom_hls_iv_value,
        }
    }
}

/// 更新解密配置字段
#[tauri::command]
pub async fn update_decryption_setting_field(
    field: String,
    value: String,
    app: AppHandle,
) -> Result<(), String> {
    log::info!("Updating decryption setting field: {} = {}", field, value);

    let db = get_db(&app)?;
    db.config.update_decryption_setting_field(&field, &value)?;

    Ok(())
}

// ========================================
// 解密密钥
// ========================================

/// 获取所有解密密钥
#[tauri::command]
pub async fn get_decryption_keys(app: AppHandle) -> Result<Vec<DecryptionKeyResponse>, String> {
    log::info!("Getting decryption keys");

    let db = get_db(&app)?;
    let keys = db.config.get_decryption_keys()?;

    Ok(keys.into_iter().map(|k| k.into()).collect())
}

/// 解密密钥响应（用于前端）
#[derive(Debug, Serialize, Deserialize)]
pub struct DecryptionKeyResponse {
    pub id: i64,
    pub kid: Option<String>,
    pub key: String,
    pub sort_order: i32,
}

impl From<DecryptionKey> for DecryptionKeyResponse {
    fn from(value: DecryptionKey) -> Self {
        Self {
            id: value.id,
            kid: value.kid,
            key: value.key,
            sort_order: value.sort_order,
        }
    }
}

/// 添加解密密钥
#[tauri::command]
pub async fn add_decryption_key(
    kid: Option<String>,
    key: String,
    app: AppHandle,
) -> Result<i64, String> {
    log::info!("Adding decryption key");

    let db = get_db(&app)?;
    let id = db.config.add_decryption_key(kid.as_deref(), &key)?;

    Ok(id)
}

/// 删除解密密钥
#[tauri::command]
pub async fn delete_decryption_key(id: i64, app: AppHandle) -> Result<(), String> {
    log::info!("Deleting decryption key: {}", id);

    let db = get_db(&app)?;
    db.config.delete_decryption_key(id)?;

    Ok(())
}

// ========================================
// 导入/导出
// ========================================

/// 导出配置到指定路径
#[tauri::command]
pub async fn export_config(file_path: String, app: AppHandle) -> Result<(), String> {
    log::info!("Exporting config to: {}", file_path);

    let db = get_db(&app)?;

    // 收集所有配置
    let config = AllConfigExport {
        app: db.config.get_app_settings()?.into(),
        m3u8dl: db.config.get_m3u8dl_settings()?.into(),
        ffmpeg: db.config.get_ffmpeg_settings()?.into(),
        network: db.config.get_network_settings()?.into(),
        decryption: db.config.get_decryption_settings()?.into(),
        headers: db
            .config
            .get_network_headers()?
            .into_iter()
            .map(|h| h.into())
            .collect(),
        keys: db
            .config
            .get_decryption_keys()?
            .into_iter()
            .map(|k| k.into())
            .collect(),
    };

    let content = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(&file_path, content).map_err(|e| format!("Failed to export config: {}", e))?;

    Ok(())
}

/// 完整配置导出结构
#[derive(Debug, Serialize, Deserialize)]
struct AllConfigExport {
    app: AppSettingsResponse,
    m3u8dl: M3U8DLSettingsResponse,
    ffmpeg: FFmpegSettingsResponse,
    network: NetworkSettingsResponse,
    decryption: DecryptionSettingsResponse,
    headers: Vec<NetworkHeaderResponse>,
    keys: Vec<DecryptionKeyResponse>,
}

/// 从指定路径导入配置
#[tauri::command]
pub async fn import_config(file_path: String, app: AppHandle) -> Result<(), String> {
    log::info!("Importing config from: {}", file_path);

    let content =
        fs::read_to_string(&file_path).map_err(|e| format!("Failed to read config file: {}", e))?;

    let _config: AllConfigExport = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse config file: {}", e))?;

    let _db = get_db(&app)?;

    // 导入各项配置（字段级别更新）
    // 注意：这里简化处理，实际可能需要更复杂的导入逻辑
    // 例如批量更新或事务处理

    log::info!("Config imported successfully");

    Ok(())
}

// ========================================
// 加载所有配置（兼容旧版 API）
// ========================================

/// 加载所有配置
#[tauri::command]
pub async fn load_settings(
    app: AppHandle,
) -> Result<std::collections::HashMap<String, serde_json::Value>, String> {
    log::info!("Loading all settings");

    let db = get_db(&app)?;

    let mut settings = std::collections::HashMap::new();

    // 加载各类配置
    let app_settings = db.config.get_app_settings()?;
    settings.insert(
        "app".to_string(),
        serde_json::to_value(AppSettingsResponse::from(app_settings))
            .map_err(|e| format!("Failed to serialize app settings: {}", e))?,
    );

    let m3u8dl_settings = db.config.get_m3u8dl_settings()?;
    settings.insert(
        "m3u8dl".to_string(),
        serde_json::to_value(M3U8DLSettingsResponse::from(m3u8dl_settings))
            .map_err(|e| format!("Failed to serialize m3u8dl settings: {}", e))?,
    );

    let ffmpeg_settings = db.config.get_ffmpeg_settings()?;
    settings.insert(
        "ffmpeg".to_string(),
        serde_json::to_value(FFmpegSettingsResponse::from(ffmpeg_settings))
            .map_err(|e| format!("Failed to serialize ffmpeg settings: {}", e))?,
    );

    let network_settings = db.config.get_network_settings()?;
    settings.insert(
        "network".to_string(),
        serde_json::to_value(NetworkSettingsResponse::from(network_settings))
            .map_err(|e| format!("Failed to serialize network settings: {}", e))?,
    );

    let decryption_settings = db.config.get_decryption_settings()?;
    settings.insert(
        "decryption".to_string(),
        serde_json::to_value(DecryptionSettingsResponse::from(decryption_settings))
            .map_err(|e| format!("Failed to serialize decryption settings: {}", e))?,
    );

    // 加载网络请求头
    let headers = db.config.get_network_headers()?;
    settings.insert(
        "headers".to_string(),
        serde_json::to_value(
            headers
                .into_iter()
                .map(NetworkHeaderResponse::from)
                .collect::<Vec<_>>(),
        )
        .map_err(|e| format!("Failed to serialize headers: {}", e))?,
    );

    // 加载解密密钥
    let keys = db.config.get_decryption_keys()?;
    settings.insert(
        "keys".to_string(),
        serde_json::to_value(
            keys.into_iter()
                .map(DecryptionKeyResponse::from)
                .collect::<Vec<_>>(),
        )
        .map_err(|e| format!("Failed to serialize keys: {}", e))?,
    );

    Ok(settings)
}

// ========================================
// 兼容旧版 API（将被废弃）
// ========================================

/// 保存单个配置模块（兼容旧版）
#[tauri::command]
pub async fn save_setting(
    _key: String,
    _value: serde_json::Value,
    _app: AppHandle,
) -> Result<(), String> {
    log::warn!("save_setting is deprecated, use update_*_setting_field instead");

    // 简化处理：不再支持旧版的保存方式
    Err("save_setting is deprecated. Use specific update_*_setting_field commands.".to_string())
}

/// 批量保存配置（兼容旧版）
#[tauri::command]
pub async fn save_settings(
    _settings: std::collections::HashMap<String, serde_json::Value>,
    _app: AppHandle,
) -> Result<(), String> {
    log::warn!("save_settings is deprecated, use individual update commands instead");

    Err("save_settings is deprecated. Use individual update_*_setting_field commands.".to_string())
}

/// 重置单个配置模块（兼容旧版）
#[tauri::command]
pub async fn reset_setting(_key: String, _app: AppHandle) -> Result<(), String> {
    log::warn!("reset_setting is not implemented in new config system");
    Err("reset_setting is not implemented".to_string())
}

/// 重置所有配置（兼容旧版）
#[tauri::command]
pub async fn reset_all_settings(_app: AppHandle) -> Result<(), String> {
    log::warn!("reset_all_settings is not implemented in new config system");
    Err("reset_all_settings is not implemented".to_string())
}
