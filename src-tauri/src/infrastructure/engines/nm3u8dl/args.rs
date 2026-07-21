//! N_m3u8DL-RE 命令行参数构建
//!
//! 「默认值 + 覆盖」合并的唯一实现点：任务级覆盖（非空）> 全局默认。
//! 移植自前端 `src/utils/commandBuilder.ts`（重构后前端不再构建 CLI 参数）。

use crate::domain::config::{
    AppSettings, DecryptionConfig, HlsEncryptionMethod, MuxFormat, NetworkConfig, ToolConfigs,
};
use crate::domain::task::TaskSpec;
use serde::Serialize;

/// 枚举 → CLI 字符串（复用 serde 的 rename 定义，单一来源）
fn enum_str<T: Serialize>(v: &T) -> String {
    serde_json::to_value(v)
        .ok()
        .and_then(|j| j.as_str().map(String::from))
        .unwrap_or_default()
}

/// 构建下载命令参数
///
/// `ffmpeg_bin`：已解析的 FFmpeg 二进制路径（混流需要；None 时不注入相关参数）
pub fn build_download_args(
    spec: &TaskSpec,
    tools: &ToolConfigs,
    app: &AppSettings,
    ffmpeg_bin: Option<&str>,
) -> Vec<String> {
    let cfg = &tools.nm3u8dl;
    let ov = &spec.overrides;
    let mut args: Vec<String> = vec![spec.url.clone()];

    // === 基础参数 ===
    if !spec.save_dir.is_empty() {
        args.extend(["--save-dir".into(), spec.save_dir.clone()]);
    }
    if !spec.file_name.is_empty() {
        args.extend(["--save-name".into(), spec.file_name.clone()]);
    }
    // 临时目录：全局 tmp 目录优先，否则跟随保存目录
    let tmp_dir = if !app.default_tmp_dir.is_empty() {
        app.default_tmp_dir.clone()
    } else {
        spec.save_dir.clone()
    };
    if !tmp_dir.is_empty() {
        args.extend(["--tmp-dir".into(), tmp_dir]);
    }

    // === 下载参数（仅在与工具默认值不同时输出，保持命令行精简）===
    if cfg.thread_count != 8 {
        args.extend(["--thread-count".into(), cfg.thread_count.to_string()]);
    }
    if cfg.retry_count != 3 {
        args.extend(["--download-retry-count".into(), cfg.retry_count.to_string()]);
    }
    if cfg.timeout != 100 {
        args.extend(["--http-request-timeout".into(), cfg.timeout.to_string()]);
    }
    // 限速：任务覆盖 > 全局默认（"0"/空 = 不限速）
    let max_speed = first_non_empty(ov.max_speed.as_deref(), cfg.max_speed.as_str());
    if let Some(speed) = max_speed.filter(|s| *s != "0") {
        args.extend(["-R".into(), speed.to_string()]);
    }

    // === 流选择：任务覆盖 > 全局默认 ===
    if cfg.auto_select {
        args.push("--auto-select".into());
    }
    let selection = ov.selection.as_ref();
    push_select(
        &mut args,
        "-sv",
        selection.and_then(|s| s.video.as_deref()),
        cfg.select_video.as_deref(),
    );
    push_select(
        &mut args,
        "-sa",
        selection.and_then(|s| s.audio.as_deref()),
        cfg.select_audio.as_deref(),
    );
    push_select(
        &mut args,
        "-ss",
        selection.and_then(|s| s.subtitle.as_deref()),
        cfg.select_subtitle.as_deref(),
    );

    // === 流排除 ===
    push_if_some(&mut args, "-dv", cfg.drop_video.as_deref());
    push_if_some(&mut args, "-da", cfg.drop_audio.as_deref());
    push_if_some(&mut args, "-ds", cfg.drop_subtitle.as_deref());

    // === 混流（混流默认值归属 FFmpeg 工具配置）===
    if !cfg.skip_merge {
        let mux_format = ov.mux_format.unwrap_or(tools.ffmpeg.mux_format);
        args.extend([
            "-M".into(),
            build_mux_options(mux_format, tools, ffmpeg_bin),
        ]);
    }
    if cfg.no_date_info {
        args.push("--no-date-info".into());
    }
    if cfg.use_ffmpeg_concat_demuxer {
        args.push("--use-ffmpeg-concat-demuxer".into());
    }

    // === 网络 ===
    append_network_args(&mut args, &cfg.network);

    // === 其他下载选项 ===
    if cfg.skip_merge {
        args.push("--skip-merge".into());
    }
    if !cfg.del_after_done {
        args.push("--no-delete-temp".into());
    }
    if !cfg.check_segments_count {
        args.extend(["--check-segments-count".into(), "false".into()]);
    }
    if cfg.binary_merge {
        args.push("--binary-merge".into());
    }
    if cfg.write_meta_json {
        args.push("--write-meta-json".into());
    }
    if cfg.concurrent_download {
        args.push("-mt".into());
    }

    // === 字幕 ===
    if ov.subtitles_only.unwrap_or(cfg.sub_only) {
        args.push("--sub-only".into());
    }
    let sub_format = ov.subtitle_format.unwrap_or(cfg.sub_format);
    args.extend(["--sub-format".into(), enum_str(&sub_format)]);
    if cfg.auto_subtitle_fix {
        args.push("--auto-subtitle-fix".into());
    }

    // === 解密 ===
    append_decryption_args(&mut args, &cfg.decryption, true);
    // 任务级密钥：全局密钥库为空时生效
    if cfg.decryption.keys.is_empty() {
        push_if_some(&mut args, "--key", ov.key.as_deref());
    }

    // === 直播 ===
    if cfg.live_perform_as_vod {
        args.push("--live-perform-as-vod".into());
    }
    if cfg.live_real_time_merge {
        args.push("--live-real-time-merge".into());
    }
    if !cfg.live_keep_segments {
        args.extend(["--live-keep-segments".into(), "false".into()]);
    }
    if cfg.live_pipe_mux {
        args.push("--live-pipe-mux".into());
    }
    if cfg.live_fix_vtt_by_audio {
        args.push("--live-fix-vtt-by-audio".into());
    }
    push_if_some(
        &mut args,
        "--live-record-limit",
        cfg.live_record_limit.as_deref(),
    );
    if cfg.live_wait_time > 0 {
        args.extend(["--live-wait-time".into(), cfg.live_wait_time.to_string()]);
    }
    if cfg.live_take_count != 16 {
        args.extend(["--live-take-count".into(), cfg.live_take_count.to_string()]);
    }

    // === 范围下载 ===
    push_if_some(&mut args, "--custom-range", ov.custom_range.as_deref());

    // （定时开始由前端调度器统一处理，不传 --task-start-at）

    // === 日志 ===
    append_log_args(&mut args, app);
    if !app.log_file_path.is_empty() {
        args.extend(["--log-file-path".into(), app.log_file_path.clone()]);
    }

    // === 高级 ===
    if cfg.allow_hls_multi_ext_map {
        args.push("--allow-hls-multi-ext-map".into());
    }
    push_if_some(
        &mut args,
        "--urlprocessor-args",
        cfg.url_processor_args.as_deref(),
    );

    // === FFmpeg 二进制（混流/二进制合并需要）===
    if let Some(bin) = ffmpeg_bin {
        args.extend(["--ffmpeg-binary-path".into(), bin.into()]);
    }

    args
}

/// 构建解析模式命令参数（仅解析流信息，不下载）
pub fn build_parse_args(url: &str, tools: &ToolConfigs, app: &AppSettings) -> Vec<String> {
    let mut args: Vec<String> = vec![url.into(), "--skip-download".into(), "--auto-select".into()];
    append_network_args(&mut args, &tools.nm3u8dl.network);
    append_decryption_args(&mut args, &tools.nm3u8dl.decryption, false);
    append_log_args(&mut args, app);
    args
}

/// 混流选项字符串：`format=mp4:muxer=ffmpeg[:bin_path="..."][:skip_sub=true][:keep=true]`
fn build_mux_options(
    mux_format: MuxFormat,
    tools: &ToolConfigs,
    ffmpeg_bin: Option<&str>,
) -> String {
    let ffmpeg = &tools.ffmpeg;
    let mut parts = vec![
        format!("format={}", enum_str(&mux_format)),
        format!("muxer={}", enum_str(&ffmpeg.muxer)),
    ];

    // bin_path：显式配置 > 已解析的 ffmpeg 路径
    let bin_path = ffmpeg.mux_bin_path.as_deref().filter(|s| !s.is_empty());
    if let Some(bin) = bin_path.or(ffmpeg_bin) {
        parts.push(format!("bin_path=\"{bin}\""));
    }
    if ffmpeg.mux_skip_subtitles {
        parts.push("skip_sub=true".into());
    }
    if ffmpeg.mux_keep_original {
        parts.push("keep=true".into());
    }
    parts.join(":")
}

/// 网络参数
fn append_network_args(args: &mut Vec<String>, network: &NetworkConfig) {
    if network.use_system_proxy {
        args.push("--use-system-proxy".into());
    } else if let Some(proxy) = network.custom_proxy.as_deref().filter(|s| !s.is_empty()) {
        args.extend(["--custom-proxy".into(), proxy.into()]);
    }

    let mut headers: Vec<_> = network.headers.iter().filter(|h| h.enabled).collect();
    headers.sort_by_key(|h| h.sort_order);
    for header in headers {
        args.extend(["-H".into(), format!("{}: {}", header.name, header.value)]);
    }

    push_if_some(
        &mut *args,
        "--base-url",
        network.base_url.as_deref().filter(|s| !s.is_empty()),
    );
    if network.append_url_params {
        args.push("--append-url-params".into());
    }
}

/// 解密参数
///
/// `include_real_time`：解析模式下不启用实时解密
fn append_decryption_args(
    args: &mut Vec<String>,
    decryption: &DecryptionConfig,
    include_real_time: bool,
) {
    let mut keys: Vec<_> = decryption.keys.iter().collect();
    keys.sort_by_key(|k| k.sort_order);
    for k in keys {
        match k.kid.as_deref().filter(|s| !s.is_empty()) {
            Some(kid) => args.extend(["--key".into(), format!("{}:{}", kid, k.key)]),
            None => args.extend(["--key".into(), k.key.clone()]),
        }
    }

    push_if_some(
        args,
        "--key-text-file",
        decryption
            .key_text_file
            .as_deref()
            .filter(|s| !s.is_empty()),
    );
    args.extend(["--decryption-engine".into(), enum_str(&decryption.engine)]);
    push_if_some(
        args,
        "--decryption-binary-path",
        decryption.bin_path.as_deref().filter(|s| !s.is_empty()),
    );
    if include_real_time && decryption.real_time_decryption {
        args.push("--mp4-real-time-decryption".into());
    }

    if decryption.custom_hls.enabled {
        if decryption.custom_hls.method != HlsEncryptionMethod::Unknown {
            args.extend([
                "--custom-hls-method".into(),
                enum_str(&decryption.custom_hls.method),
            ]);
        }
        push_if_some(
            args,
            "--custom-hls-key",
            decryption.custom_hls.key_value.as_deref(),
        );
        push_if_some(
            args,
            "--custom-hls-iv",
            decryption.custom_hls.iv_value.as_deref(),
        );
    }
}

/// 日志参数
fn append_log_args(args: &mut Vec<String>, app: &AppSettings) {
    use crate::domain::config::LogLevel;
    if app.no_log {
        args.push("--no-log".into());
    } else if app.log_level != LogLevel::Info {
        args.extend(["--log-level".into(), enum_str(&app.log_level)]);
    }
}

fn push_if_some(args: &mut Vec<String>, flag: &str, value: Option<&str>) {
    if let Some(v) = value.filter(|s| !s.is_empty()) {
        args.extend([flag.into(), v.into()]);
    }
}

fn push_select(
    args: &mut Vec<String>,
    flag: &str,
    override_value: Option<&str>,
    default_value: Option<&str>,
) {
    push_if_some(args, flag, override_value.or(default_value));
}

fn first_non_empty<'a>(a: Option<&'a str>, b: &'a str) -> Option<&'a str> {
    a.filter(|s| !s.is_empty())
        .or_else(|| (!b.is_empty()).then_some(b))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::config::{DecryptionKey, NetworkHeader, SubtitleFormat};
    use crate::domain::download::UrlType;
    use crate::domain::task::{StreamSelection, TaskOverrides};

    fn test_spec(overrides: TaskOverrides) -> TaskSpec {
        TaskSpec {
            task_id: "t1".into(),
            url: "https://example.com/index.m3u8".into(),
            file_name: "episode-01".into(),
            save_dir: "D:/Videos".into(),
            overrides,
            url_type: UrlType::Hls,
        }
    }

    /// 全默认配置：对照前端 commandBuilder 的特征输出
    #[test]
    fn default_config_produces_baseline_args() {
        let tools = ToolConfigs::default();
        let app = AppSettings::default();
        let spec = test_spec(TaskOverrides::default());

        let args = build_download_args(&spec, &tools, &app, None);

        let expected: Vec<String> = vec![
            "https://example.com/index.m3u8",
            "--save-dir",
            "D:/Videos",
            "--save-name",
            "episode-01",
            "--tmp-dir",
            "D:/Videos",
            "--auto-select",
            "-M",
            "format=mp4:muxer=ffmpeg",
            "--use-system-proxy",
            "--sub-format",
            "SRT",
            "--auto-subtitle-fix",
            "--decryption-engine",
            "MP4DECRYPT",
        ]
        .into_iter()
        .map(String::from)
        .collect();
        assert_eq!(args, expected);
    }

    #[test]
    fn deviating_defaults_emit_args() {
        let mut tools = ToolConfigs::default();
        tools.nm3u8dl.thread_count = 16;
        tools.nm3u8dl.retry_count = 5;
        tools.nm3u8dl.timeout = 30;
        let app = AppSettings::default();
        let spec = test_spec(TaskOverrides::default());

        let args = build_download_args(&spec, &tools, &app, None);

        assert!(args
            .windows(2)
            .any(|w| w[0] == "--thread-count" && w[1] == "16"));
        assert!(args
            .windows(2)
            .any(|w| w[0] == "--download-retry-count" && w[1] == "5"));
        assert!(args
            .windows(2)
            .any(|w| w[0] == "--http-request-timeout" && w[1] == "30"));
    }

    #[test]
    fn overrides_take_precedence() {
        let mut tools = ToolConfigs::default();
        tools.nm3u8dl.max_speed = "10M".into();
        tools.nm3u8dl.sub_format = SubtitleFormat::Srt;
        let app = AppSettings::default();
        let spec = test_spec(TaskOverrides {
            max_speed: Some("5M".into()),
            mux_format: Some(MuxFormat::Mkv),
            subtitle_format: Some(SubtitleFormat::Vtt),
            subtitles_only: Some(true),
            custom_range: Some("00:00:00-00:10:00".into()),
            selection: Some(StreamSelection {
                video: Some("res:1080".into()),
                audio: None,
                subtitle: None,
            }),
            ..Default::default()
        });

        let args = build_download_args(&spec, &tools, &app, None);
        let joined = args.join(" ");

        // 任务覆盖 5M 优先于全局 10M
        assert!(joined.contains("-R 5M"), "got: {joined}");
        // 容器覆盖为 mkv
        assert!(joined.contains("-M format=mkv:muxer=ffmpeg"));
        // 字幕格式覆盖
        assert!(joined.contains("--sub-format VTT"));
        assert!(joined.contains("--sub-only"));
        // 范围下载
        assert!(joined.contains("--custom-range 00:00:00-00:10:00"));
        // 流选择覆盖
        assert!(joined.contains("-sv res:1080"));
    }

    #[test]
    fn network_and_decryption_args() {
        let mut tools = ToolConfigs::default();
        tools.nm3u8dl.network.use_system_proxy = false;
        tools.nm3u8dl.network.custom_proxy = Some("http://127.0.0.1:7890".into());
        tools.nm3u8dl.network.base_url = Some("https://cdn.example.com".into());
        tools.nm3u8dl.network.append_url_params = true;
        tools.nm3u8dl.network.headers = vec![
            NetworkHeader {
                id: 1,
                name: "Cookie".into(),
                value: "a=b".into(),
                enabled: true,
                sort_order: 0,
            },
            NetworkHeader {
                id: 2,
                name: "Off".into(),
                value: "x".into(),
                enabled: false,
                sort_order: 1,
            },
            NetworkHeader {
                id: 3,
                name: "Referer".into(),
                value: "https://r.com".into(),
                enabled: true,
                sort_order: 2,
            },
        ];
        tools.nm3u8dl.decryption.keys = vec![DecryptionKey {
            id: 1,
            kid: Some("abc123".into()),
            key: "deadbeef".into(),
            sort_order: 0,
        }];
        tools.nm3u8dl.decryption.real_time_decryption = true;
        let app = AppSettings::default();
        let spec = test_spec(TaskOverrides::default());

        let args = build_download_args(&spec, &tools, &app, None);
        let joined = args.join(" ");

        assert!(joined.contains("--custom-proxy http://127.0.0.1:7890"));
        assert!(!joined.contains("--use-system-proxy"));
        assert!(joined.contains("-H Cookie: a=b"));
        assert!(!joined.contains("Off: x"), "禁用的请求头不应输出");
        assert!(joined.contains("-H Referer: https://r.com"));
        assert!(joined.contains("--base-url https://cdn.example.com"));
        assert!(joined.contains("--append-url-params"));
        assert!(joined.contains("--key abc123:deadbeef"));
        assert!(joined.contains("--mp4-real-time-decryption"));
    }

    #[test]
    fn task_key_used_only_when_no_global_keys() {
        let tools = ToolConfigs::default();
        let app = AppSettings::default();
        let spec = test_spec(TaskOverrides {
            key: Some("11223344".into()),
            ..Default::default()
        });
        let args = build_download_args(&spec, &tools, &app, None);
        assert!(args
            .windows(2)
            .any(|w| w[0] == "--key" && w[1] == "11223344"));
    }

    #[test]
    fn mux_options_include_paths_and_flags() {
        let mut tools = ToolConfigs::default();
        tools.ffmpeg.mux_bin_path = Some("C:/tools/ffmpeg.exe".into());
        tools.ffmpeg.mux_skip_subtitles = true;
        tools.ffmpeg.mux_keep_original = true;
        let app = AppSettings::default();
        let spec = test_spec(TaskOverrides::default());

        let args = build_download_args(&spec, &tools, &app, None);
        let joined = args.join(" ");
        assert!(
            joined.contains(r#"-M format=mp4:muxer=ffmpeg:bin_path="C:/tools/ffmpeg.exe":skip_sub=true:keep=true"#),
            "got: {joined}"
        );
    }

    #[test]
    fn ffmpeg_binary_injected_when_resolved() {
        let tools = ToolConfigs::default();
        let app = AppSettings::default();
        let spec = test_spec(TaskOverrides::default());

        let args = build_download_args(&spec, &tools, &app, Some("C:/ffmpeg/bin/ffmpeg.exe"));
        assert!(args
            .windows(2)
            .any(|w| w[0] == "--ffmpeg-binary-path" && w[1] == "C:/ffmpeg/bin/ffmpeg.exe"));

        let args = build_download_args(&spec, &tools, &app, None);
        assert!(!args.iter().any(|a| a == "--ffmpeg-binary-path"));
    }

    #[test]
    fn live_options_emitted() {
        let mut tools = ToolConfigs::default();
        tools.nm3u8dl.live_real_time_merge = true;
        tools.nm3u8dl.live_keep_segments = false;
        tools.nm3u8dl.live_record_limit = Some("01:00:00".into());
        tools.nm3u8dl.live_wait_time = 30;
        tools.nm3u8dl.live_take_count = 32;
        let app = AppSettings::default();
        let spec = test_spec(TaskOverrides::default());

        let joined = build_download_args(&spec, &tools, &app, None).join(" ");
        assert!(joined.contains("--live-real-time-merge"));
        assert!(joined.contains("--live-keep-segments false"));
        assert!(joined.contains("--live-record-limit 01:00:00"));
        assert!(joined.contains("--live-wait-time 30"));
        assert!(joined.contains("--live-take-count 32"));
    }

    #[test]
    fn log_args_respect_app_settings() {
        let tools = ToolConfigs::default();
        let spec = test_spec(TaskOverrides::default());

        let mut app = AppSettings::default();
        app.no_log = true;
        let joined = build_download_args(&spec, &tools, &app, None).join(" ");
        assert!(joined.contains("--no-log"));

        let mut app = AppSettings::default();
        app.log_level = crate::domain::config::LogLevel::Debug;
        let joined = build_download_args(&spec, &tools, &app, None).join(" ");
        assert!(joined.contains("--log-level DEBUG"));
    }

    #[test]
    fn parse_args_are_minimal() {
        let tools = ToolConfigs::default();
        let app = AppSettings::default();

        let args = build_parse_args("https://example.com/index.m3u8", &tools, &app);
        let expected: Vec<String> = vec![
            "https://example.com/index.m3u8",
            "--skip-download",
            "--auto-select",
            "--use-system-proxy",
            "--decryption-engine",
            "MP4DECRYPT",
        ]
        .into_iter()
        .map(String::from)
        .collect();
        assert_eq!(args, expected);
    }
}
