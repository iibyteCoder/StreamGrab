//! 工具管理模块
//!
//! 提供外部工具的统一管理接口，支持多平台和多工具扩展
//!
//! ## 核心概念
//!
//! - **目录路径 (dirPath)**: 用户配置的目录，包含可执行文件
//! - **可执行文件路径 (exePath)**: 检测到的实际可执行文件完整路径
//!
//! ## 使用流程
//!
//! 1. 用户配置目录路径（或通过下载自动设置）
//! 2. 调用检测 API 验证工具是否可用
//! 3. 实际使用时通过 `get_xxx_exe_path` 获取可执行文件路径

mod config;
mod detector;

pub use config::*;
pub use detector::*;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::platform::{ExeNames, Platform};

/// 工具名称常量
pub mod tool_names {
    /// N_m3u8DL-RE 下载器
    pub const DOWNLOADER: &str = "N_m3u8DL-RE";
    /// FFmpeg 主程序
    pub const FFMPEG: &str = "FFmpeg";
    /// FFprobe 工具
    pub const FFPROBE: &str = "FFprobe";
}

/// 工具信息（API 返回格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolInfo {
    /// 工具名称
    pub name: String,
    /// 是否已安装（可执行文件存在且可运行）
    pub installed: bool,
    /// 版本号
    pub version: Option<String>,
    /// 可执行文件完整路径（检测后获得）
    pub exe_path: Option<String>,
    /// 配置的目录路径（用户设置）
    pub dir_path: Option<String>,
    /// 错误信息（未安装时）
    pub error: Option<String>,
}

/// 工具套件信息（包含多个相关工具，如 FFmpeg + FFprobe）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiteInfo {
    /// 套件名称
    pub name: String,
    /// 配置的目录路径
    pub dir_path: Option<String>,
    /// 包含的工具列表
    pub tools: Vec<ToolInfo>,
    /// 是否全部已安装
    pub all_installed: bool,
}

/// 工具定义 Trait
///
/// 每个工具需要实现此 trait 来定义其行为特性
pub trait ToolDefinition: Send + Sync {
    /// 工具显示名称
    fn name(&self) -> &'static str;

    /// 可执行文件名配置（支持主程序和别名）
    fn exe_names(&self) -> ExeNames;

    /// 获取版本号的命令行参数
    fn version_args(&self) -> &'static [&'static str];

    /// 从命令输出解析版本号
    fn parse_version(&self, stdout: &str, stderr: &str) -> Option<String>;

    /// GitHub 仓库（格式: "owner/repo"，用于检查更新和下载）
    fn github_repo(&self) -> Option<&'static str> {
        None
    }

    /// 在 GitHub releases 中查找适合指定平台的资源
    /// 返回 (下载URL, 文件名)
    fn find_release_asset(
        &self,
        assets: &[serde_json::Value],
        platform: Platform,
    ) -> Option<(String, String)> {
        let _ = (assets, platform);
        None
    }
}

/// 工具路径配置
///
/// 存储用户配置的工具目录路径
#[derive(Debug, Clone, Default)]
pub struct ToolPaths {
    /// N_m3u8DL-RE 目录路径
    pub downloader_dir: Option<PathBuf>,
    /// FFmpeg 套件目录路径（ffmpeg 和 ffprobe 应在同一目录）
    pub ffmpeg_dir: Option<PathBuf>,
}

impl ToolPaths {
    /// 从配置创建路径实例
    pub fn new(downloader_dir: Option<&str>, ffmpeg_dir: Option<&str>) -> Self {
        Self {
            downloader_dir: downloader_dir.map(PathBuf::from),
            ffmpeg_dir: ffmpeg_dir.map(PathBuf::from),
        }
    }
}
