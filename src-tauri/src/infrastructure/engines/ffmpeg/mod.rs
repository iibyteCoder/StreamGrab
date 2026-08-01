//! FFmpeg 下载引擎
//!
//! 直链视频下载引擎：流拷贝下载（[`args`]）+ `-progress` 输出解析（[`parser`]）。
//! 解析模式走 ffprobe（见 `infrastructure::media::ffprobe`）。

pub mod args;
pub mod parser;

use std::sync::Arc;

use crate::domain::config::{AppSettings, ToolConfigs};
use crate::domain::download::{
    DownloadEngine, EngineEvent, EngineSession, StreamInfo, ToolId, UrlType,
};
use crate::domain::task::{ProgressData, TaskSpec};
use parser::FfmpegOutputParser;

/// FFmpeg 引擎
#[derive(Default)]
pub struct FfmpegEngine {
    parser: Arc<FfmpegOutputParser>,
}

impl FfmpegEngine {
    pub fn new() -> Self {
        Self {
            parser: Arc::new(FfmpegOutputParser::new()),
        }
    }
}

impl DownloadEngine for FfmpegEngine {
    fn id(&self) -> ToolId {
        ToolId::Ffmpeg
    }

    fn handles(&self, url_type: UrlType) -> bool {
        url_type.needs_ffmpeg()
    }

    fn build_download_args(
        &self,
        spec: &TaskSpec,
        tools: &ToolConfigs,
        _app: &AppSettings,
    ) -> Vec<String> {
        args::build_download_args(spec, &tools.ffmpeg)
    }

    /// 直链解析参数：实际是 ffprobe 参数（由命令层用 ffprobe 二进制执行）
    fn build_parse_args(&self, url: &str, _tools: &ToolConfigs, _app: &AppSettings) -> Vec<String> {
        crate::infrastructure::media::ffprobe::probe_args(url)
    }

    /// 解析 ffprobe JSON 输出为流信息
    fn parse_streams(&self, stdout: &str) -> StreamInfo {
        crate::infrastructure::media::ffprobe::stream_info_from_json(stdout).unwrap_or_default()
    }

    fn new_session(&self) -> Box<dyn EngineSession> {
        Box::new(FfmpegSession {
            parser: Arc::clone(&self.parser),
            buffer: String::new(),
            total_duration_us: 0,
        })
    }
}

/// FFmpeg 逐任务解析会话
///
/// 累积 `-progress` 的 key=value 行，遇 `progress=` 行解析整块；
/// 从启动输出中提取 Duration 用于百分比计算。
struct FfmpegSession {
    parser: Arc<FfmpegOutputParser>,
    buffer: String,
    total_duration_us: i64,
}

impl EngineSession for FfmpegSession {
    fn parse_chunk(&mut self, chunk: &str) -> Vec<EngineEvent> {
        let line = chunk.trim();
        if line.is_empty() {
            return Vec::new();
        }

        // Duration 行（启动阶段输出）
        if line.contains("Duration:") {
            if let Some(duration) = self.parser.parse_duration(line) {
                self.total_duration_us = duration;
            }
            return Vec::new();
        }

        // 进度 key=value 行（排除普通信息行）
        let is_progress_line = line.contains('=')
            && !line.starts_with("Input")
            && !line.starts_with("Output")
            && !line.starts_with("Stream")
            && !line.starts_with("Metadata");

        if is_progress_line {
            self.buffer.push_str(line);
            self.buffer.push('\n');

            // 块结束标记
            if line.starts_with("progress=") {
                let block = std::mem::take(&mut self.buffer);
                if let Some((current_us, total_size, speed)) =
                    self.parser.parse_progress_block(&block)
                {
                    let percent = if self.total_duration_us > 0 {
                        ((current_us as f64 / self.total_duration_us as f64) * 100.0).min(100.0)
                            as i32
                    } else {
                        0
                    };
                    return vec![EngineEvent::Progress {
                        data: ProgressData {
                            percent,
                            overall_percent: percent,
                            speed: speed.unwrap_or(0),
                            downloaded_size: total_size.unwrap_or(0) as i64,
                            total_size: 0,
                            current_action: "下载中".into(),
                            ..Default::default()
                        },
                    }];
                }
            }
            return Vec::new();
        }

        // 其余作为日志
        vec![EngineEvent::Log {
            level: "info".into(),
            message: line.to_string(),
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_computes_percent_from_duration() {
        let engine = FfmpegEngine::new();
        let mut session = engine.new_session();

        // 启动输出中的 Duration（5 分钟）
        assert!(session
            .parse_chunk("  Duration: 00:05:00.00, start: 0.000000, bitrate: 1234 kb/s")
            .is_empty());

        // 进度块：已进行 2.5 分钟 → 50%
        for line in [
            "out_time_us=150000000",
            "total_size=5000000",
            "bitrate=500.0kbits/s",
            "speed=1.50x",
        ] {
            assert!(session.parse_chunk(line).is_empty());
        }
        let event = session
            .parse_chunk("progress=continue")
            .pop()
            .unwrap();
        match event {
            EngineEvent::Progress { data } => {
                assert_eq!(data.percent, 50);
                assert_eq!(data.downloaded_size, 5_000_000);
                assert_eq!(data.speed, 500_000);
            }
            other => panic!("expected progress, got {other:?}"),
        }
    }

    #[test]
    fn non_progress_lines_are_logs() {
        let engine = FfmpegEngine::new();
        let mut session = engine.new_session();
        let event = session
            .parse_chunk("Input #0, mov,mp4,m4a,3gp,3g2,mj2")
            .pop()
            .unwrap();
        // "Input" 开头不作为进度行 → 日志
        assert!(matches!(event, EngineEvent::Log { .. }));
    }

    #[test]
    fn engine_handles_only_http_video() {
        let engine = FfmpegEngine::new();
        assert!(engine.handles(UrlType::HttpVideo));
        assert!(!engine.handles(UrlType::Hls));
        assert!(!engine.handles(UrlType::Unknown));
        assert_eq!(engine.id(), ToolId::Ffmpeg);
    }
}
