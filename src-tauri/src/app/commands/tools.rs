//! 工具管理命令
//!
//! 提供外部工具（N_m3u8DL-RE、FFmpeg）的版本检测、下载和管理功能

use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use log::info;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

use crate::infrastructure::platform::Platform;
use crate::infrastructure::tools::{ToolDetector, ToolInfo, ToolPaths, ToolRegistry};

// ========================================
// 缓存机制
// ========================================

/// 缓存条目
struct CacheEntry<T> {
    data: T,
    timestamp: Instant,
}

/// 版本缓存数据类型
type ReleaseCacheData = (Option<ToolReleaseInfo>, Option<ToolReleaseInfo>);

/// 版本信息缓存（5 分钟有效期）
static RELEASE_CACHE: OnceLock<std::sync::Mutex<CacheEntry<ReleaseCacheData>>> = OnceLock::new();

/// 缓存有效期
const CACHE_TTL: Duration = Duration::from_secs(5 * 60);

fn get_cache() -> &'static std::sync::Mutex<CacheEntry<ReleaseCacheData>> {
    RELEASE_CACHE.get_or_init(|| {
        std::sync::Mutex::new(CacheEntry {
            data: (None, None),
            timestamp: Instant::now() - CACHE_TTL - Duration::from_secs(1),
        })
    })
}

fn is_cache_valid() -> bool {
    get_cache()
        .lock()
        .map(|c| c.timestamp.elapsed() < CACHE_TTL)
        .unwrap_or(false)
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
// Tauri 命令
// ========================================

/// 获取 N_m3u8DL-RE 工具信息
#[tauri::command(rename_all = "camelCase")]
pub async fn get_nm3u8dl_info(path: Option<String>) -> Result<ToolInfo, String> {
    let paths = ToolPaths::new(path.as_deref(), None);
    let detector = ToolDetector::new(paths);
    Ok(detector.detect_downloader())
}

/// 获取 FFmpeg 工具信息
#[tauri::command(rename_all = "camelCase")]
pub async fn get_ffmpeg_info(path: Option<String>) -> Result<ToolInfo, String> {
    let paths = ToolPaths::new(None, path.as_deref());
    let detector = ToolDetector::new(paths);
    let suite = detector.detect_ffmpeg_suite();

    // 返回 FFmpeg 主程序信息
    Ok(suite
        .tools
        .iter()
        .find(|t| t.name == crate::infrastructure::tools::tool_names::FFMPEG)
        .cloned()
        .unwrap_or_else(|| ToolInfo {
            name: "FFmpeg".to_string(),
            installed: false,
            version: None,
            exe_path: None,
            dir_path: path,
            error: Some("未找到".to_string()),
        }))
}

/// 获取 FFprobe 工具信息
#[tauri::command(rename_all = "camelCase")]
pub async fn get_ffprobe_info(ffmpeg_path: Option<String>) -> Result<ToolInfo, String> {
    let paths = ToolPaths::new(None, ffmpeg_path.as_deref());
    let detector = ToolDetector::new(paths);
    let suite = detector.detect_ffmpeg_suite();

    Ok(suite
        .tools
        .iter()
        .find(|t| t.name == crate::infrastructure::tools::tool_names::FFPROBE)
        .cloned()
        .unwrap_or_else(|| ToolInfo {
            name: "FFprobe".to_string(),
            installed: false,
            version: None,
            exe_path: None,
            dir_path: ffmpeg_path,
            error: Some("未找到".to_string()),
        }))
}

/// 获取 N_m3u8DL-RE 最新版本信息
#[tauri::command(rename_all = "camelCase")]
pub async fn get_nm3u8dl_latest_release() -> Result<ToolReleaseInfo, String> {
    // 检查缓存
    if is_cache_valid() {
        if let Some(ref info) = get_cache().lock().ok().and_then(|c| c.data.0.clone()) {
            info!("[Tools] 使用缓存的 N_m3u8DL-RE 版本信息");
            return Ok(info.clone());
        }
    }

    let registry = ToolRegistry::global();
    let config = registry.downloader();
    let github_repo = config.github_repo().ok_or("未配置 GitHub 仓库")?;

    let url = format!(
        "https://api.github.com/repos/{}/releases/latest",
        github_repo
    );
    info!("[Tools] 获取 N_m3u8DL-RE 最新版本: {}", url);

    let (version, published_at, download_url, filename) =
        fetch_github_release(&url, |a| config.find_release_asset(a)).await?;

    let info = ToolReleaseInfo {
        version: version.clone(),
        download_url: download_url.clone(),
        filename,
        published_at,
    };

    info!(
        "[Tools] N_m3u8DL-RE 版本: {}, URL: {}",
        version, download_url
    );

    if let Ok(mut cache) = get_cache().lock() {
        cache.data.0 = Some(info.clone());
        cache.timestamp = Instant::now();
    }

    Ok(info)
}

/// 获取 FFmpeg 最新版本信息
#[tauri::command(rename_all = "camelCase")]
pub async fn get_ffmpeg_latest_release() -> Result<ToolReleaseInfo, String> {
    // 检查缓存
    if is_cache_valid() {
        if let Some(ref info) = get_cache().lock().ok().and_then(|c| c.data.1.clone()) {
            info!("[Tools] 使用缓存的 FFmpeg 版本信息");
            return Ok(info.clone());
        }
    }

    let registry = ToolRegistry::global();
    let config = registry.ffmpeg();
    let github_repo = config.github_repo().ok_or("未配置 GitHub 仓库")?;

    let url = format!(
        "https://api.github.com/repos/{}/releases/latest",
        github_repo
    );
    info!("[Tools] 获取 FFmpeg 最新版本: {}", url);

    let (version, published_at, download_url, filename) =
        fetch_github_release(&url, |a| config.find_release_asset(a)).await?;

    let info = ToolReleaseInfo {
        version: version.clone(),
        download_url: download_url.clone(),
        filename,
        published_at,
    };

    info!("[Tools] FFmpeg 版本: {}, URL: {}", version, download_url);

    if let Ok(mut cache) = get_cache().lock() {
        cache.data.1 = Some(info.clone());
        cache.timestamp = Instant::now();
    }

    Ok(info)
}

/// 从 GitHub 获取最新版本
async fn fetch_github_release<F>(
    url: &str,
    find_asset: F,
) -> Result<(String, String, String, String), String>
where
    F: FnOnce(&[serde_json::Value]) -> Option<(String, String)>,
{
    let client = reqwest::Client::builder()
        .user_agent("StreamGrab/0.5.0")
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let response = client
        .get(url)
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .map_err(|e| format!("请求 GitHub API 失败: {}", e))?;

    info!("[Tools] 响应状态: {}", response.status());

    if response.status() == 403 {
        return Err("GitHub API 请求频率限制，请稍后重试".to_string());
    }

    if !response.status().is_success() {
        return Err(format!("GitHub API 返回错误: {}", response.status()));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    let tag_name = json["tag_name"].as_str().unwrap_or("unknown");
    let release_name = json["name"].as_str().unwrap_or("");
    let published_at = json["published_at"].as_str().unwrap_or("").to_string();
    let assets = json["assets"].as_array().cloned().unwrap_or_default();

    // 如果 tag_name 是 "latest"（如 FFmpeg-Builds），则从 name 字段提取版本信息
    // name 格式: "Latest Auto-Build (2026-02-19 13:07)"
    let version = if tag_name == "latest" && !release_name.is_empty() {
        // 尝试从 name 中提取日期作为版本标识
        let date_re = regex::Regex::new(r"\((\d{4}-\d{2}-\d{2})").ok();
        if let Some(re) = date_re {
            if let Some(cap) = re.captures(release_name) {
                format!("latest-{}", cap.get(1).map_or("unknown", |m| m.as_str()))
            } else {
                release_name.to_string()
            }
        } else {
            release_name.to_string()
        }
    } else {
        tag_name.to_string()
    };

    let (download_url, filename) = find_asset(&assets).ok_or_else(|| {
        format!(
            "未找到适合 {} 的下载链接",
            Platform::current().arch_keywords()
        )
    })?;

    Ok((version, published_at, download_url, filename))
}

/// 下载工具
///
/// @param tool 工具名称
/// @param download_url 下载链接
/// @param target_dir 目标目录
/// @return 解压后的目录路径
#[tauri::command(rename_all = "camelCase")]
pub async fn download_tool(
    tool: String,
    download_url: String,
    target_dir: String,
    app: AppHandle,
) -> Result<String, String> {
    let target_path = PathBuf::from(&target_dir);

    info!("[Tools] 开始下载 {} 到 {}", tool, target_dir);

    // 确保目标目录存在
    if !target_path.exists() {
        std::fs::create_dir_all(&target_path).map_err(|e| format!("创建目录失败: {}", e))?;
    }

    // 发送开始事件
    let _ = app.emit(
        &format!("tool:download:start:{}", tool),
        &serde_json::json!({ "url": &download_url }),
    );

    // 创建 HTTP 客户端（继承系统代理设置，增加超时时间）
    let client = reqwest::Client::builder()
        .user_agent("StreamGrab-Downloader")
        .timeout(Duration::from_secs(300)) // 5 分钟超时
        .connect_timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    info!("[Tools] 开始下载文件: {}", download_url);

    // 下载
    let response = client
        .get(&download_url)
        .send()
        .await
        .map_err(|e| format!("下载请求失败: {}", e))?;

    info!("[Tools] 下载响应状态: {}", response.status());

    if !response.status().is_success() {
        return Err(format!("下载失败: HTTP {}", response.status()));
    }

    // 获取响应内容长度用于进度显示
    let total_size = response.content_length().unwrap_or(0);
    let filename = download_url.rsplit('/').next().unwrap_or("download.zip");
    let zip_path = target_path.join(filename);

    info!(
        "[Tools] 文件大小: {} bytes, 保存到: {:?}",
        total_size, zip_path
    );

    // 下载整个文件到内存（避免流式下载可能的中断问题）
    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("下载文件内容失败: {}", e))?;

    let actual_size = bytes.len() as u64;
    info!("[Tools] 实际下载大小: {} bytes", actual_size);

    // 验证下载完整性
    if total_size > 0 && actual_size != total_size {
        return Err(format!(
            "下载不完整: 期望 {} bytes, 实际 {} bytes",
            total_size, actual_size
        ));
    }

    // 检查是否是有效的 ZIP 文件（ZIP 文件以 PK 开头）
    if bytes.len() < 4 {
        return Err("下载的文件太小，可能不是有效的 ZIP 文件".to_string());
    }
    if &bytes[0..2] != b"PK" {
        return Err("下载的文件不是有效的 ZIP 格式（缺少 PK 签名）".to_string());
    }

    // 发送下载完成事件
    let _ = app.emit(
        &format!("tool:download:progress:{}", tool),
        &DownloadProgress {
            tool: tool.clone(),
            status: "downloaded".to_string(),
            downloaded: actual_size,
            total: total_size,
            percent: 100.0,
        },
    );

    // 保存文件
    let mut file = std::fs::File::create(&zip_path).map_err(|e| format!("创建文件失败: {}", e))?;
    use std::io::Write;
    file.write_all(&bytes)
        .map_err(|e| format!("写入文件失败: {}", e))?;
    file.sync_all()
        .map_err(|e| format!("同步文件失败: {}", e))?;

    info!("[Tools] 文件保存完成，开始解压...");

    // 发送解压事件
    let _ = app.emit(
        &format!("tool:download:progress:{}", tool),
        &DownloadProgress {
            tool: tool.clone(),
            status: "extracting".to_string(),
            downloaded: total_size,
            total: total_size,
            percent: 100.0,
        },
    );

    // 解压
    let tool_dir = extract_zip(&zip_path, &target_path, &tool)?;
    info!("[Tools] 解压完成，工具目录: {}", tool_dir);

    // 清理 zip 文件
    let _ = std::fs::remove_file(&zip_path);

    // 发送完成事件
    let _ = app.emit(
        &format!("tool:download:complete:{}", tool),
        &serde_json::json!({ "path": &tool_dir }),
    );

    Ok(tool_dir)
}

/// 解压 ZIP 文件，返回可执行文件所在目录
fn extract_zip(
    zip_path: &std::path::Path,
    target_dir: &std::path::Path,
    tool: &str,
) -> Result<String, String> {
    info!("[Tools] 开始解压 ZIP 文件: {:?}", zip_path);

    let file = std::fs::File::open(zip_path).map_err(|e| format!("打开 ZIP 失败: {}", e))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("读取 ZIP 失败: {}", e))?;

    let registry = ToolRegistry::global();
    let exe_names = if tool.to_lowercase().contains("ffmpeg") {
        registry.ffmpeg().exe_names().all_exe()
    } else {
        registry.downloader().exe_names().all_exe()
    };

    info!("[Tools] 期望的可执行文件名: {:?}", exe_names);
    info!("[Tools] ZIP 包含 {} 个文件", archive.len());

    // 创建小写版本用于不区分大小写匹配
    let exe_names_lower: Vec<String> = exe_names.iter().map(|s| s.to_lowercase()).collect();

    let mut found_exe_dir: Option<PathBuf> = None;
    let mut all_files: Vec<String> = Vec::new();

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("读取条目 {} 失败: {}", i, e))?;

        let outpath = match file.enclosed_name() {
            Some(p) => target_dir.join(p),
            None => continue,
        };

        if file.name().ends_with('/') {
            std::fs::create_dir_all(&outpath).map_err(|e| format!("创建目录失败: {}", e))?;
        } else {
            // 记录所有文件名
            if let Some(filename) = outpath.file_name().and_then(|n| n.to_str()) {
                all_files.push(filename.to_string());
            }

            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(p).map_err(|e| format!("创建目录失败: {}", e))?;
                }
            }

            let mut outfile =
                std::fs::File::create(&outpath).map_err(|e| format!("创建文件失败: {}", e))?;
            std::io::copy(&mut file, &mut outfile).map_err(|e| format!("写入文件失败: {}", e))?;

            // 检查是否是目标可执行文件（不区分大小写）
            if let Some(filename) = outpath.file_name().and_then(|n| n.to_str()) {
                let filename_lower = filename.to_lowercase();
                if exe_names_lower.contains(&filename_lower) && found_exe_dir.is_none() {
                    found_exe_dir = outpath.parent().map(|p| p.to_path_buf());
                    info!("[Tools] 找到可执行文件: {:?}", outpath);
                }
            }
        }
    }

    if found_exe_dir.is_none() {
        // 打印所有找到的文件，帮助调试
        info!("[Tools] ZIP 中的所有文件: {:?}", all_files);
        return Err(format!(
            "未找到可执行文件。期望: {:?}, ZIP 中实际文件: {:?}",
            exe_names, all_files
        ));
    }

    found_exe_dir
        .map(|p| p.to_string_lossy().to_string())
        .ok_or_else(|| format!("未找到可执行文件: {:?}", exe_names))
}
