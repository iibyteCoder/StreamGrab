//! M3U8DL 下载器实现
//!
//! 使用 N_m3u8DL-RE 下载流媒体

use super::{DownloadHandle, Downloader, MediaInfo, ProgressData, ResolvedConfig};
use crate::domain::config::value_objects::*;

/// M3U8DL 下载器
pub struct M3U8DLDownloader;

impl M3U8DLDownloader {
    /// 创建新实例
    pub fn new() -> Self {
        Self
    }

    /// 检测是否为流媒体 URL
    fn is_streaming_url(url: &str) -> bool {
        let lower = url.to_lowercase();
        lower.contains(".m3u8")
            || lower.contains(".mpd")
            || lower.contains(".ism")
            || lower.contains("m3u8")
            || lower.contains("mpd")
            || lower.contains("manifest")
            || lower.contains("playlist")
            || lower.contains("master")
    }

    /// 构建基本参数
    fn build_base_args(&self, url: &str, config: &ResolvedConfig) -> Vec<String> {
        let mut args = vec![url.to_string()];

        // 基础路径
        if !config.task.save_dir.is_empty() {
            args.push("--save-dir".to_string());
            args.push(config.task.save_dir.clone());
        }
        if !config.task.save_name.is_empty() {
            args.push("--save-name".to_string());
            args.push(config.task.save_name.clone());
        }

        // 临时目录
        if !config.app.default_tmp_dir.is_empty() {
            args.push("--tmp-dir".to_string());
            args.push(config.app.default_tmp_dir.clone());
        }

        args
    }

    /// 构建下载参数
    fn build_download_args(&self, args: &mut Vec<String>, config: &ResolvedConfig) {
        let m = &config.m3u8dl;

        // 线程数
        if m.thread_count != 8 {
            args.push("--thread-count".to_string());
            args.push(m.thread_count.to_string());
        }

        // 重试次数
        if m.retry_count != 3 {
            args.push("--download-retry-count".to_string());
            args.push(m.retry_count.to_string());
        }

        // 超时
        if m.timeout != 100 {
            args.push("--http-request-timeout".to_string());
            args.push(m.timeout.to_string());
        }

        // 限速
        if !m.max_speed.is_empty() && m.max_speed != "0" {
            args.push("-R".to_string());
            args.push(m.max_speed.clone());
        }

        // 流选择
        if m.auto_select {
            args.push("--auto-select".to_string());
        }
        if let Some(ref v) = &m.select_video {
            if !v.is_empty() {
                args.push("-sv".to_string());
                args.push(v.clone());
            }
        }
        if let Some(ref v) = &m.select_audio {
            if !v.is_empty() {
                args.push("-sa".to_string());
                args.push(v.clone());
            }
        }
        if let Some(ref v) = &m.select_subtitle {
            if !v.is_empty() {
                args.push("-ss".to_string());
                args.push(v.clone());
            }
        }

        // 流排除
        if let Some(ref v) = &m.drop_video {
            if !v.is_empty() {
                args.push("-dv".to_string());
                args.push(v.clone());
            }
        }
        if let Some(ref v) = &m.drop_audio {
            if !v.is_empty() {
                args.push("-da".to_string());
                args.push(v.clone());
            }
        }
        if let Some(ref v) = &m.drop_subtitle {
            if !v.is_empty() {
                args.push("-ds".to_string());
                args.push(v.clone());
            }
        }

        // 下载行为
        if m.skip_merge {
            args.push("--skip-merge".to_string());
        }
        if !m.del_after_done {
            args.push("--no-delete-temp".to_string());
        }
        if !m.check_segments_count {
            args.push("--check-segments-count".to_string());
            args.push("false".to_string());
        }
        if m.binary_merge {
            args.push("--binary-merge".to_string());
        }
        if m.write_meta_json {
            args.push("--write-meta-json".to_string());
        }
        if m.concurrent_download {
            args.push("-mt".to_string());
        }
    }

    /// 构建混流参数
    fn build_mux_args(&self, args: &mut Vec<String>, config: &ResolvedConfig) {
        let m = &config.m3u8dl;

        if m.skip_merge {
            return;
        }

        // 混流选项
        let mut mux_parts = vec![format!("format={}", m.mux_format)];

        if m.muxer != Muxer::FFmpeg {
            mux_parts.push(format!("muxer={}", m.muxer));
        }
        if let Some(ref path) = m.mux_bin_path {
            if !path.is_empty() {
                mux_parts.push(format!("bin_path=\"{}\"", path));
            }
        }
        if m.mux_skip_subtitles {
            mux_parts.push("skip_sub=true".to_string());
        }
        if m.mux_keep_original {
            mux_parts.push("keep=true".to_string());
        }

        args.push("-M".to_string());
        args.push(mux_parts.join(":"));

        // 其他混流相关
        if m.no_date_info {
            args.push("--no-date-info".to_string());
        }
        if m.use_ffmpeg_concat_demuxer {
            args.push("--use-ffmpeg-concat-demuxer".to_string());
        }

        // 外部媒体导入
        for imp in &m.mux_imports {
            let mut imp_parts = vec![format!("path=\"{}\"", imp.path)];
            if let Some(ref lang) = imp.lang {
                imp_parts.push(format!("lang={}", lang));
            }
            if let Some(ref name) = imp.name {
                imp_parts.push(format!("name=\"{}\"", name));
            }
            args.push("--mux-import".to_string());
            args.push(imp_parts.join(":"));
        }
    }

    /// 构建字幕参数
    fn build_subtitle_args(&self, args: &mut Vec<String>, config: &ResolvedConfig) {
        let m = &config.m3u8dl;

        if m.sub_only {
            args.push("--sub-only".to_string());
        }
        if m.sub_format != SubtitleFormat::SRT {
            args.push("--sub-format".to_string());
            args.push(m.sub_format.to_string());
        }
        if m.auto_subtitle_fix {
            args.push("--auto-subtitle-fix".to_string());
        }
    }

    /// 构建直播参数
    fn build_live_args(&self, args: &mut Vec<String>, config: &ResolvedConfig) {
        let m = &config.m3u8dl;

        if m.live_perform_as_vod {
            args.push("--live-perform-as-vod".to_string());
        }
        if m.live_real_time_merge {
            args.push("--live-real-time-merge".to_string());
        }
        if !m.live_keep_segments {
            args.push("--live-keep-segments".to_string());
            args.push("false".to_string());
        }
        if m.live_pipe_mux {
            args.push("--live-pipe-mux".to_string());
        }
        if m.live_fix_vtt_by_audio {
            args.push("--live-fix-vtt-by-audio".to_string());
        }
        if let Some(ref limit) = m.live_record_limit {
            if !limit.is_empty() {
                args.push("--live-record-limit".to_string());
                args.push(limit.clone());
            }
        }
        if m.live_wait_time > 0 {
            args.push("--live-wait-time".to_string());
            args.push(m.live_wait_time.to_string());
        }
        if m.live_take_count != 16 {
            args.push("--live-take-count".to_string());
            args.push(m.live_take_count.to_string());
        }
    }

    /// 构建网络参数
    fn build_network_args(&self, args: &mut Vec<String>, config: &ResolvedConfig) {
        let n = &config.network;

        if n.use_system_proxy {
            args.push("--use-system-proxy".to_string());
        } else if let Some(ref proxy) = n.custom_proxy {
            if !proxy.is_empty() {
                args.push("--custom-proxy".to_string());
                args.push(proxy.clone());
            }
        }

        // 请求头
        for header in n.headers.iter().filter(|h| h.enabled) {
            args.push("-H".to_string());
            args.push(format!("{}: {}", header.name, header.value));
        }

        if let Some(ref base_url) = n.base_url {
            if !base_url.is_empty() {
                args.push("--base-url".to_string());
                args.push(base_url.clone());
            }
        }
        if n.append_url_params {
            args.push("--append-url-params".to_string());
        }
    }

    /// 构建解密参数
    fn build_decryption_args(&self, args: &mut Vec<String>, config: &ResolvedConfig) {
        let d = &config.decryption;

        // 密钥
        for key in &d.keys {
            if let Some(ref kid) = key.kid {
                args.push("--key".to_string());
                args.push(format!("{}:{}", kid, key.key));
            } else {
                args.push("--key".to_string());
                args.push(key.key.clone());
            }
        }

        if let Some(ref file) = d.key_text_file {
            if !file.is_empty() {
                args.push("--key-text-file".to_string());
                args.push(file.clone());
            }
        }
        if d.decryption_engine != DecryptionEngine::MP4Decrypt {
            args.push("--decryption-engine".to_string());
            args.push(d.decryption_engine.to_string());
        }
        if let Some(ref path) = d.decryption_bin_path {
            if !path.is_empty() {
                args.push("--decryption-binary-path".to_string());
                args.push(path.clone());
            }
        }
        if d.real_time_decryption {
            args.push("--mp4-real-time-decryption".to_string());
        }

        // 自定义 HLS 解密
        let ch = &d.custom_hls;
        if ch.enabled {
            if ch.method != HlsEncryptionMethod::UNKNOWN {
                args.push("--custom-hls-method".to_string());
                args.push(ch.method.to_string());
            }
            if !ch.key.value.is_empty() {
                args.push("--custom-hls-key".to_string());
                args.push(ch.key.value.clone());
            }
            if !ch.iv.value.is_empty() {
                args.push("--custom-hls-iv".to_string());
                args.push(ch.iv.value.clone());
            }
        }
    }

    /// 构建高级参数
    fn build_advanced_args(&self, args: &mut Vec<String>, config: &ResolvedConfig) {
        let m = &config.m3u8dl;

        if m.allow_hls_multi_ext_map {
            args.push("--allow-hls-multi-ext-map".to_string());
        }
        if let Some(ref args_str) = m.url_processor_args {
            if !args_str.is_empty() {
                args.push("--urlprocessor-args".to_string());
                args.push(args_str.clone());
            }
        }

        // 任务特定
        if let Some(ref range) = config.task.custom_range {
            if !range.is_empty() {
                args.push("--custom-range".to_string());
                args.push(range.clone());
            }
        }
        if let Some(ref start_at) = config.task.start_at {
            if !start_at.is_empty() {
                args.push("--task-start-at".to_string());
                args.push(start_at.clone());
            }
        }
    }
}

impl Default for M3U8DLDownloader {
    fn default() -> Self {
        Self::new()
    }
}

impl Downloader for M3U8DLDownloader {
    fn detect(&self, url: &str) -> bool {
        Self::is_streaming_url(url)
    }

    fn parse(&self, url: &str, config: &ResolvedConfig) -> Result<MediaInfo, String> {
        // 解析时使用 --skip-download
        let args = self.build_cmd(url, config);
        let parse_args = {
            let mut args = args;
            args.push("--skip-download".to_string());
            args.push("--auto-select".to_string());
            args
        };

        // TODO: 调用进程并解析输出
        // 这部分需要与进程管理器集成
        Err("Not implemented yet".to_string())
    }

    fn download(
        &self,
        url: &str,
        config: &ResolvedConfig,
        on_progress: Option<Box<dyn Fn(ProgressData) + Send + Sync>>,
    ) -> Result<DownloadHandle, String> {
        // TODO: 调用进程管理器执行下载
        // 这部分需要与进程管理器集成
        Err("Not implemented yet".to_string())
    }

    fn build_cmd(&self, url: &str, config: &ResolvedConfig) -> Vec<String> {
        let mut args = self.build_base_args(url, config);
        self.build_download_args(&mut args, config);
        self.build_mux_args(&mut args, config);
        self.build_subtitle_args(&mut args, config);
        self.build_live_args(&mut args, config);
        self.build_network_args(&mut args, config);
        self.build_decryption_args(&mut args, config);
        self.build_advanced_args(&mut args, config);
        args
    }
}
