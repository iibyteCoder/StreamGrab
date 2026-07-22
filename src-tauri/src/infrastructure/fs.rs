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

/// 计算字节的 SHA-256 哈希（小写十六进制）
pub fn compute_sha256(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let hash = Sha256::digest(data);
    hash.iter().map(|b| format!("{b:02x}")).collect()
}

/// 从 `.sha256` 文件内容中解析目标文件的期望哈希
///
/// 支持两种常见格式：
/// - `<hash>  <filename>`（sha256sum 输出）
/// - 仅 `<hash>`（单行纯哈希）
pub fn parse_sha256_content(content: &str, filename: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        // 格式: "<hash>  <filename>" 或 "<hash> <filename>"
        if let Some((hash, name)) = line.split_once(|c: char| c.is_whitespace()) {
            let name = name.trim_start_matches('*').trim();
            if name == filename || name.ends_with(filename) {
                return Some(hash.to_lowercase());
            }
        } else {
            // 单行纯哈希（无文件名）
            let candidate = line.to_lowercase();
            if candidate.len() == 64 && candidate.chars().all(|c| c.is_ascii_hexdigit()) {
                return Some(candidate);
            }
        }
    }
    None
}

/// 验证下载内容的 SHA-256 完整性
///
/// 尝试从 `{download_url}.sha256` 获取校验文件：
/// - 获取成功且哈希不匹配 → 返回 Err（供应链投毒风险）
/// - 获取成功且哈希匹配 → Ok
/// - 获取失败（404/网络错误）→ 日志警告，Ok（第三方 release 可能不提供）
pub async fn verify_download_integrity(
    client: &reqwest::Client,
    download_url: &str,
    filename: &str,
    data: &[u8],
) -> Result<(), String> {
    let sha256_url = format!("{download_url}.sha256");
    let response = client.get(&sha256_url).send().await;

    let response = match response {
        Ok(r) if r.status().is_success() => r,
        Ok(r) => {
            log::warn!(
                "[Integrity] 校验文件不可用 (HTTP {}): {sha256_url}，跳过完整性验证",
                r.status()
            );
            return Ok(());
        }
        Err(e) => {
            log::warn!("[Integrity] 无法获取校验文件: {sha256_url} ({e})，跳过完整性验证");
            return Ok(());
        }
    };

    let content = response
        .text()
        .await
        .map_err(|e| format!("读取校验文件失败: {e}"))?;

    let expected = parse_sha256_content(&content, filename)
        .ok_or_else(|| format!("校验文件中未找到 {filename} 的哈希值"))?;

    let actual = compute_sha256(data);
    if actual != expected {
        return Err(format!(
            "SHA-256 校验失败！文件可能被篡改。\n期望: {expected}\n实际: {actual}"
        ));
    }

    log::info!("[Integrity] SHA-256 校验通过: {filename} ({actual})");
    Ok(())
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
    fn sha256_computation_is_correct() {
        // SHA-256 of empty string
        assert_eq!(
            compute_sha256(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        // SHA-256 of "hello"
        assert_eq!(
            compute_sha256(b"hello"),
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn parse_sha256_sum_format() {
        let content = "abc123  myfile.zip\n";
        assert_eq!(
            parse_sha256_content(content, "myfile.zip"),
            Some("abc123".to_string())
        );
    }

    #[test]
    fn parse_sha256_bare_hash() {
        let hash = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let content = format!("{hash}\n");
        assert_eq!(
            parse_sha256_content(&content, "anything.zip"),
            Some(hash.to_string())
        );
    }

    #[test]
    fn parse_sha256_multiple_files() {
        let content = "aaa111  first.zip\nbbb222  second.zip\n";
        assert_eq!(
            parse_sha256_content(content, "second.zip"),
            Some("bbb222".to_string())
        );
        assert_eq!(parse_sha256_content(content, "third.zip"), None);
    }

    #[test]
    fn parse_sha256_case_insensitive() {
        let content =
            "ABCDEF1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF1234567890  tool.zip\n";
        assert_eq!(
            parse_sha256_content(content, "tool.zip"),
            Some("abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string())
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
