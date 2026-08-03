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
            buffer: String::new(),
            video_total: 0,
            video_downloaded: 0,
            audio_total: 0,
            audio_downloaded: 0,
        })
    }
}

/// N_m3u8DL-RE 逐任务解析会话
///
/// 缓冲进程输出，用流式扫描从粘连文本中提取进度块（N_m3u8DL-RE 非 TTY
/// 下多次进度更新零分隔粘连），并聚合视频/音频双流分片进度，
/// 避免「视频下完、音频开始时进度跳回 0」。
struct Nm3u8dlSession {
    parser: Arc<OutputParser>,
    /// 跨块输出缓冲（保留最后一个 `\n` 之后的未完成尾部）
    buffer: String,
    video_total: u32,
    video_downloaded: u32,
    audio_total: u32,
    audio_downloaded: u32,
}

impl Nm3u8dlSession {
    /// `RawEvent` → `EngineEvent`，同时聚合视频/音频双流总体进度
    fn map_event(&mut self, ev: RawEvent) -> EngineEvent {
        match ev {
            RawEvent::Log { level, message } => EngineEvent::Log { level, message },
            RawEvent::Status { action } => EngineEvent::Status { action },
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

                EngineEvent::Progress { data }
            }
        }
    }
}

impl EngineSession for Nm3u8dlSession {
    fn parse_chunk(&mut self, chunk: &str) -> Vec<EngineEvent> {
        self.buffer.push_str(chunk);
        // 仅处理到最后一个 `\n`：其前的内容已完整，其后的尾部可能仍在写入
        let split_at = match self.buffer.rfind('\n') {
            Some(i) => i + 1,
            None => return Vec::new(),
        };
        let complete: String = self.buffer.drain(..split_at).collect();
        if complete.is_empty() {
            return Vec::new();
        }

        self.parser
            .parse_stream(&complete)
            .into_iter()
            .map(|ev| self.map_event(ev))
            .collect()
    }

    fn finalize(&mut self) -> Vec<EngineEvent> {
        // N_m3u8DL-RE 非 TTY 下的退出倾泻是无换行粘连块：
        // parse_chunk 的按行排水永远等不到 `\n`，这里整体冲刷
        let rest = std::mem::take(&mut self.buffer);
        if rest.trim().is_empty() {
            return Vec::new();
        }
        self.parser
            .parse_stream(&rest)
            .into_iter()
            .map(|ev| self.map_event(ev))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 从事件流中取出最后一条 Progress 的数据
    fn last_progress(events: &[EngineEvent]) -> &crate::domain::task::ProgressData {
        events
            .iter()
            .rev()
            .find_map(|e| match e {
                EngineEvent::Progress { data } => Some(data),
                _ => None,
            })
            .expect("expected at least one progress event")
    }

    #[test]
    fn session_aggregates_video_and_audio_progress() {
        let engine = Nm3u8dlEngine::new();
        let mut session = engine.new_session();

        // 视频流进度
        let events = session.parse_chunk(
            "Vid 1280x720 | 1159 Kbps ------------------------------ 30/60 50.00% 1.00MB/2.00MB 1.00MBps 00:00:10\n",
        );
        let data = last_progress(&events);
        assert_eq!(data.overall_percent, 50); // 30/60，暂无音频
        assert_eq!(data.total_segments, 60);

        // 音频流进度加入后，总体进度 = (30+15)/(60+30) = 50%
        let events = session.parse_chunk(
            "Aud Audio                ------------------------------ 15/30 50.00% -    -    --:--:--\n",
        );
        let data = last_progress(&events);
        assert_eq!(data.overall_percent, 50);
        assert_eq!(data.total_segments, 90);
        assert_eq!(data.downloaded_segments, 45);

        // 视频完成、音频继续 → 进度不回退
        let events = session.parse_chunk(
            "Vid 1280x720 | 1159 Kbps ------------------------------ 60/60 100.00% 2.00MB/2.00MB 1.00MBps 00:00:00\n",
        );
        let data = last_progress(&events);
        assert_eq!(data.overall_percent, 83); // (60+15)/90 ≈ 83
    }

    #[test]
    fn sessions_are_independent() {
        let engine = Nm3u8dlEngine::new();
        let mut s1 = engine.new_session();
        let mut s2 = engine.new_session();

        let _ = s1.parse_chunk(
            "Vid 1280x720 | 1159 Kbps ------------------------------ 60/60 100.00% - - --:--:--\n",
        );
        let events = s2.parse_chunk(
            "Vid 1280x720 | 1159 Kbps ------------------------------ 1/60 1.67% - - --:--:--\n",
        );
        let data = last_progress(&events);
        assert_eq!(data.total_segments, 60);
    }

    #[test]
    fn session_parses_real_glued_progress_stream() {
        // 真实捕获：N_m3u8DL-RE 20260628 非 TTY 下进度块零分隔粘连
        let engine = Nm3u8dlEngine::new();
        let mut session = engine.new_session();
        let blob = "\
01:33:30.258 INFO : [0x1]: Video, h264 (avc1), 1280x720Vid 1280x720 | 1981 Kbps ------------------------------  1/5 20.00% -0.00Bps00:00:00Aud Audio                ------------------------------ 0/100 0.00% -    -    --:--:--Vid 1280x720 | 1981 Kbps ------------------------------  2/5 40.00% -0.00Bps00:00:00Aud Audio                ------------------------------ 0/100 0.00% -    -    --:--:--01:33:30.803 INFO : 二进制合并中...\n";
        let events = session.parse_chunk(blob);
        let progs: Vec<&EngineEvent> = events
            .iter()
            .filter(|e| matches!(e, EngineEvent::Progress { .. }))
            .collect();
        assert!(progs.len() >= 4, "got {} progress events", progs.len());
        // 应存在 Vid 2/5 = 40% 的进度（单流 percent 字段，聚合不改写 percent）
        let vid40 = events.iter().any(|e| match e {
            EngineEvent::Progress { data } => data.percent == 40,
            _ => false,
        });
        assert!(vid40, "应提取到 Vid 2/5 40% 进度");
    }

    #[test]
    fn finalize_drains_newline_less_exit_dump() {
        // 真实行为（实测 20260628 二进制）：N_m3u8DL-RE 非 TTY 下将全部
        // 进度帧积压到进程退出瞬间一次性输出，且经 NonAnsiWriter 剥离换行后
        // 是无 `\n` 的单个粘连块。parse_chunk 按行排水解析不到，finalize 必须兜住。
        let engine = Nm3u8dlEngine::new();
        let mut session = engine.new_session();

        let dump = "\
23:24:43.000 INFO : 开始下载...Vid 1264x528 | 2233 Kbps ------------------------------ 0/8 0.00% -0.00Bps --:--:--\
Vid 1264x528 | 2233 Kbps ------------------------------ 3/8 37.50% 2.26MB/18.09MB 1.37MBps 00:00:03\
Vid 1264x528 | 2233 Kbps ------------------------------ 8/8 100.00% 13.12MB - 00:00:00\
23:24:45.442 INFO : Done";

        // 无 `\n` → parse_chunk 全部缓冲，零事件
        assert!(session.parse_chunk(dump).is_empty());

        // finalize 冲刷：应得到进度事件且聚合到 100%，并保留日志/状态
        let events = session.finalize();
        let data = last_progress(&events);
        assert_eq!(data.overall_percent, 100);
        assert_eq!(data.downloaded_segments, 8);
        assert_eq!(data.total_segments, 8);
        assert!(
            events
                .iter()
                .any(|e| matches!(e, EngineEvent::Status { action } if action == "downloading")),
            "开始下载状态标记应被解析"
        );
        // 再次 finalize 应为空（缓冲已排空）
        assert!(session.finalize().is_empty());
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
