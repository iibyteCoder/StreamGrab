//! 已验证路径类型
//!
//! 根除"空/相对/不存在路径"全族 bug 的编译期保障。

use std::path::{Path, PathBuf};

use crate::shared::{AppError, AppResult};

/// 已验证的绝对路径（非空 + 绝对 + 存在）
///
/// 构造时强制校验三项不变量，下游代码无需重复检查。
/// 命令层构造一次，往下传递即可。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResolvedPath(PathBuf);

impl ResolvedPath {
    /// 从用户输入字符串构造
    ///
    /// 空串、相对路径、不存在的路径均返回错误。
    pub fn new(raw: &str) -> AppResult<Self> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(AppError::config("路径不能为空"));
        }
        let path = PathBuf::from(trimmed);
        Self::from_path(path)
    }

    /// 从 PathBuf 构造（校验绝对 + 存在）
    pub fn from_path(path: PathBuf) -> AppResult<Self> {
        if !path.is_absolute() {
            return Err(AppError::config(format!(
                "路径必须为绝对路径: {}",
                path.display()
            )));
        }
        if !path.exists() {
            return Err(AppError::config(format!("路径不存在: {}", path.display())));
        }
        Ok(Self(path))
    }

    /// 尝试构造，失败返回 None（用于可选路径，如 ffmpeg_bin 回退到 PATH 搜索）
    pub fn try_new(raw: &str) -> Option<Self> {
        Self::new(raw).ok()
    }

    /// 尝试从 PathBuf 构造，失败返回 None
    pub fn try_from_path(path: PathBuf) -> Option<Self> {
        Self::from_path(path).ok()
    }

    /// 从可能为空的 Option<&str> 构造（None/空串 → None）
    pub fn from_optional(raw: Option<&str>) -> Option<Self> {
        raw.filter(|s| !s.trim().is_empty()).and_then(Self::try_new)
    }

    /// 获取内部 Path 引用
    pub fn as_path(&self) -> &Path {
        &self.0
    }

    /// 获取内部 PathBuf 引用
    pub fn as_path_buf(&self) -> &PathBuf {
        &self.0
    }

    /// 转为 lossy 字符串（Windows 路径可能含非 UTF-8）
    pub fn to_string_lossy(&self) -> String {
        self.0.to_string_lossy().into_owned()
    }

    /// 消费自身，返回内部 PathBuf
    pub fn into_inner(self) -> PathBuf {
        self.0
    }
}

impl AsRef<Path> for ResolvedPath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl std::fmt::Display for ResolvedPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.display())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_string() {
        assert!(ResolvedPath::new("").is_err());
        assert!(ResolvedPath::new("   ").is_err());
    }

    #[test]
    fn rejects_relative_path() {
        assert!(ResolvedPath::new("relative/path").is_err());
        assert!(ResolvedPath::new("./foo").is_err());
    }

    #[test]
    fn rejects_nonexistent_absolute_path() {
        assert!(ResolvedPath::new("/nonexistent/path/xyz123").is_err());
    }

    #[test]
    fn accepts_existing_absolute_path() {
        let dir = tempfile::tempdir().unwrap();
        let resolved = ResolvedPath::from_path(dir.path().to_path_buf()).unwrap();
        assert_eq!(resolved.as_path(), dir.path());
    }

    #[test]
    fn try_new_returns_none_on_failure() {
        assert_eq!(ResolvedPath::try_new(""), None);
        assert_eq!(ResolvedPath::try_new("relative"), None);
    }

    #[test]
    fn from_optional_handles_none_and_empty() {
        assert_eq!(ResolvedPath::from_optional(None), None);
        assert_eq!(ResolvedPath::from_optional(Some("")), None);
        assert_eq!(ResolvedPath::from_optional(Some("  ")), None);
    }

    #[test]
    fn from_optional_resolves_valid_path() {
        let dir = tempfile::tempdir().unwrap();
        let path_str = dir.path().to_str().unwrap();
        let resolved = ResolvedPath::from_optional(Some(path_str));
        assert!(resolved.is_some());
        assert_eq!(resolved.unwrap().as_path(), dir.path());
    }

    #[test]
    fn display_shows_path() {
        let dir = tempfile::tempdir().unwrap();
        let resolved = ResolvedPath::from_path(dir.path().to_path_buf()).unwrap();
        assert_eq!(resolved.to_string(), dir.path().display().to_string());
    }
}
