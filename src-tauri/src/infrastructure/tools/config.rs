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
        // "0.6.0+df70f0b3da0c..."（v0.6.0+：纯版本号+构建哈希）
        // "N_m3u8DL-RE version 0.3.0.0"（旧版本）

        let combined = format!("{}\n{}", stdout, stderr);

        // 尝试匹配 "version X.X.X" 或 "vX.X.X" 格式
        let patterns = [
            r"version\s+(\d+\.\d+\.\d+(?:\.\d+)?)", // version 0.3.0.0
            r"v(\d+\.\d+\.\d+(?:\.\d+)?)",          // v0.3.0.0
            r"(\d+\.\d+\.\d+(?:\.\d+)?)",           // 直接版本号 0.6.0
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

    fn find_release_asset(&self, assets: &[Value], platform: Platform) -> Option<(String, String)> {
        // 官方发布格式：Windows 为 .zip，macOS/Linux 自 v0.6.0 起为 .tar.gz
        for asset in assets {
            let name = asset["name"].as_str().unwrap_or("");
            let download_url = asset["browser_download_url"].as_str().unwrap_or("");
            let name_lower = name.to_lowercase();

            let format_ok = match platform {
                Platform::Windows => name_lower.ends_with(".zip"),
                Platform::MacOS | Platform::Linux => {
                    name_lower.ends_with(".tar.gz") || name_lower.ends_with(".zip")
                }
            };

            if format_ok && platform.is_platform_asset(name) {
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
        // 注意：BtbN 不提供 macOS 构建，macOS 由 fetch_release 改走 evermeet.cx 源
        Some("BtbN/FFmpeg-Builds")
    }

    fn find_release_asset(&self, assets: &[Value], platform: Platform) -> Option<(String, String)> {
        // BtbN 命名示例：
        //   ffmpeg-master-latest-win64-gpl-shared.zip（Windows x64）
        //   ffmpeg-master-latest-winarm64-gpl-shared.zip（Windows arm64）
        //   ffmpeg-master-latest-linux64-gpl-shared.tar.xz（Linux x64）
        let format_ok = |name_lower: &str| match platform {
            Platform::Windows => name_lower.ends_with(".zip"),
            Platform::Linux => name_lower.ends_with(".tar.xz") || name_lower.ends_with(".zip"),
            // macOS 走 evermeet.cx 源，理论上不会到达这里
            Platform::MacOS => false,
        };

        // 两轮匹配：优先 master-latest 滚动构建，其次回退固定版本（如 n7.1/n8.1）
        for prefer_master in [true, false] {
            for asset in assets {
                let name = asset["name"].as_str().unwrap_or("");
                let name_lower = name.to_lowercase();

                // shared 版本包含 ffprobe；排除 lgpl（"lgpl" 包含子串 "gpl"，须显式排除）
                if !format_ok(&name_lower)
                    || !platform.is_platform_asset(name)
                    || !name_lower.contains("gpl")
                    || name_lower.contains("lgpl")
                    || !name_lower.contains("shared")
                {
                    continue;
                }
                if name_lower.contains("master-latest") != prefer_master {
                    continue;
                }

                let download_url = asset["browser_download_url"].as_str().unwrap_or("");
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

// ========================================
// 测试
// ========================================

#[cfg(test)]
mod tests {
    use super::super::super::platform::Arch;
    use super::*;
    use serde_json::json;

    /// 构造 GitHub release 资产（真实字段：name + browser_download_url）
    fn asset(name: &str) -> Value {
        json!({
            "name": name,
            "browser_download_url": format!("https://example.com/{name}"),
        })
    }

    /// N_m3u8DL-RE v0.6.0-beta 的真实资产列表（官方 release）
    fn nm3u8dl_assets() -> Vec<Value> {
        [
            "N_m3u8DL-RE_v0.6.0-beta_android-bionic-arm64_20260629.tar.gz",
            "N_m3u8DL-RE_v0.6.0-beta_android-bionic-x64_20260629.tar.gz",
            "N_m3u8DL-RE_v0.6.0-beta_linux-arm64_20260629.tar.gz",
            "N_m3u8DL-RE_v0.6.0-beta_linux-x64_20260629.tar.gz",
            "N_m3u8DL-RE_v0.6.0-beta_osx-arm64_20260629.tar.gz",
            "N_m3u8DL-RE_v0.6.0-beta_osx-x64_20260629.tar.gz",
            "N_m3u8DL-RE_v0.6.0-beta_win-arm64_20260629.zip",
            "N_m3u8DL-RE_v0.6.0-beta_win-NT6.0-x86_20260629.zip",
            "N_m3u8DL-RE_v0.6.0-beta_win-x64_20260629.zip",
        ]
        .iter()
        .map(|n| asset(n))
        .collect()
    }

    #[test]
    fn nm3u8dl_picks_win_x64_zip_on_windows() {
        let config = Nm3u8dlReConfig;
        let (url, filename) = config
            .find_release_asset(&nm3u8dl_assets(), Platform::Windows)
            .expect("Windows x64 应匹配 win-x64.zip");
        assert_eq!(filename, "N_m3u8DL-RE_v0.6.0-beta_win-x64_20260629.zip");
        assert!(url.ends_with(&filename));
    }

    #[test]
    fn nm3u8dl_picks_osx_asset_on_macos() {
        // find_release_asset 使用运行期架构；逐架构匹配矩阵见 platform.rs 测试
        let config = Nm3u8dlReConfig;
        let (_url, filename) = config
            .find_release_asset(&nm3u8dl_assets(), Platform::MacOS)
            .expect("macOS 应匹配当前架构的 osx tar.gz");
        let expected = match Arch::current() {
            Arch::Arm64 => "N_m3u8DL-RE_v0.6.0-beta_osx-arm64_20260629.tar.gz",
            Arch::X64 => "N_m3u8DL-RE_v0.6.0-beta_osx-x64_20260629.tar.gz",
        };
        assert_eq!(filename, expected);
    }

    #[test]
    fn nm3u8dl_picks_linux_x64_tar_gz_on_linux() {
        let config = Nm3u8dlReConfig;
        let (_url, filename) = config
            .find_release_asset(&nm3u8dl_assets(), Platform::Linux)
            .expect("Linux 应匹配 linux-x64.tar.gz");
        assert_eq!(
            filename,
            "N_m3u8DL-RE_v0.6.0-beta_linux-x64_20260629.tar.gz"
        );
    }

    /// BtbN FFmpeg-Builds latest 的真实资产列表（节选）
    fn ffmpeg_assets() -> Vec<Value> {
        [
            "checksums.sha256",
            "ffmpeg-master-latest-linux64-gpl-shared.tar.xz",
            "ffmpeg-master-latest-linux64-gpl.tar.xz",
            "ffmpeg-master-latest-linux64-lgpl-shared.tar.xz",
            "ffmpeg-master-latest-linuxarm64-gpl-shared.tar.xz",
            "ffmpeg-master-latest-win64-gpl-shared.zip",
            "ffmpeg-master-latest-win64-gpl.zip",
            "ffmpeg-master-latest-win64-lgpl-shared.zip",
            "ffmpeg-master-latest-winarm64-gpl-shared.zip",
            "ffmpeg-n7.1-latest-win64-gpl-shared-7.1.zip",
            "ffmpeg-n8.1-latest-linux64-gpl-shared-8.1.tar.xz",
        ]
        .iter()
        .map(|n| asset(n))
        .collect()
    }

    #[test]
    fn ffmpeg_picks_master_win64_gpl_shared_zip_on_windows() {
        let config = FfmpegConfig;
        let (_url, filename) = config
            .find_release_asset(&ffmpeg_assets(), Platform::Windows)
            .expect("Windows 应匹配 win64 gpl shared zip");
        // 必须是 master-latest 的 gpl-shared，而非 lgpl-shared 或固定版本
        assert_eq!(filename, "ffmpeg-master-latest-win64-gpl-shared.zip");
    }

    #[test]
    fn ffmpeg_picks_linux64_tar_xz_on_linux() {
        let config = FfmpegConfig;
        let (_url, filename) = config
            .find_release_asset(&ffmpeg_assets(), Platform::Linux)
            .expect("Linux 应匹配 linux64 gpl shared tar.xz");
        assert_eq!(filename, "ffmpeg-master-latest-linux64-gpl-shared.tar.xz");
    }

    #[test]
    fn ffmpeg_returns_none_on_macos() {
        // BtbN 无 macOS 构建；macOS 由 fetch_release 改走 evermeet.cx
        let config = FfmpegConfig;
        assert!(config
            .find_release_asset(&ffmpeg_assets(), Platform::MacOS)
            .is_none());
    }

    #[test]
    fn ffmpeg_falls_back_to_pinned_version_without_master() {
        let config = FfmpegConfig;
        let assets = vec![
            asset("ffmpeg-n7.1-latest-win64-lgpl-shared-7.1.zip"),
            asset("ffmpeg-n7.1-latest-win64-gpl-shared-7.1.zip"),
        ];
        let (_url, filename) = config
            .find_release_asset(&assets, Platform::Windows)
            .expect("无 master 构建时应回退固定版本");
        assert_eq!(filename, "ffmpeg-n7.1-latest-win64-gpl-shared-7.1.zip");
    }

    #[test]
    fn nm3u8dl_parse_version_handles_hash_suffix() {
        let config = Nm3u8dlReConfig;
        // v0.6.0+ 实际输出：纯版本号 + 构建哈希
        let v = config.parse_version("0.6.0+df70f0b3da0c630bd413bf617e758051f6b64757", "");
        assert_eq!(v.as_deref(), Some("0.6.0"));
        // 旧版格式
        let v = config.parse_version("N_m3u8DL-RE version 0.3.0.0", "");
        assert_eq!(v.as_deref(), Some("0.3.0.0"));
    }

    #[test]
    fn platform_arch_keyword_sanity() {
        // 防止回归：架构关键字与组合关键字一致
        assert_eq!(Arch::X64.keyword(), "x64");
        assert_eq!(Arch::Arm64.keyword(), "arm64");
        assert!(Platform::Windows
            .combined_keywords_for(Arch::X64)
            .contains(&"win-x64"));
        assert!(Platform::MacOS
            .combined_keywords_for(Arch::Arm64)
            .contains(&"osx-arm64"));
        assert!(Platform::Linux
            .combined_keywords_for(Arch::X64)
            .contains(&"linux64"));
    }
}
