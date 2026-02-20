//! 平台抽象层
//!
//! 提供跨平台的配置和工具名称解析
//! 目前只支持 Windows，但架构设计便于后续扩展

use std::path::{Path, PathBuf};

/// 支持的平台
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Windows,
    #[allow(dead_code)]
    MacOS,
    #[allow(dead_code)]
    Linux,
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

    /// 获取 GitHub release 资产名称关键字
    /// 用于在 GitHub releases 中查找适合当前平台的下载包
    pub fn release_keywords(&self) -> &'static [&'static str] {
        match self {
            Platform::Windows => &["win", "windows", "win64", "x64"],
            Platform::MacOS => &["macos", "darwin", "osx", "arm64"],
            Platform::Linux => &["linux", "ubuntu"],
        }
    }

    /// 获取架构关键字
    pub fn arch_keywords(&self) -> &'static str {
        #[cfg(target_arch = "x86_64")]
        {
            "x64"
        }
        #[cfg(target_arch = "aarch64")]
        {
            "arm64"
        }
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            "unknown"
        }
    }

    /// 判断给定的文件名是否适合当前平台
    pub fn is_platform_asset(&self, filename: &str) -> bool {
        let filename_lower = filename.to_lowercase();
        let has_platform = self
            .release_keywords()
            .iter()
            .any(|kw| filename_lower.contains(kw));
        let has_arch = filename_lower.contains(self.arch_keywords());
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

    /// 获取带平台扩展名的主可执行文件名
    pub fn main_exe(&self) -> String {
        format!("{}{}", self.main, Platform::current().exe_extension())
    }

    /// 获取所有带平台扩展名的可执行文件名
    pub fn all_exe(&self) -> Vec<String> {
        let platform = Platform::current();
        let mut names = vec![format!("{}{}", self.main, platform.exe_extension())];
        for extra in self.extras {
            names.push(format!("{}{}", extra, platform.exe_extension()));
        }
        names
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
    fn test_exe_names() {
        let ffmpeg = ExeNames::new("ffmpeg", &["ffprobe"]);
        assert_eq!(ffmpeg.main_exe(), "ffmpeg.exe");
        assert_eq!(ffmpeg.all_exe(), vec!["ffmpeg.exe", "ffprobe.exe"]);
    }
}
