//! N_m3u8DL-RE 下载引擎
//!
//! 流媒体（HLS/DASH/MSS）下载引擎：参数构建（[`args`]）+ 输出解析（[`parser`]）。
//! 未知 URL 类型的兜底引擎（格式覆盖最广）。

pub mod args;
pub mod parser;

use std::sync::Arc;

use crate::domain::config::{AppSettings, ToolConfigs};
use crate::domain::download::{
    DownloadEngine, EngineEvent, EngineSession, StreamInfo, ToolId, UrlType,
};
use crate::domain::task::TaskSpec;
use parser::{OutputParser, RawEvent, StreamKind};

/// N_m3u8DL-RE 引擎
///
/// 无状态、全局共享；解析器正则预编译一次。
#[derive(Default)]
pub struct Nm3u8dlEngine {
    parser: Arc<OutputParser>,
}

impl Nm3u8dlEngine {
    pub fn new() -> Self {
        Self {
            parser: Arc::new(OutputParser::new()),
        }
    }
}

/// 解析 FFmpeg 可执行文件路径，并保证返回「存在 + 绝对」路径。
///
/// N_m3u8DL-RE 以**自身进程 CWD** 解析 `--ffmpeg-binary-path`，与 StreamGrab 的 CWD 不同，
/// 解析 ffmpeg 二进制路径为 [`ResolvedPath`]（非空+绝对+存在）。
///
/// 相对路径（如历史脏数据 `ffmpeg-master-.../bin/ffmpeg.exe`）会被绝对化；
/// 无法得到存在的绝对路径时返回 `None`，让 N_m3u8DL-RE 回退到 PATH 搜索。
fn resolve_ffmpeg_bin(tools: &ToolConfigs) -> Option<String> {
    let path = crate::infrastructure::tools::get_ffmpeg_exe_path(
        (!tools.ffmpeg.ffmpeg_path.is_empty()).then_some(tools.ffmpeg.ffmpeg_path.as_str()),
    )?;
    let abs = if path.is_absolute() {
        path
    } else {
        std::env::current_dir().ok()?.join(path)
    };
    crate::shared::ResolvedPath::try_from_path(abs).map(|rp| rp.to_string_lossy())
}

impl DownloadEngine for Nm3u8dlEngine {
    fn id(&self) -> ToolId {
        ToolId::Nm3u8dl
    }

    fn handles(&self, url_type: UrlType) -> bool {
        // 未知类型也由此引擎兜底（RE 格式覆盖最广，解析失败再报错）
        url_type.is_streaming() || url_type == UrlType::Unknown
    }

    fn build_download_args(
        &self,
        spec: &TaskSpec,
        tools: &ToolConfigs,
        app: &AppSettings,
    ) -> Vec<String> {
        // 混流/二进制合并需要 FFmpeg 二进制：解析为绝对路径后注入
        let ffmpeg_bin = resolve_ffmpeg_bin(tools);
        args::build_download_args(spec, tools, app, ffmpeg_bin.as_deref())
    }

    fn build_parse_args(&self, url: &str, tools: &ToolConfigs, app: &AppSettings) -> Vec<String> {
        // 解析阶段同样需要 FFmpeg（N_m3u8DL-RE 启动会校验/调用），与下载保持一致注入
        let ffmpeg_bin = resolve_ffmpeg_bin(tools);
        args::build_parse_args(url, tools, app, ffmpeg_bin.as_deref())
    }

    fn parse_streams(&self, stdout: &str) -> StreamInfo {
        self.parser.parse_streams(stdout)
    }

    fn new_session(&self) -> Box<dyn EngineSession> {
        Box::new(Nm3u8dlSession {
            parser: Arc::clone(&self.parser),
            video_total: 0,
            video_downloaded: 0,
            audio_total: 0,
            audio_downloaded: 0,
        })
    }
}

/// N_m3u8DL-RE 逐任务解析会话
///
/// 聚合视频/音频双流的下载进度，计算总体进度百分比，
/// 避免「视频下完、音频开始时进度跳回 0」。
struct Nm3u8dlSession {
    parser: Arc<OutputParser>,
    video_total: u32,
    video_downloaded: u32,
    audio_total: u32,
    audio_downloaded: u32,
}

impl EngineSession for Nm3u8dlSession {
    fn parse_line(&mut self, line: &str) -> Option<EngineEvent> {
        match self.parser.parse_line(line)? {
            RawEvent::Log { level, message } => Some(EngineEvent::Log { level, message }),
            RawEvent::Status { action } => Some(EngineEvent::Status { action }),
            RawEvent::Progress { kind, mut data } => {
                match kind {
                    StreamKind::Video => {
                        self.video_total = data.total_segments as u32;
                        self.video_downloaded = data.downloaded_segments as u32;
                    }
                    StreamKind::Audio => {
                        self.audio_total = data.total_segments as u32;
                        self.audio_downloaded = data.downloaded_segments as u32;
                    }
                }

                let total = self.video_total + self.audio_total;
                let downloaded = self.video_downloaded + self.audio_downloaded;
                data.overall_percent = if total > 0 {
                    (downloaded * 100 / total) as i32
                } else {
                    0
                };
                data.total_segments = total as i32;
                data.downloaded_segments = downloaded as i32;
                data.current_action = format!("下载中 {downloaded}/{total}");

                Some(EngineEvent::Progress { data })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_aggregates_video_and_audio_progress() {
        let engine = Nm3u8dlEngine::new();
        let mut session = engine.new_session();

        // 视频流进度
        let event = session
            .parse_line("Vid 1280x720 | 1159 Kbps ------------------------------ 30/60 50.00% 1.00MB/2.00MB 1.00MBps 00:00:10")
            .unwrap();
        match event {
            EngineEvent::Progress { data } => {
                assert_eq!(data.overall_percent, 50); // 30/60，暂无音频
                assert_eq!(data.total_segments, 60);
            }
            other => panic!("expected progress, got {other:?}"),
        }

        // 音频流进度加入后，总体进度 = (30+15)/(60+30) = 50%
        let event = session
            .parse_line("Aud Audio                ------------------------------ 15/30 50.00% -    -    --:--:--")
            .unwrap();
        match event {
            EngineEvent::Progress { data } => {
                assert_eq!(data.overall_percent, 50);
                assert_eq!(data.total_segments, 90);
                assert_eq!(data.downloaded_segments, 45);
            }
            other => panic!("expected progress, got {other:?}"),
        }

        // 视频完成、音频继续 → 进度不回退
        let event = session
            .parse_line("Vid 1280x720 | 1159 Kbps ------------------------------ 60/60 100.00% 2.00MB/2.00MB 1.00MBps 00:00:00")
            .unwrap();
        match event {
            EngineEvent::Progress { data } => {
                assert_eq!(data.overall_percent, 83); // (60+15)/90 ≈ 83
            }
            other => panic!("expected progress, got {other:?}"),
        }
    }

    #[test]
    fn sessions_are_independent() {
        let engine = Nm3u8dlEngine::new();
        let mut s1 = engine.new_session();
        let mut s2 = engine.new_session();

        let _ = s1.parse_line(
            "Vid 1280x720 | 1159 Kbps ------------------------------ 60/60 100.00% - - --:--:--",
        );
        let event = s2
            .parse_line(
                "Vid 1280x720 | 1159 Kbps ------------------------------ 1/60 1.67% - - --:--:--",
            )
            .unwrap();
        match event {
            EngineEvent::Progress { data } => assert_eq!(data.total_segments, 60),
            other => panic!("expected progress, got {other:?}"),
        }
    }

    #[test]
    fn engine_handles_streaming_and_unknown() {
        let engine = Nm3u8dlEngine::new();
        assert!(engine.handles(UrlType::Hls));
        assert!(engine.handles(UrlType::Dash));
        assert!(engine.handles(UrlType::Mss));
        assert!(engine.handles(UrlType::Unknown));
        assert!(!engine.handles(UrlType::HttpVideo));
        assert_eq!(engine.id(), ToolId::Nm3u8dl);
    }
}
