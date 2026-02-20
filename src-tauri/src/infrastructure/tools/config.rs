//! 工具配置实现
//!
//! 定义具体工具（N_m3u8DL-RE、FFmpeg、FFprobe）的行为特性

use super::super::platform::{ExeNames, Platform};
use super::{tool_names, ToolDefinition};

use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::OnceLock;

// ========================================
// N_m3u8DL-RE 下载器
// ========================================

/// N_m3u8DL-RE 工具定义
pub struct Nm3u8dlReConfig;

impl ToolDefinition for Nm3u8dlReConfig {
    fn name(&self) -> &'static str {
        tool_names::DOWNLOADER
    }

    fn exe_names(&self) -> ExeNames {
        ExeNames::new("N_m3u8DL-RE", &[])
    }

    fn version_args(&self) -> &'static [&'static str] {
        &["--version"]
    }

    fn parse_version(&self, stdout: &str, stderr: &str) -> Option<String> {
        // N_m3u8DL-RE 版本输出格式示例：
        // "N_m3u8DL-RE version 0.3.0.0"
        // 或直接输出版本号

        let combined = format!("{}\n{}", stdout, stderr);

        // 尝试匹配 "version X.X.X" 或 "vX.X.X" 格式
        let patterns = [
            r"version\s+(\d+\.\d+\.\d+(?:\.\d+)?)", // version 0.3.0.0
            r"v(\d+\.\d+\.\d+(?:\.\d+)?)",          // v0.3.0.0
            r"(\d+\.\d+\.\d+(?:\.\d+)?)",           // 直接版本号 0.3.0.0
        ];

        for pattern in &patterns {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(cap) = re.captures(&combined) {
                    if let Some(m) = cap.get(1) {
                        return Some(m.as_str().to_string());
                    }
                }
            }
        }

        None
    }

    fn github_repo(&self) -> Option<&'static str> {
        Some("nilaoda/N_m3u8DL-RE")
    }

    fn find_release_asset(&self, assets: &[Value]) -> Option<(String, String)> {
        let platform = Platform::current();

        for asset in assets {
            let name = asset["name"].as_str().unwrap_or("");
            let download_url = asset["browser_download_url"].as_str().unwrap_or("");

            // 查找 ZIP 格式的平台匹配版本
            if name.ends_with(".zip") && platform.is_platform_asset(name) {
                let filename = name.rsplit('/').next().unwrap_or(name);
                return Some((download_url.to_string(), filename.to_string()));
            }
        }

        None
    }
}

// ========================================
// FFmpeg 主程序
// ========================================

/// FFmpeg 工具定义
pub struct FfmpegConfig;

impl ToolDefinition for FfmpegConfig {
    fn name(&self) -> &'static str {
        tool_names::FFMPEG
    }

    fn exe_names(&self) -> ExeNames {
        ExeNames::new("ffmpeg", &["ffprobe"])
    }

    fn version_args(&self) -> &'static [&'static str] {
        &["-version"]
    }

    fn parse_version(&self, stdout: &str, _stderr: &str) -> Option<String> {
        // FFmpeg 版本输出格式示例：
        // 标准版本: "ffmpeg version 8.0"
        // BtbN 构建: "ffmpeg version N-118800-gbe4c3c2859-20260219"

        let first_line = stdout.lines().next()?;

        // 尝试匹配标准版本号格式
        let std_re = Regex::new(r"ffmpeg\s+version\s+(\d+\.\d+(?:\.\d+)?)").ok()?;
        if let Some(cap) = std_re.captures(first_line) {
            return cap.get(1).map(|m| m.as_str().to_string());
        }

        // 尝试匹配 BtbN 构建格式：N-118800-gbe4c3c2859-20260219
        // 提取日期部分作为版本标识
        let btbm_re = Regex::new(r"ffmpeg\s+version\s+N-\d+-[a-f0-9]+-(\d{8})").ok()?;
        if let Some(cap) = btbm_re.captures(first_line) {
            if let Some(date_match) = cap.get(1) {
                let date = date_match.as_str();
                // 格式化为 YYYY-MM-DD
                return Some(format!("{}-{}-{}", &date[0..4], &date[4..6], &date[6..8]));
            }
        }

        // 回退：提取任何类似版本号的模式
        let fallback_re = Regex::new(r"(\d{4}-\d{2}-\d{2})").ok()?;
        if let Some(cap) = fallback_re.captures(first_line) {
            return cap.get(1).map(|m| m.as_str().to_string());
        }

        None
    }

    fn github_repo(&self) -> Option<&'static str> {
        Some("BtbN/FFmpeg-Builds")
    }

    fn find_release_asset(&self, assets: &[Value]) -> Option<(String, String)> {
        let platform = Platform::current();

        for asset in assets {
            let name = asset["name"].as_str().unwrap_or("");
            let download_url = asset["browser_download_url"].as_str().unwrap_or("");

            // 查找 shared 版本（包含 ffprobe）
            // 命名格式: ffmpeg-master-latest-win64-gpl-shared.zip
            let name_lower = name.to_lowercase();
            if name.ends_with(".zip")
                && platform.is_platform_asset(name)
                && name_lower.contains("gpl")
                && name_lower.contains("shared")
            {
                let filename = name.rsplit('/').next().unwrap_or(name);
                return Some((download_url.to_string(), filename.to_string()));
            }
        }

        None
    }
}

// ========================================
// FFprobe
// ========================================

/// FFprobe 工具定义
pub struct FfprobeConfig;

impl ToolDefinition for FfprobeConfig {
    fn name(&self) -> &'static str {
        tool_names::FFPROBE
    }

    fn exe_names(&self) -> ExeNames {
        ExeNames::new("ffprobe", &[])
    }

    fn version_args(&self) -> &'static [&'static str] {
        &["-version"]
    }

    fn parse_version(&self, stdout: &str, _stderr: &str) -> Option<String> {
        let first_line = stdout.lines().next()?;
        let re = Regex::new(r"ffprobe\s+version\s+(\d+\.\d+(?:\.\d+)?)").ok()?;
        re.captures(first_line)
            .and_then(|cap| cap.get(1).map(|m| m.as_str().to_string()))
    }

    // FFprobe 不单独下载，随 FFmpeg 套件一起
}

// ========================================
// 工具注册表
// ========================================

/// 工具注册表
///
/// 集中管理所有工具的配置定义
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn ToolDefinition>>,
}

impl ToolRegistry {
    /// 获取全局注册表实例
    pub fn global() -> &'static Self {
        static REGISTRY: OnceLock<ToolRegistry> = OnceLock::new();
        REGISTRY.get_or_init(|| {
            let mut registry = ToolRegistry {
                tools: HashMap::new(),
            };
            registry.register(Box::new(Nm3u8dlReConfig));
            registry.register(Box::new(FfmpegConfig));
            registry.register(Box::new(FfprobeConfig));
            registry
        })
    }

    /// 注册工具
    fn register(&mut self, tool: Box<dyn ToolDefinition>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    /// 获取工具配置
    pub fn get(&self, name: &str) -> Option<&dyn ToolDefinition> {
        self.tools.get(name).map(|b| b.as_ref())
    }

    /// 获取 N_m3u8DL-RE 配置
    pub fn downloader(&self) -> &dyn ToolDefinition {
        self.get(tool_names::DOWNLOADER).unwrap()
    }

    /// 获取 FFmpeg 配置
    pub fn ffmpeg(&self) -> &dyn ToolDefinition {
        self.get(tool_names::FFMPEG).unwrap()
    }

    /// 获取 FFprobe 配置
    pub fn ffprobe(&self) -> &dyn ToolDefinition {
        self.get(tool_names::FFPROBE).unwrap()
    }
}
