//! 输出解析器
//!
//! 解析 N_m3u8DL-RE 的输出并转换为结构化事件

use regex::Regex;
use serde_json::Value;

/// 解析后的事件
#[derive(Debug, Clone)]
pub struct ParsedEvent {
    /// 事件类型
    pub event_type: String,
    /// 事件数据
    pub data: Value,
}

/// 输出解析器
pub struct OutputParser {
    /// 进度正则
    progress_regex: Regex,
    /// 速度正则
    speed_regex: Regex,
    /// 错误正则
    error_regex: Regex,
    /// 完成正则
    complete_regex: Regex,
}

impl OutputParser {
    /// 创建新的解析器
    pub fn new() -> Self {
        Self {
            // 匹配进度: 45.2%
            progress_regex: Regex::new(r"(\d+(?:\.\d+)?)\s*%").unwrap(),
            // 匹配速度: 12.5 MiB/s 或 12.5MB/s
            speed_regex: Regex::new(r"(\d+(?:\.\d+)?)\s*(MiB|MB|GiB|GB|KiB|KB)/s")
                .unwrap(),
            // 匹配错误
            error_regex: Regex::new(r"(?i)(error|failed|exception|错误)").unwrap(),
            // 匹配完成
            complete_regex: Regex::new(r"(?i)(download\s+complete|下载完成|done)")
                .unwrap(),
        }
    }

    /// 解析输出行
    ///
    /// # Arguments
    /// * `line` - 输出行
    ///
    /// # Returns
    /// 解析后的事件，如果无法解析则返回 None
    pub fn parse(&self, line: &str) -> Option<ParsedEvent> {
        let line = line.trim();

        if line.is_empty() {
            return None;
        }

        // 检查是否是完成消息
        if self.complete_regex.is_match(line) {
            return Some(ParsedEvent {
                event_type: "status".to_string(),
                data: serde_json::json!({
                    "status": "completed",
                    "message": line
                }),
            });
        }

        // 检查是否是错误
        if self.error_regex.is_match(line) {
            return Some(ParsedEvent {
                event_type: "log".to_string(),
                data: serde_json::json!({
                    "level": "error",
                    "message": line
                }),
            });
        }

        // 尝试解析进度
        if let Some(progress) = self.parse_progress(line) {
            return Some(progress);
        }

        // 尝试解析日志
        if let Some(log_event) = self.parse_log(line) {
            return Some(log_event);
        }

        // 返回原始日志
        Some(ParsedEvent {
            event_type: "log".to_string(),
            data: serde_json::json!({
                "level": "info",
                "message": line
            }),
        })
    }

    /// 解析进度信息
    fn parse_progress(&self, line: &str) -> Option<ParsedEvent> {
        // 提取进度百分比
        let percent = self.progress_regex
            .captures(line)
            .and_then(|caps| caps[1].parse::<f64>().ok())?;

        // 提取速度
        let speed_str = self.speed_regex
            .captures(line)
            .map(|caps| format!("{} {}/s", &caps[1], &caps[2]));

        // 解析速度值（转换为字节/秒）
        let speed = speed_str
            .as_ref()
            .map(|s| self.parse_speed_to_bytes(s))
            .unwrap_or(0);

        Some(ParsedEvent {
            event_type: "progress".to_string(),
            data: serde_json::json!({
                "percent": percent,
                "speed": speed,
                "speedStr": speed_str,
                "downloadedSize": 0,  // 需要更复杂的解析
                "totalSize": 0,       // 需要更复杂的解析
                "eta": 0              // 需要更复杂的解析
            }),
        })
    }

    /// 解析日志级别
    fn parse_log(&self, line: &str) -> Option<ParsedEvent> {
        let lower = line.to_lowercase();

        let level = if lower.contains("[info]") || lower.contains("[信息]") {
            "info"
        } else if lower.contains("[warn]") || lower.contains("[警告]") {
            "warn"
        } else if lower.contains("[debug]") || lower.contains("[调试]") {
            "debug"
        } else if lower.contains("[error]") || lower.contains("[错误]") {
            "error"
        } else {
            return None;
        };

        Some(ParsedEvent {
            event_type: "log".to_string(),
            data: serde_json::json!({
                "level": level,
                "message": line
            }),
        })
    }

    /// 将速度字符串转换为字节/秒
    fn parse_speed_to_bytes(&self, speed_str: &str) -> u64 {
        let lower = speed_str.to_lowercase();

        // 提取数字部分
        let num: f64 = self.progress_regex
            .captures(speed_str)
            .and_then(|caps| caps[1].parse().ok())
            .unwrap_or(0.0);

        // 根据单位转换
        if lower.contains("gib") || lower.contains("gb") {
            (num * 1024.0 * 1024.0 * 1024.0) as u64
        } else if lower.contains("mib") || lower.contains("mb") {
            (num * 1024.0 * 1024.0) as u64
        } else if lower.contains("kib") || lower.contains("kb") {
            (num * 1024.0) as u64
        } else {
            num as u64
        }
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
    fn test_parse_progress() {
        let parser = OutputParser::new();

        let result = parser.parse("Downloading... 45.2% Speed: 12.5 MiB/s");
        assert!(result.is_some());

        let event = result.unwrap();
        assert_eq!(event.event_type, "progress");

        let data = event.data.as_object().unwrap();
        assert_eq!(data["percent"], 45.2);
    }

    #[test]
    fn test_parse_error() {
        let parser = OutputParser::new();

        let result = parser.parse("Error: Failed to connect to server");
        assert!(result.is_some());

        let event = result.unwrap();
        assert_eq!(event.event_type, "log");

        let data = event.data.as_object().unwrap();
        assert_eq!(data["level"], "error");
    }

    #[test]
    fn test_parse_complete() {
        let parser = OutputParser::new();

        let result = parser.parse("Download Complete!");
        assert!(result.is_some());

        let event = result.unwrap();
        assert_eq!(event.event_type, "status");

        let data = event.data.as_object().unwrap();
        assert_eq!(data["status"], "completed");
    }
}
