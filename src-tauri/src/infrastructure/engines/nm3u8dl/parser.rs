//! N_m3u8DL-RE 输出解析器
//!
//! 两类解析：
//! 1. **下载输出**（逐行）：日志行、进度行（Vid/Aud）、状态标记 → [`RawEvent`]
//! 2. **解析模式输出**（整段）：流列表 → [`StreamInfo`]（`parse_url` 使用）
//!
//! ## 输出格式
//!
//! ```text
//! 21:05:11.051 WARN : 你已开启下载完成后混流，自动开启二进制合并
//! 21:05:11.048 INFO : Vid 1280x720 | 1159 Kbps | mp4a.40.2 | 60 Segments | ~02m58s
//! Vid 1280x720 | 1159 Kbps ------------------------------ 1/61 1.64% 32.88KB/1.96MB 32.88KBps 00:00:12
//! ```

use crate::domain::download::{AudioStream, BaseStream, StreamInfo, SubtitleStream, VideoStream};
use crate::domain::media::parse_resolution;
use crate::domain::task::ProgressData;
use regex::Regex;

/// 进度行的流类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamKind {
    Video,
    Audio,
}

/// 解析器原始事件
///
/// 进度事件携带**单流**的分片信息；跨流总体进度聚合由会话层完成
#[derive(Debug, Clone, PartialEq)]
pub enum RawEvent {
    Log {
        level: String,
        message: String,
    },
    /// 阶段变化：downloading / merging / muxing / completed
    Status {
        action: String,
    },
    Progress {
        kind: StreamKind,
        data: ProgressData,
    },
}

/// N_m3u8DL-RE 输出解析器（正则全部预编译，可跨任务共享）
pub struct OutputParser {
    /// 日志行: `HH:MM:SS.mmm LEVEL : message`
    log_line: Regex,
    /// 进度行: `Vid/Aud ... ---- N/M percent size speed eta`
    progress_line: Regex,
    /// 备用百分比提取
    percent: Regex,
    /// 备用分片提取
    segments_pair: Regex,
    /// 开始下载标记
    start_download: Regex,
    /// 合并状态标记
    merging: Regex,
    /// 完成标记
    complete: Regex,
    /// 解析模式日志前缀（剥离后提取流信息行）
    stream_log_prefix: Regex,
    /// `NN Segments`
    segment_count: Regex,
    /// `~NNmNNs`
    duration_approx: Regex,
    /// 大小值 `32.88KB`
    size_value: Regex,
}

impl OutputParser {
    pub fn new() -> Self {
        Self {
            log_line: Regex::new(
                r"^(\d{2}:\d{2}:\d{2}\.\d+)\s+(INFO|WARN|ERROR|DEBUG)\s*:\s*(.+)$",
            )
            .unwrap(),
            progress_line: Regex::new(
                r"^(Vid|Aud)\s+(.+?)\s+-+\s+(\d+)/(\d+)\s+(\d+(?:\.\d+)?)%\s+(.+?)\s+([\d.]+(?:KB|MB|GB|B)ps|-)\s+(\d{2}:\d{2}:\d{2}|--:--:--)$"
            ).unwrap(),
            percent: Regex::new(r"(\d+(?:\.\d+)?)%").unwrap(),
            segments_pair: Regex::new(r"(\d+)/(\d+)").unwrap(),
            start_download: Regex::new(r"^开始下载").unwrap(),
            merging: Regex::new(r"(二进制合并中|正在合并|Merging\.\.\.)").unwrap(),
            complete: Regex::new(r"^All done$").unwrap(),
            stream_log_prefix: Regex::new(
                r"^\d{2}:\d{2}:\d{2}\.\d+\s+(?:INFO|WARN|ERROR|DEBUG)\s*:\s*(.+)$",
            )
            .unwrap(),
            segment_count: Regex::new(r"(\d+)\s*[Ss]egment").unwrap(),
            duration_approx: Regex::new(r"~(\d+)m(\d+)s").unwrap(),
            size_value: Regex::new(r"([\d.]+)(KB|MB|GB|B)").unwrap(),
        }
    }

    /// 解析单行下载输出
    pub fn parse_line(&self, line: &str) -> Option<RawEvent> {
        let line = line.trim();
        if line.is_empty() {
            return None;
        }

        // 1. 日志格式行（有时间戳前缀）
        if let Some(caps) = self.log_line.captures(line) {
            let level = &caps[2];
            let message = &caps[3];
            return self.parse_log_message(level, message);
        }

        // 2. 进度行（Vid/Aud 开头）
        if line.starts_with("Vid ") || line.starts_with("Aud ") {
            return self.parse_progress_line(line);
        }

        // 3. 其余作为普通日志
        Some(RawEvent::Log {
            level: "info".into(),
            message: line.to_string(),
        })
    }

    /// 解析日志消息（检查状态标记）
    fn parse_log_message(&self, level: &str, message: &str) -> Option<RawEvent> {
        if self.complete.is_match(message) {
            return Some(RawEvent::Status {
                action: "completed".into(),
            });
        }
        if self.merging.is_match(message) {
            return Some(RawEvent::Status {
                action: "merging".into(),
            });
        }
        if self.start_download.is_match(message) {
            return Some(RawEvent::Status {
                action: "downloading".into(),
            });
        }

        let level = match level {
            "ERROR" => "error",
            "WARN" => "warn",
            "DEBUG" => "debug",
            _ => "info",
        };
        Some(RawEvent::Log {
            level: level.into(),
            message: message.to_string(),
        })
    }

    /// 解析进度行
    fn parse_progress_line(&self, line: &str) -> Option<RawEvent> {
        if let Some(caps) = self.progress_line.captures(line) {
            let kind = if &caps[1] == "Vid" {
                StreamKind::Video
            } else {
                StreamKind::Audio
            };
            let downloaded: i32 = caps[3].parse().unwrap_or(0);
            let total: i32 = caps[4].parse().unwrap_or(0);
            let (downloaded_size, total_size) = self.parse_size_info(&caps[6]);
            let speed = self.parse_speed(&caps[7]);
            let eta = self.parse_eta(&caps[8]);
            let percent = if total > 0 {
                (downloaded as f64 / total as f64 * 100.0).round() as i32
            } else {
                caps[5].parse::<f64>().unwrap_or(0.0).round() as i32
            };

            return Some(RawEvent::Progress {
                kind,
                data: ProgressData {
                    percent,
                    overall_percent: 0, // 由会话层聚合
                    speed: speed as i64,
                    downloaded_size: downloaded_size as i64,
                    total_size: total_size as i64,
                    downloaded_segments: downloaded,
                    total_segments: total,
                    eta: eta as i32,
                    current_action: format!("下载中 {downloaded}/{total}"),
                },
            });
        }

        self.parse_simple_progress(line)
    }

    /// 简化进度解析（正则不匹配时的兜底）
    fn parse_simple_progress(&self, line: &str) -> Option<RawEvent> {
        let percent = self
            .percent
            .captures(line)?
            .get(1)?
            .as_str()
            .parse::<f64>()
            .ok()?;

        let (downloaded, total) = match self.segments_pair.captures(line) {
            Some(caps) => (
                caps.get(1)?.as_str().parse::<i32>().ok()?,
                caps.get(2)?.as_str().parse::<i32>().ok()?,
            ),
            None => (0, 0),
        };

        Some(RawEvent::Progress {
            kind: StreamKind::Video,
            data: ProgressData {
                percent: percent.round() as i32,
                downloaded_segments: downloaded,
                total_segments: total,
                ..Default::default()
            },
        })
    }

    /// 解析大小信息 `32.88KB/1.96MB` 或 `-`
    fn parse_size_info(&self, size_info: &str) -> (u64, u64) {
        if size_info == "-" {
            return (0, 0);
        }
        let mut parts = size_info.split('/');
        match (parts.next(), parts.next()) {
            (Some(d), Some(t)) => (self.parse_size(d), self.parse_size(t)),
            _ => (0, 0),
        }
    }

    /// 解析单个大小值 `32.88KB`
    fn parse_size(&self, size_str: &str) -> u64 {
        if let Some(caps) = self.size_value.captures(size_str) {
            let num: f64 = caps.get(1).unwrap().as_str().parse().unwrap_or(0.0);
            let unit = caps.get(2).unwrap().as_str();
            return match unit {
                "GB" => (num * 1024.0 * 1024.0 * 1024.0) as u64,
                "MB" => (num * 1024.0 * 1024.0) as u64,
                "KB" => (num * 1024.0) as u64,
                _ => num as u64,
            };
        }
        0
    }

    /// 解析速度 `32.88KBps` 或 `-`
    fn parse_speed(&self, speed_str: &str) -> u64 {
        if speed_str == "-" {
            return 0;
        }
        self.parse_size(speed_str.trim_end_matches("ps"))
    }

    /// 解析 ETA `00:00:12` 或 `--:--:--`
    fn parse_eta(&self, eta_str: &str) -> u32 {
        if eta_str == "--:--:--" {
            return 0;
        }
        let parts: Vec<&str> = eta_str.split(':').collect();
        if parts.len() == 3 {
            if let (Ok(h), Ok(m), Ok(s)) = (
                parts[0].parse::<u32>(),
                parts[1].parse::<u32>(),
                parts[2].parse::<u32>(),
            ) {
                return h * 3600 + m * 60 + s;
            }
        }
        0
    }

    // ========================================
    // 解析模式：流列表解析
    // ========================================

    /// 从解析模式 stdout 提取流信息
    ///
    /// ```text
    /// Vid 960x544 | 785 Kbps | mp4a.40.2 | 56 Segments | ~02m49s
    /// Aud audio-32000 | Audio | 57 Segments | ~02m49s
    /// ```
    ///
    /// 同时支持带日志前缀（`21:29:59.639 INFO : Vid ...`）的行
    pub fn parse_streams(&self, stdout: &str) -> StreamInfo {
        let mut videos: Vec<VideoStream> = vec![];
        let mut audios: Vec<AudioStream> = vec![];
        let mut subtitles: Vec<SubtitleStream> = vec![];
        let mut max_segments: u32 = 0;
        let mut duration: f64 = 0.0;

        for line in stdout.lines() {
            let line = line.trim();
            // 剥离日志前缀
            let message = self
                .stream_log_prefix
                .captures(line)
                .and_then(|caps| caps.get(1))
                .map(|m| m.as_str())
                .unwrap_or(line);

            if message.starts_with("Vid ") || message.contains("| Vid ") {
                if let Some(video) = Self::parse_video_line(message) {
                    videos.push(video);
                }
            }
            if message.starts_with("Aud ") || message.contains("| Aud ") {
                if let Some(audio) = Self::parse_audio_line(message) {
                    audios.push(audio);
                }
            }
            if message.starts_with("Sub ") || message.contains("| Sub ") {
                if let Some(subtitle) = Self::parse_subtitle_line(message) {
                    subtitles.push(subtitle);
                }
            }
            if let Some(segs) = self.parse_segments_from_line(message) {
                max_segments = max_segments.max(segs);
            }
            if let Some(dur) = self.parse_duration_from_line(message) {
                duration = duration.max(dur);
            }
        }

        // 按带宽降序排序
        videos.sort_by(|a, b| b.base.bandwidth.cmp(&a.base.bandwidth));
        audios.sort_by(|a, b| b.base.bandwidth.cmp(&a.base.bandwidth));

        StreamInfo {
            videos,
            audios,
            subtitles,
            duration,
            segment_count: max_segments,
            is_live: false,
            is_encrypted: false,
        }
    }

    /// 解析视频流信息行: `Vid 960x544 | 785 Kbps | mp4a.40.2 | 56 Segments | ~02m49s`
    fn parse_video_line(line: &str) -> Option<VideoStream> {
        let vid_part = if let Some(stripped) = line.strip_prefix("Vid ") {
            stripped
        } else if let Some(pos) = line.find("| Vid ") {
            &line[pos + 6..]
        } else {
            return None;
        };

        let parts: Vec<&str> = vid_part.split('|').map(|s| s.trim()).collect();
        if parts.is_empty() {
            return None;
        }

        let resolution = parts.first().unwrap_or(&"").to_string();
        let (width, height) = parse_resolution(&resolution);
        // 码率: `785 Kbps` → bps
        let bandwidth = parts
            .get(1)
            .and_then(|s| s.split_whitespace().next())
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0)
            * 1000;
        let codecs = parts.get(2).unwrap_or(&"").to_string();

        Some(VideoStream {
            base: BaseStream {
                id: resolution.clone(),
                bandwidth,
                codecs,
                language: String::new(),
                name: resolution.clone(),
                group_id: None,
                selected: None,
            },
            resolution,
            width,
            height,
            frame_rate: 0.0,
            video_range: "SDR".to_string(),
        })
    }

    /// 解析音频流信息行: `Aud audio-32000 | Audio | 57 Segments | ~02m49s`
    fn parse_audio_line(line: &str) -> Option<AudioStream> {
        let aud_part = if let Some(stripped) = line.strip_prefix("Aud ") {
            stripped
        } else if let Some(pos) = line.find("| Aud ") {
            &line[pos + 6..]
        } else {
            return None;
        };

        let parts: Vec<&str> = aud_part.split('|').map(|s| s.trim()).collect();
        if parts.is_empty() {
            return None;
        }

        let id = parts.first().unwrap_or(&"").to_string();
        // 从 ID 提取码率: audio-32000 → 32000
        let bandwidth = id
            .split('-')
            .next_back()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);

        Some(AudioStream {
            base: BaseStream {
                id: id.clone(),
                bandwidth,
                codecs: String::new(),
                language: String::new(),
                name: id,
                group_id: None,
                selected: None,
            },
            channels: "2".to_string(),
            sample_rate: 0,
            is_default: false,
        })
    }

    /// 解析字幕流信息行
    fn parse_subtitle_line(line: &str) -> Option<SubtitleStream> {
        let sub_part = if let Some(stripped) = line.strip_prefix("Sub ") {
            stripped
        } else if let Some(pos) = line.find("| Sub ") {
            &line[pos + 6..]
        } else {
            return None;
        };

        let parts: Vec<&str> = sub_part.split('|').map(|s| s.trim()).collect();
        let id = parts.first().unwrap_or(&"").to_string();

        Some(SubtitleStream {
            base: BaseStream {
                id: id.clone(),
                bandwidth: 0,
                codecs: String::new(),
                language: String::new(),
                name: id,
                group_id: None,
                selected: None,
            },
            format: "srt".to_string(),
            is_default: false,
            is_forced: false,
        })
    }

    /// 提取行内分片数（`56 Segments`）
    fn parse_segments_from_line(&self, line: &str) -> Option<u32> {
        self.segment_count
            .captures(line)
            .and_then(|cap| cap.get(1))
            .and_then(|m| m.as_str().parse().ok())
    }

    /// 提取行内时长（`~02m49s`）
    fn parse_duration_from_line(&self, line: &str) -> Option<f64> {
        let cap = self.duration_approx.captures(line)?;
        let minutes: f64 = cap.get(1)?.as_str().parse().ok()?;
        let seconds: f64 = cap.get(2)?.as_str().parse().ok()?;
        Some(minutes * 60.0 + seconds)
    }
}

impl Default for OutputParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn progress(event: RawEvent) -> ProgressData {
        match event {
            RawEvent::Progress { data, .. } => data,
            other => panic!("expected progress, got {other:?}"),
        }
    }

    #[test]
    fn parses_log_line() {
        let parser = OutputParser::new();
        let event = parser
            .parse_line("21:05:11.051 WARN : 你已开启下载完成后混流，自动开启二进制合并")
            .unwrap();
        match event {
            RawEvent::Log { level, .. } => assert_eq!(level, "warn"),
            other => panic!("expected log, got {other:?}"),
        }
    }

    #[test]
    fn parses_start_download_status() {
        let parser = OutputParser::new();
        let event = parser
            .parse_line("21:05:11.052 INFO : 开始下载...Vid 1280x720 | 1159 Kbps | mp4a.40.2")
            .unwrap();
        assert_eq!(
            event,
            RawEvent::Status {
                action: "downloading".into()
            }
        );
    }

    #[test]
    fn parses_merging_status() {
        let parser = OutputParser::new();
        let event = parser
            .parse_line("21:05:13.341 INFO : 二进制合并中...")
            .unwrap();
        assert_eq!(
            event,
            RawEvent::Status {
                action: "merging".into()
            }
        );
    }

    #[test]
    fn parses_completed_status() {
        let parser = OutputParser::new();
        let event = parser.parse_line("21:05:15.123 INFO : All done").unwrap();
        assert_eq!(
            event,
            RawEvent::Status {
                action: "completed".into()
            }
        );
    }

    #[test]
    fn no_false_muxing_trigger() {
        let parser = OutputParser::new();
        // "下载完成后混流" 不应触发状态变化
        let event = parser
            .parse_line("21:05:11.051 WARN : 你已开启下载完成后混流，自动开启二进制合并")
            .unwrap();
        assert!(matches!(event, RawEvent::Log { .. }));
    }

    #[test]
    fn parses_video_progress_line() {
        let parser = OutputParser::new();
        let event = parser.parse_line(
            "Vid 1280x720 | 1159 Kbps ------------------------------ 1/61 1.64% 32.88KB/1.96MB 32.88KBps 00:00:12",
        ).unwrap();
        match event {
            RawEvent::Progress { kind, data } => {
                assert_eq!(kind, StreamKind::Video);
                assert_eq!(data.downloaded_segments, 1);
                assert_eq!(data.total_segments, 61);
                assert_eq!(data.percent, 2);
                assert_eq!(data.downloaded_size, (32.88 * 1024.0) as i64);
                assert_eq!(data.total_size, (1.96 * 1024.0 * 1024.0) as i64);
                assert_eq!(data.speed, (32.88 * 1024.0) as i64);
                assert_eq!(data.eta, 12);
            }
            other => panic!("expected progress, got {other:?}"),
        }
    }

    #[test]
    fn parses_audio_progress_line_with_placeholders() {
        let parser = OutputParser::new();
        let event = parser.parse_line(
            "Aud Audio                ------------------------------ 0/100 0.00% -    -    --:--:--",
        ).unwrap();
        match event {
            RawEvent::Progress { kind, data } => {
                assert_eq!(kind, StreamKind::Audio);
                assert_eq!(data.total_segments, 100);
                assert_eq!(data.speed, 0);
                assert_eq!(data.eta, 0);
            }
            other => panic!("expected progress, got {other:?}"),
        }
    }

    #[test]
    fn fallback_simple_progress() {
        let parser = OutputParser::new();
        // Vid 开头但不符合完整进度行格式 → 兜底解析百分比与分片
        let data = progress(parser.parse_line("Vid broken format 5/20 25.0%").unwrap());
        assert_eq!(data.percent, 25);
        assert_eq!(data.downloaded_segments, 5);
        assert_eq!(data.total_segments, 20);
    }

    #[test]
    fn plain_line_is_log() {
        let parser = OutputParser::new();
        let event = parser.parse_line("random output without pattern").unwrap();
        assert_eq!(
            event,
            RawEvent::Log {
                level: "info".into(),
                message: "random output without pattern".into()
            }
        );
        assert!(parser.parse_line("   ").is_none());
    }

    #[test]
    fn parses_streams_from_parse_mode_output() {
        let parser = OutputParser::new();
        let stdout = "\
21:29:59.639 INFO : Vid 1920x1080 | 4500 Kbps | avc1.640028 | 120 Segments | ~05m00s
21:29:59.640 INFO : Vid 1280x720 | 2000 Kbps | avc1.64001f | 120 Segments | ~05m00s
21:29:59.641 INFO : Aud audio-128000 | Audio | 121 Segments | ~05m00s
21:29:59.642 INFO : Sub subs-0 | Text | 10 Segments | ~05m00s";

        let info = parser.parse_streams(stdout);
        assert_eq!(info.videos.len(), 2);
        // 按带宽降序
        assert_eq!(info.videos[0].resolution, "1920x1080");
        assert_eq!(info.videos[0].base.bandwidth, 4_500_000);
        assert_eq!(info.videos[0].base.codecs, "avc1.640028");
        assert_eq!(info.videos[0].width, 1920);
        assert_eq!(info.videos[1].resolution, "1280x720");
        assert_eq!(info.audios.len(), 1);
        assert_eq!(info.audios[0].base.bandwidth, 128000);
        assert_eq!(info.subtitles.len(), 1);
        assert_eq!(info.segment_count, 121);
        assert_eq!(info.duration, 300.0);
        assert!(!info.is_live);
    }

    #[test]
    fn parses_streams_without_log_prefix() {
        let parser = OutputParser::new();
        let info =
            parser.parse_streams("Vid 960x544 | 785 Kbps | mp4a.40.2 | 56 Segments | ~02m49s");
        assert_eq!(info.videos.len(), 1);
        assert_eq!(info.videos[0].resolution, "960x544");
        assert_eq!(info.segment_count, 56);
        assert_eq!(info.duration, 169.0);
    }

    #[test]
    fn empty_output_gives_empty_info() {
        let parser = OutputParser::new();
        let info = parser.parse_streams("no streams here\njust noise");
        assert_eq!(info, StreamInfo::default());
    }
}
