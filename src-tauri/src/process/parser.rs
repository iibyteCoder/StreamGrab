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
    /// 文件大小正则: 匹配 "1.25 GiB" 或 "500 MiB" 等
    size_regex: Regex,
    /// 分片进度正则: 匹配 "100/200" 或 "50 of 100"
    segments_regex: Regex,
    /// ETA 正则: 匹配剩余时间
    eta_regex: Regex,
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
            complete_regex: Regex::new(r"(?i)(download\s+complete|下载完成|done|merged)")
                .unwrap(),
            // 匹配文件大小: 1.25 GiB, 500 MiB 等
            size_regex: Regex::new(r"(\d+(?:\.\d+)?)\s*(GiB|GB|MiB|MB|KiB|KB|B)")
                .unwrap(),
            // 匹配分片进度: 100/200 或 50 of 100
            segments_regex: Regex::new(r"(\d+)\s*(?:/|of)\s*(\d+)")
                .unwrap(),
            // 匹配 ETA: 剩余时间
            eta_regex: Regex::new(r"(?i)(?:ETA|剩余)[:\s]*(\d+):(\d+):(\d+)|(\d+):(\d+)|(\d+)\s*s(?:ec)?")
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
            .map(|s| self.parse_size_to_bytes(s))
            .unwrap_or(0);

        // 尝试提取文件大小
        let (downloaded_size, total_size) = self.parse_sizes(line);

        // 尝试提取分片进度
        let (downloaded_segments, total_segments) = self.parse_segments(line);

        // 尝试提取 ETA
        let eta = self.parse_eta(line);

        Some(ParsedEvent {
            event_type: "progress".to_string(),
            data: serde_json::json!({
                "percent": percent,
                "speed": speed,
                "speedStr": speed_str,
                "downloadedSize": downloaded_size,
                "totalSize": total_size,
                "downloadedSegments": downloaded_segments,
                "totalSegments": total_segments,
                "eta": eta
            }),
        })
    }

    /// 解析文件大小信息
    /// 返回 (已下载大小, 总大小)
    fn parse_sizes(&self, line: &str) -> (u64, u64) {
        // 尝试匹配 "X / Y GiB" 或 "X/Y MiB" 格式
        let size_pairs: Vec<_> = self.size_regex.captures_iter(line).collect();

        if size_pairs.len() >= 2 {
            // 如果有两个大小值，第一个是已下载，第二个是总大小
            let downloaded = self.size_captures_to_bytes(&size_pairs[0]);
            let total = self.size_captures_to_bytes(&size_pairs[1]);
            return (downloaded, total);
        } else if size_pairs.len() == 1 {
            // 只有一个大小值，根据进度百分比估算总大小
            let downloaded = self.size_captures_to_bytes(&size_pairs[0]);
            // 尝试从进度百分比提取
            if let Some(percent_caps) = self.progress_regex.captures(line) {
                if let Ok(percent) = percent_caps[1].parse::<f64>() {
                    if percent > 0.0 {
                        let total = (downloaded as f64 / percent * 100.0) as u64;
                        return (downloaded, total);
                    }
                }
            }
            return (downloaded, 0);
        }

        (0, 0)
    }

    /// 将正则捕获组转换为字节数
    fn size_captures_to_bytes(&self, caps: &regex::Captures) -> u64 {
        let num: f64 = caps[1].parse().unwrap_or(0.0);
        let unit = &caps[2];

        match unit.to_lowercase().as_str() {
            "gib" | "gb" => (num * 1024.0 * 1024.0 * 1024.0) as u64,
            "mib" | "mb" => (num * 1024.0 * 1024.0) as u64,
            "kib" | "kb" => (num * 1024.0) as u64,
            _ => num as u64,
        }
    }

    /// 解析分片进度
    /// 返回 (已下载分片, 总分片)
    fn parse_segments(&self, line: &str) -> (u32, u32) {
        if let Some(caps) = self.segments_regex.captures(line) {
            if let (Ok(downloaded), Ok(total)) = (caps[1].parse::<u32>(), caps[2].parse::<u32>()) {
                return (downloaded, total);
            }
        }
        (0, 0)
    }

    /// 解析 ETA（剩余时间）
    /// 返回秒数
    fn parse_eta(&self, line: &str) -> u32 {
        if let Some(caps) = self.eta_regex.captures(line) {
            // 尝试匹配 HH:MM:SS 格式
            if let (Some(h), Some(m), Some(s)) = (
                caps.get(1).and_then(|m| m.as_str().parse::<u32>().ok()),
                caps.get(2).and_then(|m| m.as_str().parse::<u32>().ok()),
                caps.get(3).and_then(|m| m.as_str().parse::<u32>().ok()),
            ) {
                return h * 3600 + m * 60 + s;
            }
            // 尝试匹配 MM:SS 格式
            if let (Some(m), Some(s)) = (
                caps.get(4).and_then(|m| m.as_str().parse::<u32>().ok()),
                caps.get(5).and_then(|m| m.as_str().parse::<u32>().ok()),
            ) {
                return m * 60 + s;
            }
            // 尝试匹配秒数格式
            if let Some(s) = caps.get(6).and_then(|m| m.as_str().parse::<u32>().ok()) {
                return s;
            }
        }
        0
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

    /// 将大小字符串转换为字节（保留公开方法供速度解析使用）
    fn parse_size_to_bytes(&self, size_str: &str) -> u64 {
        let lower = size_str.to_lowercase();

        // 提取数字部分
        let num: f64 = self.progress_regex
            .captures(size_str)
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

    #[test]
    fn test_parse_segments() {
        let parser = OutputParser::new();

        let result = parser.parse("Progress: 50/100 segments, 50.0%");
        assert!(result.is_some());

        let event = result.unwrap();
        let data = event.data.as_object().unwrap();
        assert_eq!(data["downloadedSegments"], 50);
        assert_eq!(data["totalSegments"], 100);
    }
}
