//! FFmpeg 直链下载命令行参数构建
//!
//! 直链视频走流拷贝（`-c copy`）下载，进度经 `-progress pipe:2` 输出到 stderr。
//!
//! 字段映射均为真实 ffmpeg 参数（用 master 构建 `-h protocol=http` / 通用选项实测）：
//! - `retry_count` → `-reconnect_max_retries`（重试次数）
//! - `timeout` → `-rw_timeout`（IO 超时，µs）
//! - `connection_timeout` → `-timeout`（socket 超时，µs）
//! - `preserve_timestamps` → `-copyts`（保留输入时间戳）
//! - `http_proxy` / `cookies` / `max_redirects` / `auth`（basic）/ `reconnect_on_http_error` / `reconnect_delay_total_max`

use std::path::PathBuf;

use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine as _;

use crate::domain::config::FfmpegConfig;
use crate::domain::task::TaskSpec;

/// 秒 → 微秒（ffmpeg 的 `-rw_timeout`/`-timeout` 均为 µs 单位）
pub fn seconds_to_micros(seconds: u32) -> u64 {
    (seconds as u64) * 1_000_000
}

/// 构建 `Authorization: Basic <base64(user:pass)>` 的 base64 凭证串
pub fn basic_auth_credential(username: &str, password: &str) -> String {
    BASE64_STANDARD.encode(format!("{username}:{password}"))
}

/// 构建直链下载命令参数
pub fn build_download_args(spec: &TaskSpec, cfg: &FfmpegConfig) -> Vec<String> {
    let mut args: Vec<String> = Vec::new();

    // === 网络选项（置于 -i 之前，作为输入选项）===
    // 直链代理
    if let Some(proxy) = cfg.http_proxy.as_deref().filter(|s| !s.is_empty()) {
        args.extend(["-http_proxy".into(), proxy.into()]);
    }
    // User-Agent
    if let Some(ua) = cfg.user_agent.as_deref().filter(|s| !s.is_empty()) {
        args.extend(["-user_agent".into(), ua.into()]);
    }
    // Cookie
    if let Some(cookies) = cfg.cookies.as_deref().filter(|s| !s.is_empty()) {
        args.extend(["-cookies".into(), cookies.into()]);
    }
    // 自定义请求头（Referer + basic 认证合并进单个 -headers）
    let mut header_lines: Vec<String> = Vec::new();
    if let Some(referer) = cfg.referer.as_deref().filter(|s| !s.is_empty()) {
        header_lines.push(format!("Referer: {referer}"));
    }
    if !cfg.auth.username.is_empty() {
        args.extend(["-auth_type".into(), "basic".into()]);
        let cred = basic_auth_credential(&cfg.auth.username, &cfg.auth.password);
        header_lines.push(format!("Authorization: Basic {cred}"));
    }
    if !header_lines.is_empty() {
        args.extend([
            "-headers".into(),
            format!("{}\r\n", header_lines.join("\r\n")),
        ]);
    }
    // IO 超时 / 连接超时（秒 → µs）
    if cfg.timeout > 0 {
        args.extend([
            "-rw_timeout".into(),
            seconds_to_micros(cfg.timeout).to_string(),
        ]);
    }
    if cfg.connection_timeout > 0 {
        args.extend([
            "-timeout".into(),
            seconds_to_micros(cfg.connection_timeout).to_string(),
        ]);
    }
    // 最大重定向
    if cfg.max_redirects != 8 {
        args.extend(["-max_redirects".into(), cfg.max_redirects.to_string()]);
    }

    // === 断线重连 ===
    if cfg.reconnect_attempts > 0 {
        args.extend([
            "-reconnect".into(),
            "1".into(),
            "-reconnect_streamed".into(),
            "1".into(),
            "-reconnect_max_retries".into(),
            cfg.retry_count.to_string(),
            "-reconnect_delay_max".into(),
            cfg.reconnect_delay.max(1).to_string(),
        ]);
        if let Some(codes) = cfg
            .reconnect_on_http_error
            .as_deref()
            .filter(|s| !s.is_empty())
        {
            args.extend(["-reconnect_on_http_error".into(), codes.into()]);
        }
        if cfg.reconnect_delay_total_max != 256 {
            args.extend([
                "-reconnect_delay_total_max".into(),
                cfg.reconnect_delay_total_max.to_string(),
            ]);
        }
        if !cfg.respect_retry_after {
            args.extend(["-respect_retry_after".into(), "0".into()]);
        }
    }

    // === 输入 ===
    args.extend(["-i".into(), spec.url.clone()]);

    // === 流拷贝（不重新编码）===
    args.extend(["-c".into(), "copy".into()]);

    // === 保留输入时间戳 ===
    if cfg.preserve_timestamps {
        args.push("-copyts".into());
    }

    // === 进度输出到 stderr + 非交互 ===
    args.extend(["-progress".into(), "pipe:2".into(), "-nostdin".into()]);

    // === 覆盖行为 ===
    if cfg.overwrite_existing {
        args.push("-y".into());
    } else {
        args.push("-n".into());
    }

    // === 输出路径 ===
    let output = PathBuf::from(&spec.save_dir).join(&spec.file_name);
    args.push(output.display().to_string());

    args
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::config::{AuthConfig, FfmpegConfig};
    use crate::domain::download::UrlType;
    use crate::domain::task::TaskOverrides;

    fn spec() -> TaskSpec {
        TaskSpec {
            task_id: "t1".into(),
            url: "https://example.com/movie.mp4".into(),
            file_name: "movie.mp4".into(),
            save_dir: "D:/Videos".into(),
            overrides: TaskOverrides::default(),
            url_type: UrlType::HttpVideo,
        }
    }

    /// 全默认配置：所有字段默认值都产生真实参数（每个配置项都生效）
    #[test]
    fn default_config_emits_active_args() {
        let cfg = FfmpegConfig::default();
        let args = build_download_args(&spec(), &cfg);
        let expected_output = std::path::PathBuf::from("D:/Videos")
            .join("movie.mp4")
            .display()
            .to_string();
        let expected: Vec<String> = vec![
            "-rw_timeout".into(),
            "60000000".into(),
            "-timeout".into(),
            "30000000".into(),
            "-reconnect".into(),
            "1".into(),
            "-reconnect_streamed".into(),
            "1".into(),
            "-reconnect_max_retries".into(),
            "3".into(),
            "-reconnect_delay_max".into(),
            "5".into(),
            "-i".into(),
            "https://example.com/movie.mp4".into(),
            "-c".into(),
            "copy".into(),
            "-copyts".into(),
            "-progress".into(),
            "pipe:2".into(),
            "-nostdin".into(),
            "-n".into(),
            expected_output,
        ];
        assert_eq!(args, expected);
    }

    #[test]
    fn network_options_precede_input() {
        let mut cfg = FfmpegConfig::default();
        cfg.user_agent = Some("StreamGrab/1.0".into());
        cfg.referer = Some("https://example.com".into());
        cfg.overwrite_existing = true;

        let args = build_download_args(&spec(), &cfg);

        assert!(args.starts_with(&["-user_agent".into(), "StreamGrab/1.0".into()]));
        let i_pos = args.iter().position(|a| a == "-i").unwrap();
        assert!(
            args.iter().position(|a| a == "-user_agent").unwrap() < i_pos,
            "输入选项必须在 -i 之前"
        );
        assert!(args.iter().any(|a| a == "-y"));
        assert!(!args.iter().any(|a| a == "-n"));
        assert!(args
            .windows(2)
            .any(|w| w[0] == "-headers" && w[1].contains("Referer: https://example.com")));
    }

    #[test]
    fn no_reconnect_when_disabled() {
        let mut cfg = FfmpegConfig::default();
        cfg.reconnect_attempts = 0;
        let args = build_download_args(&spec(), &cfg);
        assert!(!args.iter().any(|a| a == "-reconnect"));
        assert!(!args.iter().any(|a| a == "-reconnect_max_retries"));
    }

    #[test]
    fn retry_count_maps_to_reconnect_max_retries() {
        let mut cfg = FfmpegConfig::default();
        cfg.retry_count = 10;
        let args = build_download_args(&spec(), &cfg);
        assert!(args
            .windows(2)
            .any(|w| w[0] == "-reconnect_max_retries" && w[1] == "10"));
    }

    #[test]
    fn timeouts_map_to_microseconds() {
        let mut cfg = FfmpegConfig::default();
        cfg.timeout = 100;
        cfg.connection_timeout = 20;
        let args = build_download_args(&spec(), &cfg);
        assert!(args
            .windows(2)
            .any(|w| w[0] == "-rw_timeout" && w[1] == "100000000"));
        assert!(args
            .windows(2)
            .any(|w| w[0] == "-timeout" && w[1] == "20000000"));
    }

    #[test]
    fn zero_timeout_omits_flag() {
        let mut cfg = FfmpegConfig::default();
        cfg.timeout = 0;
        cfg.connection_timeout = 0;
        let args = build_download_args(&spec(), &cfg);
        assert!(!args.iter().any(|a| a == "-rw_timeout"));
        assert!(!args.iter().any(|a| a == "-timeout"));
    }

    #[test]
    fn preserve_timestamps_toggles_copyts() {
        let cfg = FfmpegConfig::default(); // preserve_timestamps = true
        assert!(build_download_args(&spec(), &cfg).contains(&"-copyts".into()));

        let mut cfg = FfmpegConfig::default();
        cfg.preserve_timestamps = false;
        assert!(!build_download_args(&spec(), &cfg).contains(&"-copyts".into()));
    }

    #[test]
    fn proxy_cookies_and_redirects_emit() {
        let mut cfg = FfmpegConfig::default();
        cfg.http_proxy = Some("http://127.0.0.1:7890".into());
        cfg.cookies = Some("sid=abc; theme=dark".into());
        cfg.max_redirects = 3;
        let args = build_download_args(&spec(), &cfg);
        let joined = args.join(" ");
        assert!(joined.contains("-http_proxy http://127.0.0.1:7890"));
        assert!(joined.contains("-cookies sid=abc; theme=dark"));
        assert!(joined.contains("-max_redirects 3"));
    }

    #[test]
    fn basic_auth_emits_auth_type_and_authorization_header() {
        let mut cfg = FfmpegConfig::default();
        cfg.auth = AuthConfig {
            username: "user".into(),
            password: "pass".into(),
        };
        let args = build_download_args(&spec(), &cfg);
        let joined = args.join(" ");
        assert!(joined.contains("-auth_type basic"));
        assert!(joined.contains(&format!(
            "Authorization: Basic {}",
            basic_auth_credential("user", "pass")
        )));
    }

    #[test]
    fn auth_and_referer_merge_into_single_headers() {
        let mut cfg = FfmpegConfig::default();
        cfg.referer = Some("https://r.com".into());
        cfg.auth = AuthConfig {
            username: "u".into(),
            password: "p".into(),
        };
        let args = build_download_args(&spec(), &cfg);
        let headers = args
            .windows(2)
            .find(|w| w[0] == "-headers")
            .map(|w| w[1].clone())
            .unwrap();
        assert!(headers.contains("Referer: https://r.com"));
        assert!(headers.contains("Authorization: Basic"));
        // 单值内同时含两行，用 \r\n 分隔
        assert!(headers.contains("\r\n"));
    }

    #[test]
    fn reconnect_enhancements_emit_when_configured() {
        let mut cfg = FfmpegConfig::default();
        cfg.reconnect_on_http_error = Some("404,429".into());
        cfg.reconnect_delay_total_max = 60;
        cfg.respect_retry_after = false;
        let args = build_download_args(&spec(), &cfg);
        let joined = args.join(" ");
        assert!(joined.contains("-reconnect_on_http_error 404,429"));
        assert!(joined.contains("-reconnect_delay_total_max 60"));
        assert!(joined.contains("-respect_retry_after 0"));
    }

    #[test]
    fn reconnect_enhancements_omitted_when_reconnect_disabled() {
        let mut cfg = FfmpegConfig::default();
        cfg.reconnect_attempts = 0;
        cfg.reconnect_on_http_error = Some("404".into());
        let args = build_download_args(&spec(), &cfg);
        assert!(!args.iter().any(|a| a == "-reconnect_on_http_error"));
    }

    #[test]
    fn seconds_to_micros_converts() {
        assert_eq!(seconds_to_micros(1), 1_000_000);
        assert_eq!(seconds_to_micros(60), 60_000_000);
        assert_eq!(seconds_to_micros(0), 0);
    }

    #[test]
    fn basic_auth_credential_base64() {
        // base64("user:pass") = dXNlcjpwYXNz
        assert_eq!(basic_auth_credential("user", "pass"), "dXNlcjpwYXNz");
    }
}
