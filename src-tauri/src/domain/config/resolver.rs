//! 配置解析器
//!
//! 合并配置继承链：任务配置 > 模板配置 > 全局配置 > 默认配置

use super::entity::*;
use crate::infrastructure::db::repository::{ConfigRepository, TemplateRepository};

/// 配置解析器
///
/// 负责合并配置继承链
pub struct ConfigResolver<'a> {
    config_repo: &'a ConfigRepository,
    template_repo: &'a TemplateRepository,
}

impl<'a> ConfigResolver<'a> {
    /// 创建配置解析器
    pub fn new(config_repo: &'a ConfigRepository, template_repo: &'a TemplateRepository) -> Self {
        Self { config_repo, template_repo }
    }

    /// 解析任务配置
    ///
    /// 合并顺序：任务配置 > 模板配置 > 全局配置 > 默认配置
    pub fn resolve_task_config(
        &self,
        task_id: &str,
        downloader_type: DownloaderType,
        template_id: Option<&str>,
        task_overrides: Option<&TaskConfigOverrides>,
    ) -> Result<ResolvedConfig, String> {
        // 1. 加载全局配置
        let global_config = self.load_global_config()?;

        // 2. 应用模板覆盖（如果有）
        let config_with_template = if let Some(tid) = template_id {
            self.apply_template_overrides(&global_config, tid, downloader_type)?
        } else {
            global_config
        };

        // 3. 应用任务覆盖（如果有）
        let resolved_config = if let Some(overrides) = task_overrides {
            self.apply_task_overrides(&config_with_template, overrides, downloader_type)?
        } else {
            config_with_template
        };

        // 4. 设置任务特定值
        let mut result = resolved_config;
        result.task = TaskSpecificConfig {
            save_dir: task_overrides
                .and_then(|o| o.save_dir.clone())
                .unwrap_or_else(|| result.app.default_save_dir.clone()),
            save_name: task_overrides
                .and_then(|o| o.save_name.clone())
                .unwrap_or_default(),
            save_pattern: task_overrides.and_then(|o| o.save_pattern.clone()),
            custom_range: None, // 从任务数据库加载
            start_at: None,     // 从任务数据库加载
        };

        Ok(result)
    }

    /// 加载全局配置
    fn load_global_config(&self) -> Result<ResolvedConfig, String> {
        let app = self.config_repo.get_app_settings()?;
        let m3u8dl = self.config_repo.get_m3u8dl_settings()?;
        let ffmpeg = self.config_repo.get_ffmpeg_settings()?;
        let network = self.config_repo.get_network_settings()?;
        let decryption = self.config_repo.get_decryption_settings()?;

        Ok(ResolvedConfig {
            downloader_type: DownloaderType::default(),
            template_id: None,
            app: self.convert_app_settings(&app),
            m3u8dl: self.convert_m3u8dl_settings(&m3u8dl),
            ffmpeg: self.convert_ffmpeg_settings(&ffmpeg),
            network: self.convert_network_settings(&network),
            decryption: self.convert_decryption_settings(&decryption),
            task: TaskSpecificConfig::default(),
        })
    }

    /// 应用模板覆盖
    fn apply_template_overrides(
        &self,
        base: &ResolvedConfig,
        template_id: &str,
        downloader_type: DownloaderType,
    ) -> Result<ResolvedConfig, String> {
        let template = self
            .template_repo
            .get_template(template_id)?
            .ok_or_else(|| format!("Template not found: {}", template_id))?;

        let mut result = base.clone();
        result.template_id = Some(template_id.to_string());
        result.downloader_type = template.downloader_type.clone();

        // 根据下载器类型应用覆盖
        match downloader_type {
            DownloaderType::M3U8DL => {
                if let Some(m3u8dl_overrides) =
                    self.template_repo.get_template_m3u8dl(template_id)?
                {
                    self.merge_m3u8dl_overrides(&mut result.m3u8dl, &m3u8dl_overrides);
                }
            }
            DownloaderType::FFmpeg => {
                if let Some(ffmpeg_overrides) =
                    self.template_repo.get_template_ffmpeg(template_id)?
                {
                    self.merge_ffmpeg_overrides(&mut result.ffmpeg, &ffmpeg_overrides);
                }
            }
        }

        // 应用网络覆盖
        if let Some(network_overrides) =
            self.template_repo.get_template_network(template_id)?
        {
            self.merge_network_overrides(&mut result.network, &network_overrides);
        }

        // 应用解密覆盖
        if let Some(decryption_overrides) =
            self.template_repo.get_template_decryption(template_id)?
        {
            self.merge_decryption_overrides(&mut result.decryption, &decryption_overrides);
        }

        // 替换请求头（模板请求头完全替换全局请求头）
        let template_headers = self.template_repo.get_template_headers(template_id)?;
        if !template_headers.is_empty() {
            result.network.headers = template_headers
                .into_iter()
                .map(|h| HeaderConfig {
                    name: h.name,
                    value: h.value,
                    enabled: h.enabled,
                    sort_order: h.sort_order,
                })
                .collect();
        }

        // 替换解密密钥
        let template_keys = self.template_repo.get_template_keys(template_id)?;
        if !template_keys.is_empty() {
            result.decryption.keys = template_keys
                .into_iter()
                .map(|k| DecryptionKey {
                    kid: k.kid,
                    key: k.key,
                    sort_order: k.sort_order,
                })
                .collect();
        }

        Ok(result)
    }

    /// 应用任务覆盖
    fn apply_task_overrides(
        &self,
        base: &ResolvedConfig,
        overrides: &TaskConfigOverrides,
        downloader_type: DownloaderType,
    ) -> Result<ResolvedConfig, String> {
        let mut result = base.clone();

        // 根据下载器类型应用覆盖
        match downloader_type {
            DownloaderType::M3U8DL => {
                if let Some(m3u8dl_overrides) = &overrides.m3u8dl {
                    self.merge_partial_m3u8dl(&mut result.m3u8dl, m3u8dl_overrides);
                }
            }
            DownloaderType::FFmpeg => {
                if let Some(ffmpeg_overrides) = &overrides.ffmpeg {
                    self.merge_partial_ffmpeg(&mut result.ffmpeg, ffmpeg_overrides);
                }
            }
        }

        // 追加任务请求头（不替换）
        for header in &overrides.headers {
            result.network.headers.push(header.clone());
        }

        Ok(result)
    }

    // ========================================
    // 类型转换方法
    // ========================================

    fn convert_app_settings(&self, db: &crate::infrastructure::db::repository::config_repo::AppSettings) -> AppSettings {
        AppSettings {
            language: db.language.parse().unwrap_or_default(),
            auto_start_download: db.auto_start_download,
            minimize_to_tray: db.minimize_to_tray,
            check_update: db.check_update,
            default_save_dir: db.default_save_dir.clone(),
            default_tmp_dir: db.default_tmp_dir.clone(),
            theme: db.theme.parse().unwrap_or_default(),
            show_notification: db.show_notification,
            clipboard_watch: db.clipboard_watch,
            log_level: db.log_level.parse().unwrap_or_default(),
            log_file_path: db.log_file_path.clone(),
            no_log: db.no_log,
        }
    }

    fn convert_m3u8dl_settings(&self, db: &crate::infrastructure::db::repository::config_repo::M3U8DLSettings) -> M3U8DLSettings {
        M3U8DLSettings {
            n_m3u8dl_path: db.n_m3u8dl_path.clone(),
            thread_count: db.thread_count,
            retry_count: db.retry_count,
            timeout: db.timeout,
            max_speed: db.max_speed.clone(),
            auto_select: db.auto_select,
            select_video: db.select_video.clone(),
            select_audio: db.select_audio.clone(),
            select_subtitle: db.select_subtitle.clone(),
            drop_video: db.drop_video.clone(),
            drop_audio: db.drop_audio.clone(),
            drop_subtitle: db.drop_subtitle.clone(),
            check_segments_count: db.check_segments_count,
            del_after_done: db.del_after_done,
            skip_merge: db.skip_merge,
            write_meta_json: db.write_meta_json,
            binary_merge: db.binary_merge,
            concurrent_download: db.concurrent_download,
            mux_format: db.mux_format.parse().unwrap_or_default(),
            muxer: db.muxer.parse().unwrap_or_default(),
            mux_bin_path: db.mux_bin_path.clone(),
            mux_skip_subtitles: db.mux_skip_subtitles,
            mux_keep_original: db.mux_keep_original,
            sub_only: db.sub_only,
            sub_format: db.sub_format.parse().unwrap_or_default(),
            auto_subtitle_fix: db.auto_subtitle_fix,
            live_perform_as_vod: db.live_perform_as_vod,
            live_real_time_merge: db.live_real_time_merge,
            live_keep_segments: db.live_keep_segments,
            live_pipe_mux: db.live_pipe_mux,
            live_fix_vtt_by_audio: db.live_fix_vtt_by_audio,
            live_record_limit: db.live_record_limit.clone(),
            live_wait_time: db.live_wait_time,
            live_take_count: db.live_take_count,
            allow_hls_multi_ext_map: db.allow_hls_multi_ext_map,
            url_processor_args: db.url_processor_args.clone(),
            no_date_info: db.no_date_info,
            use_ffmpeg_concat_demuxer: db.use_ffmpeg_concat_demuxer,
        }
    }

    fn convert_ffmpeg_settings(&self, db: &crate::infrastructure::db::repository::config_repo::FFmpegSettings) -> FFmpegSettings {
        FFmpegSettings {
            ffmpeg_path: db.ffmpeg_path.clone(),
            ffprobe_path: db.ffprobe_path.clone(),
            retry_count: db.retry_count,
            timeout: db.timeout,
            max_speed: db.max_speed.clone(),
            connection_timeout: db.connection_timeout,
            reconnect_attempts: db.reconnect_attempts,
            reconnect_delay: db.reconnect_delay,
            overwrite_existing: db.overwrite_existing,
            preserve_timestamps: db.preserve_timestamps,
            user_agent: db.user_agent.clone(),
            referer: db.referer.clone(),
        }
    }

    fn convert_network_settings(&self, db: &crate::infrastructure::db::repository::config_repo::NetworkSettings) -> NetworkSettings {
        let headers = self.config_repo.get_network_headers().unwrap_or_default();

        NetworkSettings {
            use_system_proxy: db.use_system_proxy,
            custom_proxy: db.custom_proxy.clone(),
            base_url: db.base_url.clone(),
            append_url_params: db.append_url_params,
            headers: headers
                .into_iter()
                .map(|h| HeaderConfig {
                    id: 0, // 新创建的 HeaderConfig 没有 id 字段
                    name: h.name,
                    value: h.value,
                    enabled: h.enabled,
                    sort_order: h.sort_order,
                })
                .collect(),
        }
    }

    fn convert_decryption_settings(&self, db: &crate::infrastructure::db::repository::config_repo::DecryptionSettings) -> DecryptionSettings {
        let keys = self.config_repo.get_decryption_keys().unwrap_or_default();

        DecryptionSettings {
            key_text_file: db.key_text_file.clone(),
            decryption_engine: db.decryption_engine.parse().unwrap_or_default(),
            decryption_bin_path: db.decryption_bin_path.clone(),
            real_time_decryption: db.real_time_decryption,
            custom_hls: CustomHlsDecryption {
                enabled: db.custom_hls_enabled,
                method: db.custom_hls_method.parse().unwrap_or_default(),
                key: KeyValue {
                    value_type: db.custom_hls_key_type.parse().unwrap_or_default(),
                    value: db.custom_hls_key_value.clone().unwrap_or_default(),
                },
                iv: KeyValue {
                    value_type: db.custom_hls_iv_type.parse().unwrap_or_default(),
                    value: db.custom_hls_iv_value.clone().unwrap_or_default(),
                },
            },
            keys: keys
                .into_iter()
                .map(|k| DecryptionKey {
                    id: 0, // 新创建的 DecryptionKey 没有 id 字段
                    kid: k.kid,
                    key: k.key,
                    sort_order: k.sort_order,
                })
                .collect(),
        }
    }

    // ========================================
    // 合并方法
    // ========================================

    fn merge_m3u8dl_overrides(
        &self,
        base: &mut M3U8DLSettings,
        overrides: &crate::infrastructure::db::repository::template_repo::TemplateM3U8DL,
    ) {
        if let Some(v) = overrides.thread_count {
            base.thread_count = v;
        }
        if let Some(v) = overrides.retry_count {
            base.retry_count = v;
        }
        if let Some(v) = overrides.timeout {
            base.timeout = v;
        }
        if let Some(ref v) = overrides.max_speed {
            base.max_speed = v.clone();
        }
        if let Some(v) = overrides.auto_select {
            base.auto_select = v;
        }
        if let Some(ref v) = overrides.select_video {
            base.select_video = Some(v.clone());
        }
        if let Some(ref v) = overrides.select_audio {
            base.select_audio = Some(v.clone());
        }
        if let Some(ref v) = overrides.select_subtitle {
            base.select_subtitle = Some(v.clone());
        }
        // ... 其他字段的合并
    }

    fn merge_ffmpeg_overrides(
        &self,
        base: &mut FFmpegSettings,
        overrides: &crate::infrastructure::db::repository::template_repo::TemplateFFmpeg,
    ) {
        if let Some(ref v) = overrides.ffmpeg_path {
            base.ffmpeg_path = v.clone();
        }
        if let Some(ref v) = overrides.ffprobe_path {
            base.ffprobe_path = v.clone();
        }
        if let Some(v) = overrides.retry_count {
            base.retry_count = v;
        }
        if let Some(v) = overrides.timeout {
            base.timeout = v;
        }
        // ... 其他字段的合并
    }

    fn merge_network_overrides(
        &self,
        base: &mut NetworkSettings,
        overrides: &crate::infrastructure::db::repository::template_repo::TemplateNetwork,
    ) {
        if let Some(v) = overrides.use_system_proxy {
            base.use_system_proxy = v;
        }
        if let Some(ref v) = overrides.custom_proxy {
            base.custom_proxy = Some(v.clone());
        }
        if let Some(ref v) = overrides.base_url {
            base.base_url = Some(v.clone());
        }
        if let Some(v) = overrides.append_url_params {
            base.append_url_params = v;
        }
    }

    fn merge_decryption_overrides(
        &self,
        base: &mut DecryptionSettings,
        overrides: &crate::infrastructure::db::repository::template_repo::TemplateDecryption,
    ) {
        if let Some(ref v) = overrides.key_text_file {
            base.key_text_file = Some(v.clone());
        }
        if let Some(ref v) = overrides.decryption_engine {
            base.decryption_engine = v.parse().unwrap_or_default();
        }
        if let Some(ref v) = overrides.decryption_bin_path {
            base.decryption_bin_path = Some(v.clone());
        }
        if let Some(v) = overrides.real_time_decryption {
            base.real_time_decryption = v;
        }
        // ... 其他字段的合并
    }

    fn merge_partial_m3u8dl(&self, base: &mut M3U8DLSettings, overrides: &PartialM3U8DLSettings) {
        if let Some(v) = overrides.thread_count {
            base.thread_count = v;
        }
        if let Some(v) = overrides.retry_count {
            base.retry_count = v;
        }
        if let Some(v) = overrides.timeout {
            base.timeout = v;
        }
        // ... 其他字段的合并
    }

    fn merge_partial_ffmpeg(&self, base: &mut FFmpegSettings, overrides: &PartialFFmpegSettings) {
        if let Some(ref v) = &overrides.ffmpeg_path {
            base.ffmpeg_path = v.clone();
        }
        if let Some(v) = overrides.retry_count {
            base.retry_count = v;
        }
        if let Some(v) = overrides.timeout {
            base.timeout = v;
        }
        // ... 其他字段的合并
    }
}

impl Default for TaskSpecificConfig {
    fn default() -> Self {
        Self {
            save_dir: String::new(),
            save_name: String::new(),
            save_pattern: None,
            custom_range: None,
            start_at: None,
        }
    }
}
