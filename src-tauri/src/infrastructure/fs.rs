//! 文件系统工具
//!
//! 输出文件查找与文件信息查询

use std::path::{Path, PathBuf};

use crate::domain::config::MuxFormat;
use crate::shared::{AppError, AppResult};

/// 媒体文件扩展名（含字幕）
const MEDIA_EXTENSIONS: [&str; 20] = [
    "mp4", "mkv", "ts", "m4a", "m4v", "webm", "avi", "mov", "flv", "wmv", "m2ts", "vob", "mp3",
    "aac", "ogg", "flac", "wav", "srt", "vtt", "ass",
];

/// 按混流格式返回可能的输出扩展名
fn possible_extensions(mux_format: Option<MuxFormat>) -> Vec<&'static str> {
    match mux_format {
        Some(MuxFormat::Mp4) => vec!["mp4"],
        Some(MuxFormat::Mkv) => vec!["mkv"],
        None => vec!["mp4", "mkv", "ts", "m4a", "m4v", "webm"],
    }
}

/// 文件是否是媒体文件
pub fn is_media_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| MEDIA_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

/// 查找实际生成的输出文件
///
/// 下载器可能生成与预期略有不同的文件名（追加扩展名、冲突重命名等），
/// 依次尝试：精确匹配 → 加扩展名 → 前缀匹配（取最新）→ 目录最新媒体文件 → 预期路径兜底
pub fn find_output_file(
    save_dir: &str,
    save_name: Option<&str>,
    mux_format: Option<MuxFormat>,
) -> Option<String> {
    let dir = PathBuf::from(save_dir);
    if !dir.exists() {
        log::warn!("Save directory does not exist: {save_dir}");
        return None;
    }

    let extensions = possible_extensions(mux_format);

    if let Some(name) = save_name {
        // 1. 精确匹配
        let exact = dir.join(name);
        if exact.exists() && is_media_file(&exact) {
            return exact.to_str().map(String::from);
        }

        // 2. 加扩展名匹配
        for ext in &extensions {
            let path = dir.join(format!("{name}.{ext}"));
            if path.exists() {
                return path.to_str().map(String::from);
            }
        }

        // 3. 前缀匹配（取修改时间最新）
        if let Some(latest) = latest_media_file_with_prefix(&dir, name) {
            return Some(latest);
        }
    }

    // 4. 目录中最新的媒体文件
    if let Some(latest) = latest_media_file(&dir) {
        return Some(latest);
    }

    log::warn!("Could not find output file in directory: {save_dir}");
    // 5. 兜底：返回预期路径
    save_name.map(|name| {
        let ext = extensions.first().unwrap_or(&"mp4");
        dir.join(format!("{name}.{ext}"))
            .to_string_lossy()
            .to_string()
    })
}

/// 目录内指定前缀的最新媒体文件
fn latest_media_file_with_prefix(dir: &Path, prefix: &str) -> Option<String> {
    let mut candidates: Vec<_> = std::fs::read_dir(dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().starts_with(prefix) && is_media_file(&e.path()))
        .collect();
    sort_by_modified_desc(&mut candidates);
    candidates
        .first()
        .and_then(|e| e.path().to_str().map(String::from))
}

/// 目录内最新媒体文件
fn latest_media_file(dir: &Path) -> Option<String> {
    let mut candidates: Vec<_> = std::fs::read_dir(dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| is_media_file(&e.path()))
        .collect();
    sort_by_modified_desc(&mut candidates);
    candidates
        .first()
        .and_then(|e| e.path().to_str().map(String::from))
}

fn sort_by_modified_desc(entries: &mut [std::fs::DirEntry]) {
    entries.sort_by(|a, b| {
        let ta = a.metadata().ok().and_then(|m| m.modified().ok());
        let tb = b.metadata().ok().and_then(|m| m.modified().ok());
        tb.cmp(&ta)
    });
}

/// 文件信息
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileInfo {
    /// 文件完整路径
    pub path: String,
    /// 文件名
    pub file_name: String,
    /// 扩展名
    pub extension: String,
    /// 大小（字节）
    pub size: u64,
    /// 修改时间（Unix 毫秒）
    pub modified: Option<i64>,
    /// 是否存在
    pub exists: bool,
}

/// 查询文件信息
pub fn file_info(path: &str) -> AppResult<FileInfo> {
    let path_buf = PathBuf::from(path);
    if !path_buf.exists() {
        return Err(AppError::other(format!("文件不存在: {path}")));
    }

    let metadata = path_buf
        .metadata()
        .map_err(|e| AppError::other(format!("获取文件信息失败: {e}")))?;

    Ok(FileInfo {
        path: path_buf.to_string_lossy().to_string(),
        file_name: path_buf
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string(),
        extension: path_buf
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_string(),
        size: metadata.len(),
        modified: metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as i64),
        exists: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn media_file_detection() {
        assert!(is_media_file(Path::new("a/video.mp4")));
        assert!(is_media_file(Path::new("a/video.MKV")));
        assert!(is_media_file(Path::new("a/sub.srt")));
        assert!(!is_media_file(Path::new("a/readme.txt")));
        assert!(!is_media_file(Path::new("a/noext")));
    }

    #[test]
    fn finds_file_with_extension_added() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("episode.mp4"), b"x").unwrap();

        let found = find_output_file(
            dir.path().to_str().unwrap(),
            Some("episode"),
            Some(MuxFormat::Mp4),
        );
        assert_eq!(
            found,
            dir.path().join("episode.mp4").to_str().map(String::from)
        );
    }

    #[test]
    fn exact_match_wins() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("movie.mkv"), b"x").unwrap();

        let found = find_output_file(dir.path().to_str().unwrap(), Some("movie.mkv"), None);
        assert_eq!(
            found,
            dir.path().join("movie.mkv").to_str().map(String::from)
        );
    }

    #[test]
    fn prefix_match_takes_latest() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("clip_conflict.mp4"), b"x").unwrap();

        let found = find_output_file(
            dir.path().to_str().unwrap(),
            Some("clip"),
            Some(MuxFormat::Mp4),
        );
        assert_eq!(
            found,
            dir.path()
                .join("clip_conflict.mp4")
                .to_str()
                .map(String::from)
        );
    }

    #[test]
    fn fallback_to_expected_path() {
        let dir = tempfile::tempdir().unwrap();
        let found = find_output_file(
            dir.path().to_str().unwrap(),
            Some("ghost"),
            Some(MuxFormat::Mkv),
        );
        assert_eq!(
            found,
            dir.path().join("ghost.mkv").to_str().map(String::from)
        );
    }

    #[test]
    fn missing_dir_returns_none() {
        assert_eq!(
            find_output_file("/nonexistent/dir/xyz", Some("a"), None),
            None
        );
    }

    #[test]
    fn file_info_reads_metadata() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("sample.mp4");
        fs::write(&file, b"0123456789").unwrap();

        let info = file_info(file.to_str().unwrap()).unwrap();
        assert_eq!(info.file_name, "sample.mp4");
        assert_eq!(info.extension, "mp4");
        assert_eq!(info.size, 10);
        assert!(info.exists);
        assert!(info.modified.is_some());

        assert!(file_info("/nonexistent/file.mp4").is_err());
    }
}
