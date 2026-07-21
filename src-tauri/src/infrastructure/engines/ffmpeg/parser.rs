//! FFmpeg 输出解析器
//!
//! 解析两类输出：
//! 1. 启动阶段的 `Duration: 00:05:30.00, ...` 行 → 总时长（微秒）
//! 2. `-progress pipe:2` 的 key=value 块（以 `progress=continue/end` 结尾）

use regex::Regex;

/// FFmpeg 输出解析器（正则预编译，可跨任务共享）
pub struct FfmpegOutputParser {
    duration_re: Regex,
}

impl FfmpegOutputParser {
    pub fn new() -> Self {
        Self {
            duration_re: Regex::new(r"Duration:\s*(\d{2}):(\d{2}):(\d{2})\.(\d{2})").unwrap(),
        }
    }

    /// 解析 `Duration: HH:MM:SS.ms` → 总时长（微秒）
    pub fn parse_duration(&self, output: &str) -> Option<i64> {
        for line in output.lines() {
            if let Some(caps) = self.duration_re.captures(line) {
                let hours: i64 = caps.get(1)?.as_str().parse().ok()?;
                let minutes: i64 = caps.get(2)?.as_str().parse().ok()?;
                let seconds: i64 = caps.get(3)?.as_str().parse().ok()?;
                let centiseconds: i64 = caps.get(4)?.as_str().parse().ok()?;
                return Some(
                    (hours * 3600 + minutes * 60 + seconds) * 1_000_000 + centiseconds * 10_000,
                );
            }
        }
        None
    }

    /// 解析一个完整的 -progress 块
    ///
    /// ```text
    /// out_time_us=83450000
    /// total_size=12345678
    /// bitrate=1234.5kbits/s
    /// speed=1.00x
    /// progress=continue
    /// ```
    ///
    /// 返回 `(当前时间微秒, 已写入字节数, 速度 bytes/s)`
    pub fn parse_progress_block(&self, block: &str) -> Option<(i64, Option<u64>, Option<i64>)> {
        let mut out_time_us: Option<i64> = None;
        let mut total_size: Option<u64> = None;
        let mut bitrate_bps: Option<i64> = None;
        let mut speed: Option<f64> = None;

        for line in block.lines() {
            let line = line.trim();
            if let Some(v) = line.strip_prefix("out_time_us=") {
                out_time_us = v.parse().ok();
            } else if let Some(v) = line.strip_prefix("total_size=") {
                total_size = v.parse().ok();
            } else if let Some(v) = line.strip_prefix("bitrate=") {
                // `1234.5kbits/s` → bits/s
                let v = v.trim().trim_end_matches("kbits/s");
                if let Ok(kbits) = v.parse::<f64>() {
                    bitrate_bps = Some((kbits * 1000.0) as i64);
                }
            } else if let Some(v) = line.strip_prefix("speed=") {
                let v = v.trim().trim_end_matches('x').trim();
                speed = v.parse().ok();
            }
        }

        let time = out_time_us?;
        // 速度：优先 bitrate；无 bitrate 时按 1.5Mbps 基准用 speed 估算
        let bytes_per_sec = bitrate_bps.or_else(|| speed.map(|s| (s * 1_500_000.0) as i64));
        Some((time, total_size, bytes_per_sec))
    }
}

impl Default for FfmpegOutputParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_duration_line() {
        let parser = FfmpegOutputParser::new();
        let line = "  Duration: 00:05:30.00, start: 0.000000, bitrate: 1234 kb/s";
        assert_eq!(parser.parse_duration(line), Some((5 * 60 + 30) * 1_000_000));
        assert_eq!(parser.parse_duration("no duration here"), None);
    }

    #[test]
    fn parses_progress_block_with_bitrate() {
        let parser = FfmpegOutputParser::new();
        let block = "\
frame=123
fps=30.00
bitrate=1234.5kbits/s
total_size=12345678
out_time_us=83450000
speed=2.00x
progress=continue";

        let (time, size, speed) = parser.parse_progress_block(block).unwrap();
        assert_eq!(time, 83_450_000);
        assert_eq!(size, Some(12_345_678));
        // 优先 bitrate: 1234.5 kbits/s → 1_234_500 bits/s
        assert_eq!(speed, Some(1_234_500));
    }

    #[test]
    fn falls_back_to_speed_estimate() {
        let parser = FfmpegOutputParser::new();
        let block = "out_time_us=1000000\nspeed=2.00x\nprogress=continue";
        let (_, _, speed) = parser.parse_progress_block(block).unwrap();
        // 2.0x * 1.5Mbps 估算
        assert_eq!(speed, Some(3_000_000));
    }

    #[test]
    fn requires_out_time() {
        let parser = FfmpegOutputParser::new();
        assert!(parser
            .parse_progress_block("speed=1.0x\nprogress=end")
            .is_none());
    }
}
