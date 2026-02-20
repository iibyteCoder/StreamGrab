//! 工具检测器
//!
//! 负责检测工具是否安装、获取版本信息等

use super::{SuiteInfo, ToolDefinition, ToolInfo, ToolPaths, ToolRegistry};

use std::path::PathBuf;
use std::process::Command;

// Windows 平台：隐藏控制台窗口的标志
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// 工具检测器
pub struct ToolDetector {
    registry: &'static ToolRegistry,
    paths: ToolPaths,
}

impl ToolDetector {
    /// 创建检测器实例
    pub fn new(paths: ToolPaths) -> Self {
        Self {
            registry: ToolRegistry::global(),
            paths,
        }
    }

    /// 检测 N_m3u8DL-RE 下载器
    pub fn detect_downloader(&self) -> ToolInfo {
        let config = self.registry.downloader();
        self.detect_single_tool(config, self.paths.downloader_dir.as_ref())
    }

    /// 检测 FFmpeg 套件（包含 ffmpeg 和 ffprobe）
    pub fn detect_ffmpeg_suite(&self) -> SuiteInfo {
        let dir = self.paths.ffmpeg_dir.as_ref();
        let ffmpeg = self.registry.ffmpeg();
        let ffprobe = self.registry.ffprobe();

        let ffmpeg_info = self.detect_single_tool(ffmpeg, dir);
        let ffprobe_info = self.detect_single_tool(ffprobe, dir);

        let all_installed = ffmpeg_info.installed && ffprobe_info.installed;

        SuiteInfo {
            name: "FFmpeg".to_string(),
            dir_path: dir.map(|p| p.to_string_lossy().to_string()),
            tools: vec![ffmpeg_info, ffprobe_info],
            all_installed,
        }
    }

    /// 检测单个工具
    fn detect_single_tool(
        &self,
        config: &dyn ToolDefinition,
        configured_dir: Option<&PathBuf>,
    ) -> ToolInfo {
        let tool_name = config.name();
        let exe_names = config.exe_names();

        // 1. 优先从配置目录查找
        if let Some(dir) = configured_dir {
            if let Some(exe_path) = exe_names.find_in_dir(dir) {
                return self.check_tool(config, exe_path, Some(dir.clone()));
            }
        }

        // 2. 从系统 PATH 查找
        if let Ok(exe_path) = which::which(exe_names.main_exe()) {
            let dir = exe_path.parent().map(PathBuf::from);
            return self.check_tool(config, exe_path, dir);
        }

        // 3. 未找到
        ToolInfo {
            name: tool_name.to_string(),
            installed: false,
            version: None,
            exe_path: None,
            dir_path: configured_dir.map(|p| p.to_string_lossy().to_string()),
            error: Some("未找到，请配置目录或下载".to_string()),
        }
    }

    /// 检查工具版本
    fn check_tool(
        &self,
        config: &dyn ToolDefinition,
        exe_path: PathBuf,
        dir_path: Option<PathBuf>,
    ) -> ToolInfo {
        let tool_name = config.name();
        let exe_str = exe_path.to_string_lossy().to_string();

        log::info!("[Detector] 检查工具: {}, 路径: {}", tool_name, exe_str);

        // Windows 平台：隐藏控制台窗口
        #[cfg(target_os = "windows")]
        let output = Command::new(&exe_path)
            .args(config.version_args())
            .creation_flags(CREATE_NO_WINDOW)
            .output();

        #[cfg(not(target_os = "windows"))]
        let output = Command::new(&exe_path).args(config.version_args()).output();

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                log::info!(
                    "[Detector] {} stdout: {}",
                    tool_name,
                    stdout.lines().next().unwrap_or("")
                );
                log::debug!(
                    "[Detector] {} stderr: {}",
                    tool_name,
                    stderr.lines().next().unwrap_or("")
                );

                let version = config.parse_version(&stdout, &stderr);
                log::info!("[Detector] {} 解析版本: {:?}", tool_name, version);

                ToolInfo {
                    name: tool_name.to_string(),
                    installed: true,
                    version,
                    exe_path: Some(exe_str),
                    dir_path: dir_path.map(|p| p.to_string_lossy().to_string()),
                    error: None,
                }
            }
            Err(e) => {
                log::warn!("[Detector] {} 执行失败: {}", tool_name, e);
                ToolInfo {
                    name: tool_name.to_string(),
                    installed: false,
                    version: None,
                    exe_path: Some(exe_str),
                    dir_path: dir_path.map(|p| p.to_string_lossy().to_string()),
                    error: Some(format!("执行失败: {}", e)),
                }
            }
        }
    }
}

// ========================================
// 便捷函数：获取可执行文件路径
// ========================================

/// 获取 N_m3u8DL-RE 可执行文件路径
///
/// 优先使用配置目录，其次从系统 PATH 查找
pub fn get_downloader_exe_path(configured_dir: Option<&str>) -> Option<PathBuf> {
    let registry = ToolRegistry::global();
    let exe_names = registry.downloader().exe_names();

    // 1. 从配置目录查找
    if let Some(dir) = configured_dir {
        let dir_path = PathBuf::from(dir);
        if let Some(exe) = exe_names.find_in_dir(&dir_path) {
            return Some(exe);
        }
    }

    // 2. 从系统 PATH 查找
    which::which(exe_names.main_exe()).ok()
}

/// 获取 FFmpeg 可执行文件路径
pub fn get_ffmpeg_exe_path(configured_dir: Option<&str>) -> Option<PathBuf> {
    let registry = ToolRegistry::global();
    let exe_names = registry.ffmpeg().exe_names();

    if let Some(dir) = configured_dir {
        let dir_path = PathBuf::from(dir);
        if let Some(exe) = exe_names.find_in_dir(&dir_path) {
            return Some(exe);
        }
    }

    which::which(exe_names.main_exe()).ok()
}

/// 获取 FFprobe 可执行文件路径
pub fn get_ffprobe_exe_path(configured_dir: Option<&str>) -> Option<PathBuf> {
    let registry = ToolRegistry::global();
    let exe_names = registry.ffprobe().exe_names();

    if let Some(dir) = configured_dir {
        let dir_path = PathBuf::from(dir);
        if let Some(exe) = exe_names.find_in_dir(&dir_path) {
            return Some(exe);
        }
    }

    which::which(exe_names.main_exe()).ok()
}
