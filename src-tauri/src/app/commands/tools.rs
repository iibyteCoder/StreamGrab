//! 工具管理命令
//!
//! 外部工具（N_m3u8DL-RE、FFmpeg）的检测、版本查询与下载安装。
//! 检测逻辑复用 `infrastructure::tools`（ToolRegistry + ToolDetector）。

use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

use crate::infrastructure::platform::Platform;
use crate::infrastructure::tools::{tool_names, ToolDetector, ToolInfo, ToolPaths, ToolRegistry};

// ========================================
// 版本信息缓存（5 分钟，规避 GitHub API 限流）
// ========================================

type ReleaseCacheData = (Option<ToolReleaseInfo>, Option<ToolReleaseInfo>);

struct CacheEntry {
    data: ReleaseCacheData,
    timestamp: Instant,
    /// 限流负缓存：在此时间点之前不再请求 GitHub API
    rate_limited_until: Option<Instant>,
}

static RELEASE_CACHE: OnceLock<Mutex<CacheEntry>> = OnceLock::new();
const CACHE_TTL: Duration = Duration::from_secs(5 * 60);
/// 限流后冷却时间（避免反复请求已耗尽配额的 API）
const RATE_LIMIT_COOLDOWN: Duration = Duration::from_secs(5 * 60);

fn cache() -> &'static Mutex<CacheEntry> {
    RELEASE_CACHE.get_or_init(|| {
        Mutex::new(CacheEntry {
            data: (None, None),
            timestamp: Instant::now() - CACHE_TTL - Duration::from_secs(1),
            rate_limited_until: None,
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
    /// 随附的额外压缩包（如 macOS 的 ffprobe 单独发布），与主包解压到同一目录
    #[serde(default)]
    pub extra_assets: Vec<String>,
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
// GitHub / evermeet 版本与下载
// ========================================

/// 获取 N_m3u8DL-RE 最新版本信息
#[tauri::command(rename_all = "camelCase")]
pub async fn get_nm3u8dl_latest_release() -> Result<ToolReleaseInfo, String> {
    if let Some(info) = cached_release(0) {
        return Ok(info);
    }
    check_rate_limit()?;
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
    check_rate_limit()?;
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

/// 检查是否处于限流冷却期
fn check_rate_limit() -> Result<(), String> {
    if let Ok(cache) = cache().lock() {
        if let Some(until) = cache.rate_limited_until {
            if Instant::now() < until {
                let secs = (until - Instant::now()).as_secs();
                return Err(format!("GitHub API 请求频率限制，请 {secs} 秒后重试"));
            }
        }
    }
    Ok(())
}

/// 标记限流（触发冷却期）
fn mark_rate_limited() {
    if let Ok(mut cache) = cache().lock() {
        cache.rate_limited_until = Some(Instant::now() + RATE_LIMIT_COOLDOWN);
    }
}

/// 构建版本查询用 HTTP 客户端
fn api_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .user_agent("StreamGrab")
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {e}"))
}

/// 平台显示名
fn platform_name(platform: Platform) -> &'static str {
    match platform {
        Platform::Windows => "Windows",
        Platform::MacOS => "macOS",
        Platform::Linux => "Linux",
    }
}

/// 从 evermeet.cx 获取单个工具的发布信息（macOS FFmpeg 源）
///
/// 返回 (版本号, zip 下载链接)
async fn evermeet_tool_info(
    client: &reqwest::Client,
    which: &str,
) -> Result<(String, String), String> {
    let url = format!("https://evermeet.cx/ffmpeg/info/{which}/release");
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("请求 evermeet.cx 失败: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("evermeet.cx 返回错误: HTTP {}", response.status()));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("解析 evermeet.cx 响应失败: {e}"))?;

    let version = json["version"]
        .as_str()
        .ok_or_else(|| format!("evermeet.cx 响应缺少 version 字段 ({which})"))?
        .to_string();
    let zip_url = json["download"]["zip"]["url"]
        .as_str()
        .ok_or_else(|| format!("evermeet.cx 响应缺少 zip 下载链接 ({which})"))?
        .to_string();

    Ok((version, zip_url))
}

/// macOS FFmpeg 最新版本（evermeet.cx 源）
///
/// BtbN/FFmpeg-Builds 不提供 macOS 构建，macOS 改走 evermeet.cx：
/// ffmpeg 与 ffprobe 为两个独立 zip，ffprobe 放入 `extra_assets` 一并下载解压。
async fn fetch_ffmpeg_evermeet() -> Result<ToolReleaseInfo, String> {
    let client = api_client()?;
    log::info!("[Tools] macOS 平台：从 evermeet.cx 获取 FFmpeg 最新版本");

    let (ffmpeg_version, ffmpeg_url) = evermeet_tool_info(&client, "ffmpeg").await?;
    let (_ffprobe_version, ffprobe_url) = evermeet_tool_info(&client, "ffprobe").await?;

    let filename = ffmpeg_url
        .rsplit('/')
        .next()
        .unwrap_or("ffmpeg.zip")
        .to_string();
    log::info!("[Tools] evermeet FFmpeg 版本: {ffmpeg_version}, URL: {ffmpeg_url}");

    Ok(ToolReleaseInfo {
        version: ffmpeg_version,
        download_url: ffmpeg_url,
        filename,
        published_at: String::new(),
        extra_assets: vec![ffprobe_url],
    })
}

/// 获取最新 release（经 ToolDefinition 选择当前平台的资产）
async fn fetch_release(
    config: &'static dyn crate::infrastructure::tools::ToolDefinition,
) -> Result<ToolReleaseInfo, String> {
    let platform = Platform::current();

    // macOS FFmpeg：BtbN 无 macOS 构建，改走 evermeet.cx 源
    if platform == Platform::MacOS && config.name() == tool_names::FFMPEG {
        return fetch_ffmpeg_evermeet().await;
    }

    let github_repo = config.github_repo().ok_or("未配置 GitHub 仓库")?;
    let url = format!("https://api.github.com/repos/{github_repo}/releases/latest");
    log::info!("[Tools] 获取最新版本: {url}");

    let client = api_client()?;

    let response = client
        .get(&url)
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .map_err(|e| format!("请求 GitHub API 失败: {e}"))?;

    if response.status() == 403 {
        mark_rate_limited();
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
    // "Latest Auto-Build (2026-08-09 13:03)" → "2026-08-09"
    let version = if tag_name == "latest" && !release_name.is_empty() {
        regex::Regex::new(r"(\d{4}-\d{2}-\d{2})")
            .ok()
            .and_then(|re| re.captures(release_name))
            .and_then(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .unwrap_or_else(|| release_name.to_string())
    } else {
        tag_name.to_string()
    };

    let (download_url, filename) =
        config
            .find_release_asset(&assets, platform)
            .ok_or_else(|| {
                format!(
                    "未找到适合 {} ({}) 的下载链接",
                    platform_name(platform),
                    platform.arch_keywords()
                )
            })?;

    log::info!("[Tools] 版本: {version}, URL: {download_url}");
    Ok(ToolReleaseInfo {
        version,
        download_url,
        filename,
        published_at,
        extra_assets: Vec::new(),
    })
}

/// 下载工具（整包下载 → 完整性校验 → 解压 → 返回可执行文件目录）
///
/// `extra_urls` 为随附的额外压缩包（如 macOS 的 ffprobe zip），
/// 按顺序下载并解压到同一目录。支持 .zip / .tar.gz / .tar.xz。
#[tauri::command(rename_all = "camelCase")]
pub async fn download_tool<R: tauri::Runtime>(
    tool: String,
    download_url: String,
    extra_urls: Vec<String>,
    target_dir: String,
    app: AppHandle<R>,
) -> Result<String, String> {
    // 目标目录为空（工具未安装且未配置路径时，前端会传空串）→ 回退到应用数据目录下的
    // tools/<tool> 子目录，使各工具路径组织一致。ResolvedPath 保证最终路径非空+绝对+存在。
    let target_path = if target_dir.trim().is_empty() {
        let tool_subdir = if tool.to_lowercase().contains("ffmpeg") {
            "ffmpeg"
        } else {
            "nm3u8dl"
        };
        let dir = app
            .path()
            .app_data_dir()
            .map_err(|e| format!("获取应用数据目录失败: {e}"))?
            .join("tools")
            .join(tool_subdir);
        log::info!("[Tools] 未指定目标目录，回退默认目录: {}", dir.display());
        dir
    } else {
        PathBuf::from(&target_dir)
    };

    if !target_path.exists() {
        std::fs::create_dir_all(&target_path).map_err(|e| format!("创建目录失败: {e}"))?;
    }

    // ResolvedPath 编译期保证：非空 + 绝对 + 存在
    let target_resolved = crate::shared::ResolvedPath::from_path(target_path)
        .map_err(|e| format!("目标目录无效: {e}"))?;
    let target_path = target_resolved.into_inner();
    log::info!("[Tools] 开始下载 {tool} 到 {}", target_path.display());

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

    let all_urls: Vec<String> = std::iter::once(download_url).chain(extra_urls).collect();
    let mut archive_paths: Vec<PathBuf> = Vec::with_capacity(all_urls.len());

    for url in &all_urls {
        let filename = url.rsplit('/').next().unwrap_or("download.bin");

        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("下载请求失败: {e}"))?;

        if !response.status().is_success() {
            return Err(format!("下载失败: HTTP {}", response.status()));
        }

        let total_size = response.content_length().unwrap_or(0);

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
        verify_archive_magic(filename, &bytes)?;

        // SHA-256 完整性校验（尝试获取 .sha256 伴随文件）
        crate::infrastructure::fs::verify_download_integrity(&client, url, filename, &bytes)
            .await?;

        let _ = app.emit(
            &format!("tool:download:progress:{tool}"),
            &DownloadProgress {
                tool: tool.clone(),
                status: "downloaded".into(),
                downloaded: actual_size,
                total: total_size.max(actual_size),
                percent: 100.0,
            },
        );

        use std::io::Write;
        let archive_path = target_path.join(filename);
        let mut file =
            std::fs::File::create(&archive_path).map_err(|e| format!("创建文件失败: {e}"))?;
        file.write_all(&bytes)
            .map_err(|e| format!("写入文件失败: {e}"))?;
        file.sync_all().map_err(|e| format!("同步文件失败: {e}"))?;
        archive_paths.push(archive_path);
    }

    let _ = app.emit(
        &format!("tool:download:progress:{tool}"),
        &DownloadProgress {
            tool: tool.clone(),
            status: "extracting".into(),
            downloaded: 0,
            total: 0,
            percent: 100.0,
        },
    );

    let mut exe_dir: Option<String> = None;
    for archive_path in &archive_paths {
        let dir = extract_archive(archive_path, &target_path, &tool)?;
        let _ = std::fs::remove_file(archive_path);
        if exe_dir.is_none() {
            exe_dir = Some(dir);
        }
    }
    let tool_dir = exe_dir.ok_or_else(|| "未定位到可执行文件目录".to_string())?;

    let _ = app.emit(
        &format!("tool:download:complete:{tool}"),
        &serde_json::json!({ "path": &tool_dir }),
    );

    Ok(tool_dir)
}

/// 校验压缩包魔数与扩展名一致（防止把 HTML 错误页当成安装包）
fn verify_archive_magic(filename: &str, bytes: &[u8]) -> Result<(), String> {
    let name = filename.to_lowercase();
    if bytes.len() < 6 {
        return Err("下载文件过小，不是有效的压缩包".to_string());
    }
    let ok = if name.ends_with(".zip") {
        &bytes[0..2] == b"PK"
    } else if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
        bytes[0] == 0x1F && bytes[1] == 0x8B
    } else if name.ends_with(".tar.xz") {
        bytes[0..6] == [0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00]
    } else {
        return Err(format!("不支持的压缩包格式: {filename}"));
    };
    if !ok {
        return Err(format!(
            "下载的文件与 {filename} 的预期格式不符（可能已损坏或为错误页）"
        ));
    }
    Ok(())
}

/// 解压压缩包，返回可执行文件所在目录
///
/// 支持 .zip（Windows 工具包）、.tar.gz（N_m3u8DL-RE 的 macOS/Linux 包）、
/// .tar.xz（BtbN FFmpeg 的 Linux 包）。
///
/// 防更新嵌套：带单一顶层目录的压缩包（如 BtbN zip），若祖先目录中已存在
/// 同名安装目录，则回溯到该目录原地覆盖，避免「bin 里再套一层安装目录」。
fn extract_archive(archive_path: &Path, target_dir: &Path, tool: &str) -> Result<String, String> {
    let registry = ToolRegistry::global();
    let exe_names = if tool.to_lowercase().contains("ffmpeg") {
        registry.ffmpeg().exe_names().all_exe()
    } else {
        registry.downloader().exe_names().all_exe()
    };
    let exe_names_lower: Vec<String> = exe_names.iter().map(|s| s.to_lowercase()).collect();

    let name = archive_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    let name_lower = name.to_lowercase();

    // 先探测压缩包顶层结构，决定实际解压目标（防更新嵌套）
    let (top_dirs, has_root_files) = archive_top_level(archive_path, &name_lower)?;
    let extract_dir = resolve_target_from_top(&top_dirs, has_root_files, target_dir);
    if extract_dir != target_dir {
        log::info!(
            "[Tools] 检测到既有安装目录，原地覆盖更新: {}",
            extract_dir.display()
        );
    }
    if !extract_dir.exists() {
        std::fs::create_dir_all(&extract_dir).map_err(|e| format!("创建解压目录失败: {e}"))?;
    }

    let file = std::fs::File::open(archive_path).map_err(|e| format!("打开压缩包失败: {e}"))?;

    let (found_exe_dir, all_files) = if name_lower.ends_with(".zip") {
        let archive = zip::ZipArchive::new(file).map_err(|e| format!("读取 ZIP 失败: {e}"))?;
        extract_zip_entries(archive, &extract_dir, &exe_names_lower)?
    } else if name_lower.ends_with(".tar.gz") || name_lower.ends_with(".tgz") {
        let gz = flate2::read::GzDecoder::new(file);
        extract_tar_entries(gz, &extract_dir, &exe_names_lower)?
    } else if name_lower.ends_with(".tar.xz") {
        let mut decompressed: Vec<u8> = Vec::new();
        lzma_rs::xz_decompress(&mut std::io::BufReader::new(file), &mut decompressed)
            .map_err(|e| format!("解压 XZ 失败: {e}"))?;
        extract_tar_entries(
            std::io::Cursor::new(decompressed),
            &extract_dir,
            &exe_names_lower,
        )?
    } else {
        return Err(format!("不支持的压缩包格式: {name}"));
    };

    found_exe_dir
        .map(|p| p.to_string_lossy().to_string())
        .ok_or_else(|| {
            format!("未找到可执行文件。期望: {exe_names:?}, 压缩包内实际文件: {all_files:?}")
        })
}

/// 读取压缩包的顶层目录名列表与是否存在根级文件
fn archive_top_level(archive_path: &Path, name_lower: &str) -> Result<(Vec<String>, bool), String> {
    let file = std::fs::File::open(archive_path).map_err(|e| format!("打开压缩包失败: {e}"))?;
    if name_lower.ends_with(".zip") {
        let archive = zip::ZipArchive::new(file).map_err(|e| format!("读取 ZIP 失败: {e}"))?;
        zip_top_level(archive)
    } else if name_lower.ends_with(".tar.gz") || name_lower.ends_with(".tgz") {
        tar_top_level(flate2::read::GzDecoder::new(file))
    } else if name_lower.ends_with(".tar.xz") {
        let mut decompressed: Vec<u8> = Vec::new();
        lzma_rs::xz_decompress(&mut std::io::BufReader::new(file), &mut decompressed)
            .map_err(|e| format!("解压 XZ 失败: {e}"))?;
        tar_top_level(std::io::Cursor::new(decompressed))
    } else {
        // 未知格式按根级文件处理，后续解压环节会给出明确错误
        Ok((Vec::new(), true))
    }
}

/// ZIP 顶层结构探测
fn zip_top_level(
    mut archive: zip::ZipArchive<std::fs::File>,
) -> Result<(Vec<String>, bool), String> {
    let mut top_dirs: Vec<String> = Vec::new();
    let mut has_root_files = false;
    for i in 0..archive.len() {
        let file = archive
            .by_index(i)
            .map_err(|e| format!("读取条目 {i} 失败: {e}"))?;
        let Some(rel) = file.enclosed_name() else {
            continue;
        };
        classify_entry(
            &rel,
            file.name().ends_with('/'),
            &mut top_dirs,
            &mut has_root_files,
        );
        if has_root_files {
            break;
        }
    }
    Ok((top_dirs, has_root_files))
}

/// tar 流顶层结构探测
fn tar_top_level<R: std::io::Read>(reader: R) -> Result<(Vec<String>, bool), String> {
    let mut archive = tar::Archive::new(reader);
    let mut top_dirs: Vec<String> = Vec::new();
    let mut has_root_files = false;
    for entry in archive
        .entries()
        .map_err(|e| format!("读取压缩包条目失败: {e}"))?
    {
        let entry = entry.map_err(|e| format!("读取条目失败: {e}"))?;
        let rel = entry
            .path()
            .map_err(|e| format!("条目路径无效: {e}"))?
            .into_owned();
        let is_dir = entry.header().entry_type().is_dir();
        classify_entry(&rel, is_dir, &mut top_dirs, &mut has_root_files);
        if has_root_files {
            break;
        }
    }
    Ok((top_dirs, has_root_files))
}

/// 条目归类：顶层目录 or 根级文件
fn classify_entry(rel: &Path, is_dir: bool, top_dirs: &mut Vec<String>, has_root_files: &mut bool) {
    let mut comps = rel.components();
    let Some(first) = comps.next() else {
        return;
    };
    let name = first.as_os_str().to_string_lossy().to_string();
    if comps.next().is_some() || is_dir {
        if !top_dirs.iter().any(|d| d.eq_ignore_ascii_case(&name)) {
            top_dirs.push(name);
        }
    } else {
        *has_root_files = true;
    }
}

/// 依据顶层结构决定实际解压目标目录（防更新嵌套）
///
/// 单一顶层目录 `T` 且无根级文件时：自 `target_dir` 向上查找，遇到第一个
/// 含名为 `T` 子目录的祖先目录即在其中解压（原地覆盖既有安装）；
/// 查至文件系统根仍未找到则就地解压（全新安装）。
/// 平铺压缩包（N_m3u8DL-RE / evermeet）直接解压到 `target_dir`。
fn resolve_target_from_top(
    top_dirs: &[String],
    has_root_files: bool,
    target_dir: &Path,
) -> PathBuf {
    if has_root_files || top_dirs.len() != 1 {
        return target_dir.to_path_buf();
    }
    let top = &top_dirs[0];
    let mut cur = target_dir.to_path_buf();
    loop {
        if contains_dir(&cur, top) {
            return cur;
        }
        match cur.parent() {
            Some(parent) => cur = parent.to_path_buf(),
            None => return target_dir.to_path_buf(),
        }
    }
}

/// dir 中是否包含指定名称的子目录（不区分大小写）
fn contains_dir(dir: &Path, name: &str) -> bool {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries.flatten().any(|e| {
                e.path().is_dir()
                    && e.file_name()
                        .to_str()
                        .map(|n| n.eq_ignore_ascii_case(name))
                        .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

/// 解压 ZIP 条目，返回 (可执行文件目录, 全部文件名列表)
fn extract_zip_entries(
    mut archive: zip::ZipArchive<std::fs::File>,
    target_dir: &Path,
    exe_names_lower: &[String],
) -> Result<(Option<PathBuf>, Vec<String>), String> {
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

        // Unix 平台确保可执行文件具备执行权限（zip 不一定携带权限信息）
        #[cfg(unix)]
        {
            let is_exe = outpath
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| exe_names_lower.contains(&n.to_lowercase()))
                .unwrap_or(false);
            if is_exe {
                set_executable(&outpath, file.unix_mode());
            }
        }

        // 记录可执行文件所在目录（不区分大小写）
        if found_exe_dir.is_none() {
            if let Some(filename) = outpath.file_name().and_then(|n| n.to_str()) {
                if exe_names_lower.contains(&filename.to_lowercase()) {
                    found_exe_dir = outpath.parent().map(|p| p.to_path_buf());
                }
            }
        }
    }

    Ok((found_exe_dir, all_files))
}

/// 解压 tar 流条目，返回 (可执行文件目录, 全部文件名列表)
///
/// `tar` crate 的 `entries()` 已拒绝绝对路径与 `..` 组件，
/// 此处额外做一次 starts_with 包含性校验作为纵深防御。
fn extract_tar_entries<R: std::io::Read>(
    reader: R,
    target_dir: &Path,
    exe_names_lower: &[String],
) -> Result<(Option<PathBuf>, Vec<String>), String> {
    let mut archive = tar::Archive::new(reader);
    let mut found_exe_dir: Option<PathBuf> = None;
    let mut all_files: Vec<String> = Vec::new();

    for entry in archive
        .entries()
        .map_err(|e| format!("读取压缩包条目失败: {e}"))?
    {
        let mut entry = entry.map_err(|e| format!("读取条目失败: {e}"))?;

        let rel_path = entry
            .path()
            .map_err(|e| format!("条目路径无效: {e}"))?
            .into_owned();
        let outpath = target_dir.join(&rel_path);
        if !outpath.starts_with(target_dir) {
            log::warn!("[Tools] 跳过可疑路径: {}", outpath.display());
            continue;
        }

        if entry.header().entry_type().is_dir() {
            std::fs::create_dir_all(&outpath).map_err(|e| format!("创建目录失败: {e}"))?;
            continue;
        }
        if !entry.header().entry_type().is_file() {
            continue; // 跳过符号链接等非普通文件
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
        std::io::copy(&mut entry, &mut outfile).map_err(|e| format!("写入文件失败: {e}"))?;

        // Unix 平台恢复压缩包携带的权限（含可执行位）
        #[cfg(unix)]
        if let Ok(mode) = entry.header().mode() {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode));
        }

        // 记录可执行文件所在目录（不区分大小写）
        if found_exe_dir.is_none() {
            if let Some(filename) = outpath.file_name().and_then(|n| n.to_str()) {
                if exe_names_lower.contains(&filename.to_lowercase()) {
                    found_exe_dir = outpath.parent().map(|p| p.to_path_buf());
                }
            }
        }
    }

    Ok((found_exe_dir, all_files))
}

/// Unix 平台设置可执行权限（优先使用压缩包携带的 mode，否则默认 0o755）
#[cfg(unix)]
fn set_executable(path: &Path, archive_mode: Option<u32>) {
    use std::os::unix::fs::PermissionsExt;
    let mode = archive_mode.unwrap_or(0o755);
    let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode));
}

// ========================================
// 解压目标决策测试（防更新嵌套）
// ========================================

#[cfg(test)]
mod tests {
    use super::*;

    const TOP: &str = "ffmpeg-master-latest-win64-gpl-shared";

    #[test]
    fn update_extracts_into_existing_install_root() {
        // 更新场景：配置目录是安装目录内的 bin/ → 应回溯到安装根目录原地覆盖
        let root = tempfile::tempdir().unwrap();
        let bin = root.path().join(TOP).join("bin");
        std::fs::create_dir_all(&bin).unwrap();
        let target = resolve_target_from_top(&[TOP.to_string()], false, &bin);
        assert_eq!(target, root.path());
    }

    #[test]
    fn fresh_install_keeps_target_dir() {
        // 全新安装：空目录中不存在顶层同名目录 → 就地解压
        let root = tempfile::tempdir().unwrap();
        let target = resolve_target_from_top(&[TOP.to_string()], false, root.path());
        assert_eq!(target, root.path());
    }

    #[test]
    fn flat_or_multi_top_archives_keep_target_dir() {
        let root = tempfile::tempdir().unwrap();
        // 含根级文件（平铺包）→ 就地解压
        let target = resolve_target_from_top(&[TOP.to_string()], true, root.path());
        assert_eq!(target, root.path());
        // 多个顶层目录 → 就地解压
        let target =
            resolve_target_from_top(&["a".to_string(), "b".to_string()], false, root.path());
        assert_eq!(target, root.path());
    }

    #[test]
    fn corrupted_nested_dir_resolves_to_nearest_install() {
        // 历史损坏形态：bin 内又嵌了一层完整安装目录 → 就近回溯
        let root = tempfile::tempdir().unwrap();
        let corrupted_bin = root.path().join(TOP).join("bin").join(TOP).join("bin");
        std::fs::create_dir_all(&corrupted_bin).unwrap();
        let target = resolve_target_from_top(&[TOP.to_string()], false, &corrupted_bin);
        assert_eq!(target, root.path().join(TOP).join("bin"));
    }
}

// ========================================
// 实测集成测试（需联网，手动运行）
// ========================================

#[cfg(test)]
mod live_tests {
    use super::*;

    /// 真实网络：拉取 N_m3u8DL-RE 最新版本并校验平台资产选择
    /// 运行：cargo test --lib live_fetch_nm3u8dl -- --ignored --nocapture
    #[tokio::test]
    #[ignore = "需要网络"]
    async fn live_fetch_nm3u8dl_release() {
        let registry = ToolRegistry::global();
        let info = fetch_release(registry.downloader())
            .await
            .expect("拉取 N_m3u8DL-RE 最新版本失败");
        println!("N_m3u8DL-RE: {info:?}");
        assert!(!info.version.is_empty());
        assert!(info.extra_assets.is_empty());

        // 按当前平台校验资产格式
        let expected_ext = match Platform::current() {
            Platform::Windows => ".zip",
            Platform::MacOS | Platform::Linux => ".tar.gz",
        };
        assert!(
            info.filename.to_lowercase().ends_with(expected_ext),
            "当前平台应下载 {expected_ext}: {}",
            info.filename
        );
    }

    /// 真实网络：拉取 FFmpeg 最新版本并校验平台源选择
    /// 运行：cargo test --lib live_fetch_ffmpeg -- --ignored --nocapture
    #[tokio::test]
    #[ignore = "需要网络"]
    async fn live_fetch_ffmpeg_release() {
        let registry = ToolRegistry::global();
        let info = fetch_release(registry.ffmpeg())
            .await
            .expect("拉取 FFmpeg 最新版本失败");
        println!("FFmpeg: {info:?}");
        assert!(!info.version.is_empty());

        match Platform::current() {
            Platform::Windows => {
                assert!(info.filename.ends_with(".zip"), "Windows 应为 BtbN zip");
                assert!(info.extra_assets.is_empty());
            }
            Platform::MacOS => {
                // evermeet 源：ffmpeg + ffprobe 两个 zip
                assert!(info.download_url.contains("evermeet.cx"));
                assert_eq!(info.extra_assets.len(), 1, "macOS 应随附 ffprobe 下载");
            }
            Platform::Linux => {
                assert!(info.filename.ends_with(".tar.xz"), "Linux 应为 BtbN tar.xz");
                assert!(info.extra_assets.is_empty());
            }
        }
    }
}
