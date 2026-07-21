//! FFmpeg 直链下载命令行参数构建
//!
//! 直链视频走流拷贝（`-c copy`）下载，进度经 `-progress pipe:2` 输出到 stderr。

use std::path::PathBuf;

use crate::domain::config::FfmpegConfig;
use crate::domain::task::TaskSpec;

/// 构建直链下载命令参数
pub fn build_download_args(spec: &TaskSpec, cfg: &FfmpegConfig) -> Vec<String> {
    let mut args: Vec<String> = Vec::new();

    // === 网络选项（置于 -i 之前，作为输入选项）===
    if let Some(ua) = cfg.user_agent.as_deref().filter(|s| !s.is_empty()) {
        args.extend(["-user_agent".into(), ua.into()]);
    }
    if let Some(referer) = cfg.referer.as_deref().filter(|s| !s.is_empty()) {
        args.extend(["-headers".into(), format!("Referer: {referer}\r\n")]);
    }

    // === 断线重连 ===
    if cfg.reconnect_attempts > 0 {
        args.extend([
            "-reconnect".into(),
            "1".into(),
            "-reconnect_streamed".into(),
            "1".into(),
            "-reconnect_delay_max".into(),
            cfg.reconnect_delay.max(1).to_string(),
        ]);
    }

    // === 输入 ===
    args.extend(["-i".into(), spec.url.clone()]);

    // === 流拷贝（不重新编码）===
    args.extend(["-c".into(), "copy".into()]);

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

    #[test]
    fn default_args_copy_stream_with_progress() {
        let cfg = FfmpegConfig::default();
        let args = build_download_args(&spec(), &cfg);
        let expected_output = std::path::PathBuf::from("D:/Videos")
            .join("movie.mp4")
            .display()
            .to_string();
        let expected: Vec<String> = vec![
            "-reconnect".into(),
            "1".into(),
            "-reconnect_streamed".into(),
            "1".into(),
            "-reconnect_delay_max".into(),
            "5".into(),
            "-i".into(),
            "https://example.com/movie.mp4".into(),
            "-c".into(),
            "copy".into(),
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
        let ua_pos = args.iter().position(|a| a == "-user_agent").unwrap();
        let i_pos = args.iter().position(|a| a == "-i").unwrap();
        assert!(ua_pos < i_pos, "输入选项必须在 -i 之前");
        assert!(args.iter().any(|a| a == "-y"));
        assert!(!args.iter().any(|a| a == "-n"));
    }

    #[test]
    fn no_reconnect_when_disabled() {
        let mut cfg = FfmpegConfig::default();
        cfg.reconnect_attempts = 0;
        let args = build_download_args(&spec(), &cfg);
        assert!(!args.iter().any(|a| a == "-reconnect"));
    }
}
