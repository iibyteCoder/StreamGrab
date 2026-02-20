//! N_m3u8DL-RE 输出解析器
//!
//! 解析 N_m3u8DL-RE 的输出并转换为结构化事件
//!
//! ## 输出格式分析
//!
//! ### 日志消息格式（有时间戳前缀）
//! ```text
//! 21:05:11.051 WARN : 你已开启下载完成后混流，自动开启二进制合并
//! 21:05:11.052 INFO : 开始下载...Vid 1280x720 | 1159 Kbps | mp4a.40.2
//! 21:05:11.048 INFO : Vid 1280x720 | 1159 Kbps | mp4a.40.2 | 60 Segments | ~02m58s
//! ```
//!
//! ### 进度行格式（以 Vid/Aud 开头，无日志前缀）
//! ```text
//! Vid 1280x720 | 1159 Kbps ------------------------------ 0/61 0.00% - 0.00Bps --:--:--
//! Aud Audio                ------------------------------ 0/100 0.00% -    -    --:--:--
//! Vid 1280x720 | 1159 Kbps ------------------------------ 1/61 1.64% 32.88KB/1.96MB 32.88KBps 00:00:12
//! ```

use regex::Regex;
use serde_json::Value;

/// 解析后的事件
#[derive(Debug, Clone)]
pub struct ParsedEvent {
    /// 事件类型: progress, status, log
    pub event_type: String,
    /// 事件数据
    pub data: Value,
}

/// 输出解析器
pub struct OutputParser {
    /// 日志行格式: `HH:MM:SS.mmm LEVEL : message`
    log_line_regex: Regex,
    /// 进度行格式: `Vid/Aud ... --- N/M percent ...`
    progress_line_regex: Regex,
    /// 开始下载标记
    start_download_regex: Regex,
    /// 合并状态标记
    merging_regex: Regex,
    /// 完成标记
    complete_regex: Regex,
}

impl OutputParser {
    /// 创建新的解析器
    pub fn new() -> Self {
        Self {
            // 匹配日志格式: `21:05:11.051 INFO : message` 或 `21:05:11.051 WARN : message`
            log_line_regex: Regex::new(r"^(\d{2}:\d{2}:\d{2}\.\d+)\s+(INFO|WARN|ERROR|DEBUG)\s*:\s*(.+)$").unwrap(),

            // 匹配进度行: `Vid 1280x720 | 1159 Kbps --- 0/61 0.00% 32.88KB/1.96MB 32.88KBps 00:00:12`
            // 或: `Aud Audio --- 0/100 0.00% - - --:--:--`
            progress_line_regex: Regex::new(
                r"^(Vid|Aud)\s+(.+?)\s+-+\s+(\d+)/(\d+)\s+(\d+(?:\.\d+)?)%\s+(.+?)\s+([\d.]+(?:KB|MB|GB|B)ps|-)\s+(\d{2}:\d{2}:\d{2}|--:--:--)$"
            ).unwrap(),

            // 开始下载标记
            start_download_regex: Regex::new(r"^开始下载").unwrap(),

            // 合并状态标记 - 精确匹配 "二进制合并中" 或 "正在合并"
            merging_regex: Regex::new(r"(二进制合并中|正在合并|Merging\.\.\.)").unwrap(),

            // 完成标记 - N_m3u8DL-RE 标准完成消息
            complete_regex: Regex::new(r"^All done$").unwrap(),
        }
    }

    /// 解析输出行
    pub fn parse(&self, line: &str) -> Option<ParsedEvent> {
        let line = line.trim();

        if line.is_empty() {
            return None;
        }

        // 1. 首先检查是否是日志格式行（有时间戳前缀）
        if let Some(caps) = self.log_line_regex.captures(line) {
            let _timestamp = &caps[1];
            let level = &caps[2];
            let message = &caps[3];

            // 检查日志消息中的关键状态
            return self.parse_log_message(level, message);
        }

        // 2. 检查是否是进度行（Vid 或 Aud）
        if line.starts_with("Vid ") || line.starts_with("Aud ") {
            return self.parse_progress_line(line);
        }

        // 3. 其他情况作为普通日志处理
        Some(ParsedEvent {
            event_type: "log".to_string(),
            data: serde_json::json!({
                "level": "info",
                "message": line
            }),
        })
    }

    /// 解析日志消息（已去掉时间戳前缀）
    fn parse_log_message(&self, level: &str, message: &str) -> Option<ParsedEvent> {
        // 检查完成状态
        if self.complete_regex.is_match(message) {
            return Some(ParsedEvent {
                event_type: "status".to_string(),
                data: serde_json::json!({
                    "status": "completed",
                    "message": message
                }),
            });
        }

        // 检查合并状态 - 精确匹配
        if self.merging_regex.is_match(message) {
            return Some(ParsedEvent {
                event_type: "status".to_string(),
                data: serde_json::json!({
                    "status": "muxing",
                    "message": message
                }),
            });
        }

        // 检查开始下载
        if self.start_download_regex.is_match(message) {
            return Some(ParsedEvent {
                event_type: "status".to_string(),
                data: serde_json::json!({
                    "status": "downloading",
                    "message": message
                }),
            });
        }

        // 普通日志消息
        let log_level = match level {
            "ERROR" => "error",
            "WARN" => "warn",
            "DEBUG" => "debug",
            _ => "info",
        };

        Some(ParsedEvent {
            event_type: "log".to_string(),
            data: serde_json::json!({
                "level": log_level,
                "message": message
            }),
        })
    }

    /// 解析进度行
    /// 格式: `Vid 1280x720 | 1159 Kbps ------------------------------ 0/61 0.00% - 0.00Bps --:--:--`
    /// 或: `Vid 1280x720 | 1159 Kbps ------------------------------ 1/61 1.64% 32.88KB/1.96MB 32.88KBps 00:00:12`
    fn parse_progress_line(&self, line: &str) -> Option<ParsedEvent> {
        if let Some(caps) = self.progress_line_regex.captures(line) {
            let stream_type = &caps[1]; // Vid 或 Aud
            let _stream_info = &caps[2]; // 1280x720 | 1159 Kbps
            let downloaded: u32 = caps[3].parse().unwrap_or(0);
            let total: u32 = caps[4].parse().unwrap_or(0);
            let percent: f64 = caps[5].parse().unwrap_or(0.0);
            let size_info = &caps[6]; // 32.88KB/1.96MB 或 -
            let speed_str = &caps[7]; // 32.88KBps 或 -
            let eta_str = &caps[8]; // 00:00:12 或 --:--:--

            // 解析文件大小
            let (downloaded_size, total_size) = self.parse_size_info(size_info);

            // 解析速度
            let speed = self.parse_speed(speed_str);

            // 解析 ETA
            let eta = self.parse_eta(eta_str);

            // 计算总进度百分比（基于分片）
            let overall_percent = if total > 0 {
                (downloaded as f64 / total as f64 * 100.0).round()
            } else {
                percent.round()
            };

            return Some(ParsedEvent {
                event_type: "progress".to_string(),
                data: serde_json::json!({
                    "streamType": stream_type,
                    "percent": overall_percent,
                    "downloadedSegments": downloaded,
                    "totalSegments": total,
                    "downloadedSize": downloaded_size,
                    "totalSize": total_size,
                    "speed": speed,
                    "speedStr": if speed_str != "-" { speed_str } else { "" },
                    "eta": eta,
                    "currentAction": format!("下载中 {}/{}", downloaded, total)
                }),
            });
        }

        // 如果正则不匹配，尝试简化解析
        self.parse_simple_progress(line)
    }

    /// 简化的进度解析（备用）
    fn parse_simple_progress(&self, line: &str) -> Option<ParsedEvent> {
        // 提取百分比
        let percent_regex = Regex::new(r"(\d+(?:\.\d+)?)%").ok()?;
        let percent = percent_regex
            .captures(line)?
            .get(1)?
            .as_str()
            .parse::<f64>()
            .ok()?;

        // 提取分片进度
        let segments_regex = Regex::new(r"(\d+)/(\d+)").ok()?;
        let (downloaded, total) = if let Some(caps) = segments_regex.captures(line) {
            (
                caps.get(1)?.as_str().parse::<u32>().ok()?,
                caps.get(2)?.as_str().parse::<u32>().ok()?,
            )
        } else {
            (0, 0)
        };

        Some(ParsedEvent {
            event_type: "progress".to_string(),
            data: serde_json::json!({
                "percent": percent,
                "downloadedSegments": downloaded,
                "totalSegments": total,
                "downloadedSize": 0,
                "totalSize": 0,
                "speed": 0,
                "eta": 0
            }),
        })
    }

    /// 解析大小信息 `32.88KB/1.96MB` 或 `-`
    fn parse_size_info(&self, size_info: &str) -> (u64, u64) {
        if size_info == "-" {
            return (0, 0);
        }

        let parts: Vec<&str> = size_info.split('/').collect();
        if parts.len() != 2 {
            return (0, 0);
        }

        let downloaded = self.parse_size(parts[0]);
        let total = self.parse_size(parts[1]);
        (downloaded, total)
    }

    /// 解析单个大小值 `32.88KB`
    fn parse_size(&self, size_str: &str) -> u64 {
        let size_regex = Regex::new(r"([\d.]+)(KB|MB|GB|B)").unwrap();

        if let Some(caps) = size_regex.captures(size_str) {
            let num: f64 = caps.get(1).unwrap().as_str().parse().unwrap_or(0.0);
            let unit = caps.get(2).unwrap().as_str();

            return match unit {
                "GB" => (num * 1024.0 * 1024.0 * 1024.0) as u64,
                "MB" => (num * 1024.0 * 1024.0) as u64,
                "KB" => (num * 1024.0) as u64,
                "B" => num as u64,
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
}

impl Default for OutputParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_log_line() {
        let parser = OutputParser::new();

        let result = parser.parse("21:05:11.051 WARN : 你已开启下载完成后混流，自动开启二进制合并");
        assert!(result.is_some());
        let event = result.unwrap();
        assert_eq!(event.event_type, "log");
    }

    #[test]
    fn test_parse_start_download() {
        let parser = OutputParser::new();

        let result =
            parser.parse("21:05:11.052 INFO : 开始下载...Vid 1280x720 | 1159 Kbps | mp4a.40.2");
        assert!(result.is_some());
        let event = result.unwrap();
        assert_eq!(event.event_type, "status");
        assert_eq!(event.data["status"], "downloading");
    }

    #[test]
    fn test_parse_merging() {
        let parser = OutputParser::new();

        let result = parser.parse("21:05:13.341 INFO : 二进制合并中...");
        assert!(result.is_some());
        let event = result.unwrap();
        assert_eq!(event.event_type, "status");
        assert_eq!(event.data["status"], "muxing");
    }

    #[test]
    fn test_parse_progress_line() {
        let parser = OutputParser::new();

        let result = parser.parse("Vid 1280x720 | 1159 Kbps ------------------------------ 1/61 1.64% 32.88KB/1.96MB 32.88KBps 00:00:12");
        assert!(result.is_some());
        let event = result.unwrap();
        assert_eq!(event.event_type, "progress");
        assert_eq!(event.data["downloadedSegments"], 1);
        assert_eq!(event.data["totalSegments"], 61);
    }

    #[test]
    fn test_parse_audio_progress() {
        let parser = OutputParser::new();

        let result = parser.parse("Aud Audio                ------------------------------ 0/100 0.00% -    -    --:--:--");
        assert!(result.is_some());
        let event = result.unwrap();
        assert_eq!(event.event_type, "progress");
        assert_eq!(event.data["streamType"], "Aud");
    }

    #[test]
    fn test_no_false_muxing_trigger() {
        let parser = OutputParser::new();

        // "下载完成后混流" 不应该触发 muxing 状态
        let result = parser.parse("21:05:11.051 WARN : 你已开启下载完成后混流，自动开启二进制合并");
        assert!(result.is_some());
        let event = result.unwrap();
        // 应该是普通日志，不是 muxing 状态
        assert_ne!(
            event.data.get("status").and_then(|s| s.as_str()),
            Some("muxing")
        );
    }

    #[test]
    fn test_parse_completed() {
        let parser = OutputParser::new();

        // "All done" 应该触发 completed 状态
        let result = parser.parse("21:05:15.123 INFO : All done");
        assert!(result.is_some());
        let event = result.unwrap();
        assert_eq!(event.event_type, "status");
        assert_eq!(event.data["status"], "completed");
    }
}
