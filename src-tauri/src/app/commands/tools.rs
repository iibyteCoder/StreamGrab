//! 工具管理命令
//!
//! 外部工具（N_m3u8DL-RE、FFmpeg）的检测、版本查询与下载安装。
//! 检测逻辑复用 `infrastructure::tools`（ToolRegistry + ToolDetector）。

use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

use crate::infrastructure::platform::Platform;
use crate::infrastructure::tools::{ToolDetector, ToolInfo, ToolPaths, ToolRegistry};

// ========================================
// 版本信息缓存（5 分钟，规避 GitHub API 限流）
// ========================================

type ReleaseCacheData = (Option<ToolReleaseInfo>, Option<ToolReleaseInfo>);

struct CacheEntry {
    data: ReleaseCacheData,
    timestamp: Instant,
}

static RELEASE_CACHE: OnceLock<Mutex<CacheEntry>> = OnceLock::new();
const CACHE_TTL: Duration = Duration::from_secs(5 * 60);

fn cache() -> &'static Mutex<CacheEntry> {
    RELEASE_CACHE.get_or_init(|| {
        Mutex::new(CacheEntry {
            data: (None, None),
            timestamp: Instant::now() - CACHE_TTL - Duration::from_secs(1),
        })
    })
}

// ========================================
// API 数据结构
// ========================================

/// 工具下载进度
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    pub tool: String,
    pub status: String,
    pub downloaded: u64,
    pub total: u64,
    pub percent: f64,
}

/// 工具发布信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolReleaseInfo {
    pub version: String,
    pub download_url: String,
    pub filename: String,
    pub published_at: String,
}

// ========================================
// 工具检测命令
// ========================================

/// 获取 N_m3u8DL-RE 工具信息（真实版本检测）
#[tauri::command(rename_all = "camelCase")]
pub async fn get_nm3u8dl_info(path: Option<String>) -> Result<ToolInfo, String> {
    let detector = ToolDetector::new(ToolPaths::new(path.as_deref(), None));
    Ok(detector.detect_downloader())
}

/// 获取 FFmpeg 工具信息
#[tauri::command(rename_all = "camelCase")]
pub async fn get_ffmpeg_info(path: Option<String>) -> Result<ToolInfo, String> {
    let detector = ToolDetector::new(ToolPaths::new(None, path.as_deref()));
    let suite = detector.detect_ffmpeg_suite();
    Ok(suite
        .tools
        .into_iter()
        .find(|t| t.name == crate::infrastructure::tools::tool_names::FFMPEG)
        .unwrap_or_else(|| ToolInfo {
            name: "FFmpeg".into(),
            installed: false,
            version: None,
            exe_path: None,
            dir_path: path,
            error: Some("未找到".into()),
        }))
}

/// 获取 FFprobe 工具信息
#[tauri::command(rename_all = "camelCase")]
pub async fn get_ffprobe_info(ffmpeg_path: Option<String>) -> Result<ToolInfo, String> {
    let detector = ToolDetector::new(ToolPaths::new(None, ffmpeg_path.as_deref()));
    let suite = detector.detect_ffmpeg_suite();
    Ok(suite
        .tools
        .into_iter()
        .find(|t| t.name == crate::infrastructure::tools::tool_names::FFPROBE)
        .unwrap_or_else(|| ToolInfo {
            name: "FFprobe".into(),
            installed: false,
            version: None,
            exe_path: None,
            dir_path: ffmpeg_path,
            error: Some("未找到".into()),
        }))
}

// ========================================
// GitHub 版本与下载
// ========================================

/// 获取 N_m3u8DL-RE 最新版本信息
#[tauri::command(rename_all = "camelCase")]
pub async fn get_nm3u8dl_latest_release() -> Result<ToolReleaseInfo, String> {
    if let Some(info) = cached_release(0) {
        return Ok(info);
    }
    let registry = ToolRegistry::global();
    let config = registry.downloader();
    let info = fetch_release(config).await?;
    store_cached_release(0, info.clone());
    Ok(info)
}

/// 获取 FFmpeg 最新版本信息
#[tauri::command(rename_all = "camelCase")]
pub async fn get_ffmpeg_latest_release() -> Result<ToolReleaseInfo, String> {
    if let Some(info) = cached_release(1) {
        return Ok(info);
    }
    let registry = ToolRegistry::global();
    let config = registry.ffmpeg();
    let info = fetch_release(config).await?;
    store_cached_release(1, info.clone());
    Ok(info)
}

fn cached_release(slot: usize) -> Option<ToolReleaseInfo> {
    let cache = cache().lock().ok()?;
    if cache.timestamp.elapsed() >= CACHE_TTL {
        return None;
    }
    match slot {
        0 => cache.data.0.clone(),
        _ => cache.data.1.clone(),
    }
}

fn store_cached_release(slot: usize, info: ToolReleaseInfo) {
    if let Ok(mut cache) = cache().lock() {
        match slot {
            0 => cache.data.0 = Some(info),
            _ => cache.data.1 = Some(info),
        }
        cache.timestamp = Instant::now();
    }
}

/// 从 GitHub 获取最新 release（经 ToolDefinition 选择平台资产）
async fn fetch_release(
    config: &'static dyn crate::infrastructure::tools::ToolDefinition,
) -> Result<ToolReleaseInfo, String> {
    let github_repo = config.github_repo().ok_or("未配置 GitHub 仓库")?;
    let url = format!("https://api.github.com/repos/{github_repo}/releases/latest");
    log::info!("[Tools] 获取最新版本: {url}");

    let client = reqwest::Client::builder()
        .user_agent("StreamGrab")
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {e}"))?;

    let response = client
        .get(&url)
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .map_err(|e| format!("请求 GitHub API 失败: {e}"))?;

    if response.status() == 403 {
        return Err("GitHub API 请求频率限制，请稍后重试".to_string());
    }
    if !response.status().is_success() {
        return Err(format!("GitHub API 返回错误: {}", response.status()));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {e}"))?;

    let tag_name = json["tag_name"].as_str().unwrap_or("unknown");
    let release_name = json["name"].as_str().unwrap_or("");
    let published_at = json["published_at"].as_str().unwrap_or("").to_string();
    let assets = json["assets"].as_array().cloned().unwrap_or_default();

    // tag 为 "latest"（如 FFmpeg-Builds 自动构建）时从名称提取日期作为版本标识
    let version = if tag_name == "latest" && !release_name.is_empty() {
        regex::Regex::new(r"\((\d{4}-\d{2}-\d{2})")
            .ok()
            .and_then(|re| re.captures(release_name))
            .map(|cap| format!("latest-{}", cap.get(1).map_or("unknown", |m| m.as_str())))
            .unwrap_or_else(|| release_name.to_string())
    } else {
        tag_name.to_string()
    };

    let (download_url, filename) = config.find_release_asset(&assets).ok_or_else(|| {
        format!(
            "未找到适合 {} 的下载链接",
            Platform::current().arch_keywords()
        )
    })?;

    log::info!("[Tools] 版本: {version}, URL: {download_url}");
    Ok(ToolReleaseInfo {
        version,
        download_url,
        filename,
        published_at,
    })
}

/// 下载工具（ZIP 整包下载 → 完整性校验 → 解压 → 返回可执行文件目录）
#[tauri::command(rename_all = "camelCase")]
pub async fn download_tool(
    tool: String,
    download_url: String,
    target_dir: String,
    app: AppHandle,
) -> Result<String, String> {
    let target_path = PathBuf::from(&target_dir);
    log::info!("[Tools] 开始下载 {tool} 到 {target_dir}");

    if !target_path.exists() {
        std::fs::create_dir_all(&target_path).map_err(|e| format!("创建目录失败: {e}"))?;
    }

    let _ = app.emit(
        &format!("tool:download:start:{tool}"),
        &serde_json::json!({ "url": &download_url }),
    );

    let client = reqwest::Client::builder()
        .user_agent("StreamGrab-Downloader")
        .timeout(Duration::from_secs(300))
        .connect_timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {e}"))?;

    let response = client
        .get(&download_url)
        .send()
        .await
        .map_err(|e| format!("下载请求失败: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("下载失败: HTTP {}", response.status()));
    }

    let total_size = response.content_length().unwrap_or(0);
    let filename = download_url.rsplit('/').next().unwrap_or("download.zip");
    let zip_path = target_path.join(filename);

    // 整包下载到内存后落盘（规避流式中断问题）
    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("下载文件内容失败: {e}"))?;
    let actual_size = bytes.len() as u64;

    if total_size > 0 && actual_size != total_size {
        return Err(format!(
            "下载不完整: 期望 {total_size} bytes, 实际 {actual_size} bytes"
        ));
    }
    if bytes.len() < 4 || &bytes[0..2] != b"PK" {
        return Err("下载的文件不是有效的 ZIP 格式".to_string());
    }

    let _ = app.emit(
        &format!("tool:download:progress:{tool}"),
        &DownloadProgress {
            tool: tool.clone(),
            status: "downloaded".into(),
            downloaded: actual_size,
            total: total_size,
            percent: 100.0,
        },
    );

    use std::io::Write;
    let mut file = std::fs::File::create(&zip_path).map_err(|e| format!("创建文件失败: {e}"))?;
    file.write_all(&bytes)
        .map_err(|e| format!("写入文件失败: {e}"))?;
    file.sync_all().map_err(|e| format!("同步文件失败: {e}"))?;

    let _ = app.emit(
        &format!("tool:download:progress:{tool}"),
        &DownloadProgress {
            tool: tool.clone(),
            status: "extracting".into(),
            downloaded: total_size,
            total: total_size,
            percent: 100.0,
        },
    );

    let tool_dir = extract_zip(&zip_path, &target_path, &tool)?;
    let _ = std::fs::remove_file(&zip_path);

    let _ = app.emit(
        &format!("tool:download:complete:{tool}"),
        &serde_json::json!({ "path": &tool_dir }),
    );

    Ok(tool_dir)
}

/// 解压 ZIP，返回可执行文件所在目录
fn extract_zip(
    zip_path: &std::path::Path,
    target_dir: &std::path::Path,
    tool: &str,
) -> Result<String, String> {
    let file = std::fs::File::open(zip_path).map_err(|e| format!("打开 ZIP 失败: {e}"))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("读取 ZIP 失败: {e}"))?;

    let registry = ToolRegistry::global();
    let exe_names = if tool.to_lowercase().contains("ffmpeg") {
        registry.ffmpeg().exe_names().all_exe()
    } else {
        registry.downloader().exe_names().all_exe()
    };
    let exe_names_lower: Vec<String> = exe_names.iter().map(|s| s.to_lowercase()).collect();

    let mut found_exe_dir: Option<PathBuf> = None;
    let mut all_files: Vec<String> = Vec::new();

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("读取条目 {i} 失败: {e}"))?;

        let outpath = match file.enclosed_name() {
            Some(p) => target_dir.join(p),
            None => continue,
        };

        if file.name().ends_with('/') {
            std::fs::create_dir_all(&outpath).map_err(|e| format!("创建目录失败: {e}"))?;
            continue;
        }

        if let Some(filename) = outpath.file_name().and_then(|n| n.to_str()) {
            all_files.push(filename.to_string());
        }
        if let Some(parent) = outpath.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
            }
        }

        let mut outfile =
            std::fs::File::create(&outpath).map_err(|e| format!("创建文件失败: {e}"))?;
        std::io::copy(&mut file, &mut outfile).map_err(|e| format!("写入文件失败: {e}"))?;

        // 记录可执行文件所在目录（不区分大小写）
        if let Some(filename) = outpath.file_name().and_then(|n| n.to_str()) {
            if exe_names_lower.contains(&filename.to_lowercase()) && found_exe_dir.is_none() {
                found_exe_dir = outpath.parent().map(|p| p.to_path_buf());
            }
        }
    }

    found_exe_dir
        .map(|p| p.to_string_lossy().to_string())
        .ok_or_else(|| {
            format!("未找到可执行文件。期望: {exe_names:?}, ZIP 中实际文件: {all_files:?}")
        })
}
