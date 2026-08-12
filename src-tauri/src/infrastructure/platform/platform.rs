//! 平台抽象层
//!
//! 提供跨平台的配置和工具名称解析
//! 支持 Windows / macOS / Linux，x64 / arm64

use std::path::{Path, PathBuf};

/// Windows 平台：创建进程时隐藏控制台窗口（`CREATE_NO_WINDOW`）
///
/// 子进程执行的唯一定义点（消除 download.rs / manager.rs / detector.rs 的重复）
#[cfg(target_os = "windows")]
pub const CREATE_NO_WINDOW: u32 = 0x08000000;

/// 支持的平台
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Windows,
    MacOS,
    Linux,
}

/// CPU 架构
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arch {
    X64,
    Arm64,
}

impl Arch {
    /// 获取当前 CPU 架构（编译目标）
    pub fn current() -> Self {
        #[cfg(target_arch = "aarch64")]
        {
            Arch::Arm64
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            Arch::X64
        }
    }

    /// 架构关键字（用于资产文件名匹配）
    pub fn keyword(&self) -> &'static str {
        match self {
            Arch::X64 => "x64",
            Arch::Arm64 => "arm64",
        }
    }
}

impl Platform {
    /// 获取当前平台
    pub fn current() -> Self {
        #[cfg(target_os = "windows")]
        {
            Platform::Windows
        }
        #[cfg(target_os = "macos")]
        {
            Platform::MacOS
        }
        #[cfg(target_os = "linux")]
        {
            Platform::Linux
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            compile_error!("Unsupported platform")
        }
    }

    /// 获取当前架构
    pub fn arch(&self) -> Arch {
        Arch::current()
    }

    /// 获取可执行文件扩展名
    pub fn exe_extension(&self) -> &'static str {
        match self {
            Platform::Windows => ".exe",
            Platform::MacOS | Platform::Linux => "",
        }
    }

    /// 获取动态库扩展名
    #[allow(dead_code)]
    pub fn lib_extension(&self) -> &'static str {
        match self {
            Platform::Windows => ".dll",
            Platform::MacOS => ".dylib",
            Platform::Linux => ".so",
        }
    }

    /// 获取 GitHub release 资产名称的操作系统关键字（不含架构）
    pub fn release_keywords(&self) -> &'static [&'static str] {
        match self {
            Platform::Windows => &["win", "windows"],
            Platform::MacOS => &["macos", "darwin", "osx"],
            Platform::Linux => &["linux", "ubuntu"],
        }
    }

    /// 明确属于「其他平台」的关键字
    ///
    /// 资产名命中任意一个即排除——防止形如 `android-bionic-x64`、
    /// `ffmpeg-master-latest-linux64-*.tar.xz` 的资产因包含架构关键字被误选。
    fn other_platform_keywords(&self) -> &'static [&'static str] {
        match self {
            Platform::Windows => &["linux", "ubuntu", "android", "osx", "macos", "darwin"],
            Platform::MacOS => &[
                "linux", "ubuntu", "android", "windows", "win64", "winarm64", "win32", "win-",
            ],
            Platform::Linux => &[
                "android", "windows", "win64", "winarm64", "win32", "win-", "osx", "macos",
                "darwin",
            ],
        }
    }

    /// 获取架构关键字
    pub fn arch_keywords(&self) -> &'static str {
        self.arch().keyword()
    }

    /// 获取组合的平台+架构关键字
    /// 这些关键字同时表示平台和架构，如 "win64" 同时表示 Windows 和 x64
    pub fn combined_keywords_for(&self, arch: Arch) -> &'static [&'static str] {
        match (self, arch) {
            (Platform::Windows, Arch::X64) => &["win64", "win-x64", "windows-x64"],
            (Platform::Windows, Arch::Arm64) => &["winarm64", "win-arm64", "windows-arm64"],
            (Platform::MacOS, Arch::Arm64) => {
                &["macos-arm64", "darwin-arm64", "osx-arm64", "macos-aarch64"]
            }
            (Platform::MacOS, Arch::X64) => &["macos-x64", "darwin-x64", "osx-x64", "macos-amd64"],
            (Platform::Linux, Arch::X64) => &["linux64", "linux-x64", "linux-amd64"],
            (Platform::Linux, Arch::Arm64) => &["linuxarm64", "linux-arm64", "linux-aarch64"],
        }
    }

    /// 获取组合的平台+架构关键字（当前架构）
    pub fn combined_keywords(&self) -> &'static [&'static str] {
        self.combined_keywords_for(self.arch())
    }

    /// 判断给定的文件名是否适合当前平台
    pub fn is_platform_asset(&self, filename: &str) -> bool {
        self.is_platform_asset_for(filename, self.arch())
    }

    /// 判断给定的文件名是否适合指定平台+架构（可注入，便于测试）
    ///
    /// 匹配顺序：
    /// 1. 排除明确属于其他平台的资产（如 Windows 上排除含 "linux"/"android" 的名称）
    /// 2. 命中组合关键字（如 "win64"、"linux-x64"、"osx-arm64"）
    /// 3. 同时包含平台关键字与架构关键字
    pub fn is_platform_asset_for(&self, filename: &str, arch: Arch) -> bool {
        let filename_lower = filename.to_lowercase();

        // 1. 排除其他平台资产
        if self
            .other_platform_keywords()
            .iter()
            .any(|kw| filename_lower.contains(kw))
        {
            return false;
        }

        // 2. 组合关键字（如 win64 同时满足平台和架构要求）
        if self
            .combined_keywords_for(arch)
            .iter()
            .any(|kw| filename_lower.contains(kw))
        {
            return true;
        }

        // 3. 平台关键字 + 架构关键字同时满足
        let has_platform = self
            .release_keywords()
            .iter()
            .any(|kw| filename_lower.contains(kw));
        let has_arch = filename_lower.contains(arch.keyword());

        has_platform && has_arch
    }
}

/// 工具可执行文件名解析器
pub struct ExeNames {
    /// 主可执行文件名（不带扩展名）
    pub main: &'static str,
    /// 附加可执行文件名列表（不带扩展名）
    pub extras: &'static [&'static str],
}

impl ExeNames {
    /// 创建新的可执行文件名配置
    pub const fn new(main: &'static str, extras: &'static [&'static str]) -> Self {
        Self { main, extras }
    }

    /// 获取指定平台下带扩展名的主可执行文件名
    pub fn main_exe_for(&self, platform: Platform) -> String {
        format!("{}{}", self.main, platform.exe_extension())
    }

    /// 获取指定平台下所有带扩展名的可执行文件名
    pub fn all_exe_for(&self, platform: Platform) -> Vec<String> {
        let mut names = vec![self.main_exe_for(platform)];
        for extra in self.extras {
            names.push(format!("{}{}", extra, platform.exe_extension()));
        }
        names
    }

    /// 获取带平台扩展名的主可执行文件名
    pub fn main_exe(&self) -> String {
        self.main_exe_for(Platform::current())
    }

    /// 获取所有带平台扩展名的可执行文件名
    pub fn all_exe(&self) -> Vec<String> {
        self.all_exe_for(Platform::current())
    }

    /// 在指定目录中查找主可执行文件
    pub fn find_in_dir(&self, dir: &Path) -> Option<PathBuf> {
        let exe_name = self.main_exe();
        let full_path = dir.join(&exe_name);
        if full_path.exists() {
            return Some(full_path);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_current() {
        let platform = Platform::current();
        assert!(matches!(
            platform,
            Platform::Windows | Platform::MacOS | Platform::Linux
        ));
    }

    #[test]
    fn test_is_platform_asset_windows() {
        // FFmpeg (BtbN) 资产名
        assert!(Platform::Windows
            .is_platform_asset_for("ffmpeg-master-latest-win64-gpl-shared.zip", Arch::X64));
        assert!(Platform::Windows
            .is_platform_asset_for("ffmpeg-n8.0-latest-win64-gpl-shared-8.0.zip", Arch::X64));
        assert!(Platform::Windows
            .is_platform_asset_for("ffmpeg-master-latest-winarm64-gpl-shared.zip", Arch::Arm64));
        // 架构不匹配
        assert!(!Platform::Windows
            .is_platform_asset_for("ffmpeg-master-latest-winarm64-gpl-shared.zip", Arch::X64));
        assert!(!Platform::Windows
            .is_platform_asset_for("ffmpeg-master-latest-win64-gpl-shared.zip", Arch::Arm64));
        // 其他平台资产必须排除
        assert!(
            !Platform::Windows.is_platform_asset("ffmpeg-master-latest-linux64-gpl-shared.tar.xz")
        );
        assert!(!Platform::Windows
            .is_platform_asset("N_m3u8DL-RE_v0.6.0-beta_osx-arm64_20260629.tar.gz"));

        // N_m3u8DL-RE 资产名
        assert!(Platform::Windows.is_platform_asset("N_m3u8DL-RE_v0.5.1-beta_win-x64_20251029.zip"));
        // win-x86 / win-NT6.0 不适合 x64
        assert!(!Platform::Windows.is_platform_asset_for(
            "N_m3u8DL-RE_v0.6.0-beta_win-NT6.0-x86_20260629.zip",
            Arch::X64
        ));
    }

    #[test]
    fn test_is_platform_asset_macos() {
        assert!(Platform::MacOS.is_platform_asset_for(
            "N_m3u8DL-RE_v0.6.0-beta_osx-arm64_20260629.tar.gz",
            Arch::Arm64
        ));
        assert!(Platform::MacOS
            .is_platform_asset_for("N_m3u8DL-RE_v0.6.0-beta_osx-x64_20260629.tar.gz", Arch::X64));
        // 架构不匹配
        assert!(!Platform::MacOS.is_platform_asset_for(
            "N_m3u8DL-RE_v0.6.0-beta_osx-x64_20260629.tar.gz",
            Arch::Arm64
        ));
        // android 资产不得误匹配（旧实现的回归用例：arm64 关键字曾属于平台关键字）
        assert!(!Platform::MacOS.is_platform_asset_for(
            "N_m3u8DL-RE_v0.6.0-beta_android-bionic-arm64_20260629.tar.gz",
            Arch::Arm64
        ));
        // Windows 资产排除
        assert!(!Platform::MacOS.is_platform_asset("N_m3u8DL-RE_v0.6.0-beta_win-x64_20260629.zip"));
    }

    #[test]
    fn test_is_platform_asset_linux() {
        assert!(Platform::Linux
            .is_platform_asset_for("ffmpeg-n8.0-latest-linux-x64-gpl-shared-8.0.zip", Arch::X64));
        // BtbN 的 linux64 / linuxarm64 命名（无连字符）
        assert!(Platform::Linux
            .is_platform_asset_for("ffmpeg-master-latest-linux64-gpl-shared.tar.xz", Arch::X64));
        assert!(Platform::Linux.is_platform_asset_for(
            "ffmpeg-master-latest-linuxarm64-gpl-shared.tar.xz",
            Arch::Arm64
        ));
        assert!(!Platform::Linux.is_platform_asset_for(
            "ffmpeg-master-latest-linuxarm64-gpl-shared.tar.xz",
            Arch::X64
        ));
        assert!(Platform::Linux.is_platform_asset_for(
            "N_m3u8DL-RE_v0.6.0-beta_linux-x64_20260629.tar.gz",
            Arch::X64
        ));
        // android 资产不得误匹配
        assert!(!Platform::Linux.is_platform_asset_for(
            "N_m3u8DL-RE_v0.6.0-beta_android-bionic-x64_20260629.tar.gz",
            Arch::X64
        ));
    }

    #[test]
    fn test_exe_names() {
        let ffmpeg = ExeNames::new("ffmpeg", &["ffprobe"]);
        assert_eq!(ffmpeg.main_exe_for(Platform::Windows), "ffmpeg.exe");
        assert_eq!(
            ffmpeg.all_exe_for(Platform::Windows),
            vec!["ffmpeg.exe", "ffprobe.exe"]
        );
        assert_eq!(ffmpeg.main_exe_for(Platform::Linux), "ffmpeg");
        assert_eq!(
            ffmpeg.all_exe_for(Platform::Linux),
            vec!["ffmpeg", "ffprobe"]
        );
        assert_eq!(ffmpeg.main_exe(), ffmpeg.main_exe_for(Platform::current()));
        assert_eq!(ffmpeg.all_exe(), ffmpeg.all_exe_for(Platform::current()));
    }
}
