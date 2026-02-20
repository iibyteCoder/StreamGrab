//! 工具管理命令
//!
//! 提供外部工具（N_m3u8DL-RE、FFmpeg）的版本检测、下载和管理功能

use std::path::PathBuf;
use std::process::Command;

use log::info;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

/// 工具信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    /// 工具名称
    pub name: String,
    /// 是否已安装（路径有效）
    pub installed: bool,
    /// 版本号
    pub version: Option<String>,
    /// 安装路径
    pub path: Option<String>,
    /// 错误信息
    pub error: Option<String>,
}

/// 工具下载进度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    /// 工具名称
    pub tool: String,
    /// 下载状态
    pub status: String,
    /// 已下载字节数
    pub downloaded: u64,
    /// 总字节数
    pub total: u64,
    /// 百分比
    pub percent: f64,
}

/// 工具下载信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolReleaseInfo {
    /// 版本号
    pub version: String,
    /// 下载 URL
    pub download_url: String,
    /// 文件名
    pub filename: String,
    /// 发布日期
    pub published_at: String,
}

/// 获取 N_m3u8DL-RE 工具信息
#[tauri::command(rename_all = "camelCase")]
pub async fn get_nm3u8dl_info(path: Option<String>) -> Result<ToolInfo, String> {
    let tool_name = "N_m3u8DL-RE";

    // 确定路径
    let tool_path = match path {
        Some(p) if !p.is_empty() => PathBuf::from(p),
        _ => {
            // 尝试从系统 PATH 查找
            which::which("N_m3u8DL-RE").map_err(|_| format!("{} 未找到，请配置路径", tool_name))?
        }
    };

    // 检查文件是否存在
    if !tool_path.exists() {
        return Ok(ToolInfo {
            name: tool_name.to_string(),
            installed: false,
            version: None,
            path: Some(tool_path.to_string_lossy().to_string()),
            error: Some("文件不存在".to_string()),
        });
    }

    // 执行命令获取版本
    let output = Command::new(&tool_path).arg("--version").output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            // 解析版本号（格式：N_m3u8DL-RE version X.X.X 或简单版本号）
            let version = parse_nm3u8dl_version(&stdout, &stderr);

            Ok(ToolInfo {
                name: tool_name.to_string(),
                installed: true,
                version,
                path: Some(tool_path.to_string_lossy().to_string()),
                error: None,
            })
        }
        Err(e) => Ok(ToolInfo {
            name: tool_name.to_string(),
            installed: false,
            version: None,
            path: Some(tool_path.to_string_lossy().to_string()),
            error: Some(format!("执行失败: {}", e)),
        }),
    }
}

/// 获取 FFmpeg 工具信息
#[tauri::command(rename_all = "camelCase")]
pub async fn get_ffmpeg_info(path: Option<String>) -> Result<ToolInfo, String> {
    let tool_name = "FFmpeg";

    // 确定路径
    let tool_path = match path {
        Some(p) if !p.is_empty() => PathBuf::from(p),
        _ => {
            // 尝试从系统 PATH 查找
            which::which("ffmpeg").map_err(|_| format!("{} 未找到，请配置路径", tool_name))?
        }
    };

    // 检查文件是否存在
    if !tool_path.exists() {
        return Ok(ToolInfo {
            name: tool_name.to_string(),
            installed: false,
            version: None,
            path: Some(tool_path.to_string_lossy().to_string()),
            error: Some("文件不存在".to_string()),
        });
    }

    // 执行命令获取版本
    let output = Command::new(&tool_path).arg("-version").output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();

            // 解析版本号（格式：ffmpeg version X.X.X ...）
            let version = parse_ffmpeg_version(&stdout);

            Ok(ToolInfo {
                name: tool_name.to_string(),
                installed: true,
                version,
                path: Some(tool_path.to_string_lossy().to_string()),
                error: None,
            })
        }
        Err(e) => Ok(ToolInfo {
            name: tool_name.to_string(),
            installed: false,
            version: None,
            path: Some(tool_path.to_string_lossy().to_string()),
            error: Some(format!("执行失败: {}", e)),
        }),
    }
}

/// 获取 N_m3u8DL-RE 最新版本信息
#[tauri::command(rename_all = "camelCase")]
pub async fn get_nm3u8dl_latest_release() -> Result<ToolReleaseInfo, String> {
    let url = "https://api.github.com/repos/nilaoda/N_m3u8DL-RE/releases/latest";

    info!("[Tools] 获取 N_m3u8DL-RE 最新版本: {}", url);

    let client = reqwest::Client::builder()
        .user_agent("StreamGrab-Tool-Checker")
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let response = client
        .get(url)
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await
        .map_err(|e| format!("请求 GitHub API 失败: {}", e))?;

    info!("[Tools] 响应状态: {}", response.status());

    if !response.status().is_success() {
        return Err(format!("GitHub API 返回错误: {}", response.status()));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    let version = json["tag_name"].as_str().unwrap_or("unknown").to_string();

    let published_at = json["published_at"].as_str().unwrap_or("").to_string();

    // 查找 Windows x64 的下载链接
    let download_url = json["assets"]
        .as_array()
        .and_then(|assets| {
            assets.iter().find(|asset| {
                let name = asset["name"].as_str().unwrap_or("");
                // 查找 Windows x64 版本
                name.contains("win-x64") && name.ends_with(".zip")
            })
        })
        .and_then(|asset| asset["browser_download_url"].as_str())
        .unwrap_or("")
        .to_string();

    let filename = download_url
        .rsplit('/')
        .next()
        .unwrap_or("N_m3u8DL-RE.zip")
        .to_string();

    info!(
        "[Tools] N_m3u8DL-RE 版本: {}, 下载链接: {}",
        version, download_url
    );

    if download_url.is_empty() {
        return Err("未找到 Windows x64 版本的下载链接".to_string());
    }

    Ok(ToolReleaseInfo {
        version,
        download_url,
        filename,
        published_at,
    })
}

/// 获取 FFmpeg 最新版本信息
#[tauri::command(rename_all = "camelCase")]
pub async fn get_ffmpeg_latest_release() -> Result<ToolReleaseInfo, String> {
    // 使用 BtbN GitHub releases 作为 FFmpeg 下载源
    let url = "https://api.github.com/repos/BtbN/FFmpeg-Builds/releases/latest";

    info!("[Tools] 获取 FFmpeg 最新版本: {}", url);

    let client = reqwest::Client::builder()
        .user_agent("StreamGrab-Tool-Checker")
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let response = client
        .get(url)
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await
        .map_err(|e| format!("请求 GitHub API 失败: {}", e))?;

    info!("[Tools] 响应状态: {}", response.status());

    if !response.status().is_success() {
        return Err(format!("GitHub API 返回错误: {}", response.status()));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    let version = json["tag_name"].as_str().unwrap_or("unknown").to_string();

    let published_at = json["published_at"].as_str().unwrap_or("").to_string();

    // 查找 Windows x64 GPL shared 版本（包含 ffprobe）
    let download_url = json["assets"]
        .as_array()
        .and_then(|assets| {
            assets.iter().find(|asset| {
                let name = asset["name"].as_str().unwrap_or("");
                // 查找 Windows 64-bit GPL shared 版本
                name.contains("win64")
                    && name.contains("gpl")
                    && name.contains("shared")
                    && name.ends_with(".zip")
            })
        })
        .and_then(|asset| asset["browser_download_url"].as_str())
        .unwrap_or("")
        .to_string();

    let filename = download_url
        .rsplit('/')
        .next()
        .unwrap_or("ffmpeg.zip")
        .to_string();

    info!(
        "[Tools] FFmpeg 版本: {}, 下载链接: {}",
        version, download_url
    );

    if download_url.is_empty() {
        return Err("未找到 Windows x64 GPL shared 版本的下载链接".to_string());
    }

    Ok(ToolReleaseInfo {
        version,
        download_url,
        filename,
        published_at,
    })
}

/// 下载工具
#[tauri::command(rename_all = "camelCase")]
pub async fn download_tool(
    tool: String,
    download_url: String,
    target_dir: String,
    app: AppHandle,
) -> Result<String, String> {
    let target_path = PathBuf::from(&target_dir);

    info!("[Tools] 开始下载 {} 到 {}", tool, target_dir);
    info!("[Tools] 下载链接: {}", download_url);

    // 确保目标目录存在
    if !target_path.exists() {
        std::fs::create_dir_all(&target_path).map_err(|e| format!("创建目录失败: {}", e))?;
        info!("[Tools] 创建目录: {}", target_dir);
    }

    // 发送开始下载事件
    let _ = app.emit(
        &format!("tool:download:start:{}", tool),
        &serde_json::json!({ "url": download_url }),
    );

    // 创建 HTTP 客户端
    let client = reqwest::Client::builder()
        .user_agent("StreamGrab-Downloader")
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    // 发起请求
    let mut response = client
        .get(&download_url)
        .send()
        .await
        .map_err(|e| format!("下载请求失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("下载失败: HTTP {}", response.status()));
    }

    // 获取文件总大小
    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;

    info!("[Tools] 文件大小: {} bytes", total_size);

    // 确定文件名
    let filename = download_url.rsplit('/').next().unwrap_or("download.zip");
    let zip_path = target_path.join(filename);

    info!("[Tools] 保存到: {:?}", zip_path);

    // 创建临时文件
    let mut file = std::fs::File::create(&zip_path).map_err(|e| format!("创建文件失败: {}", e))?;

    // 下载文件
    use std::io::Write;
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|e| format!("下载数据块失败: {}", e))?
    {
        file.write_all(&chunk)
            .map_err(|e| format!("写入文件失败: {}", e))?;
        downloaded += chunk.len() as u64;

        // 发送进度事件
        let percent = if total_size > 0 {
            (downloaded as f64 / total_size as f64) * 100.0
        } else {
            0.0
        };

        let _ = app.emit(
            &format!("tool:download:progress:{}", tool),
            &DownloadProgress {
                tool: tool.clone(),
                status: "downloading".to_string(),
                downloaded,
                total: total_size,
                percent,
            },
        );
    }

    info!("[Tools] 下载完成，开始解压...");

    // 发送解压开始事件
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

    // 解压文件
    let extracted_path = extract_zip(&zip_path, &target_path, &tool)?;

    info!("[Tools] 解压完成，可执行文件: {}", extracted_path);

    // 删除 zip 文件
    let _ = std::fs::remove_file(&zip_path);

    // 发送完成事件
    let _ = app.emit(
        &format!("tool:download:complete:{}", tool),
        &serde_json::json!({ "path": extracted_path }),
    );

    Ok(extracted_path)
}

/// 解压 ZIP 文件
fn extract_zip(
    zip_path: &std::path::Path,
    target_dir: &std::path::Path,
    tool: &str,
) -> Result<String, String> {
    info!("[Tools] 解压 ZIP 文件: {:?}", zip_path);

    let file = std::fs::File::open(zip_path).map_err(|e| format!("打开 ZIP 文件失败: {}", e))?;

    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("读取 ZIP 文件失败: {}", e))?;

    // 确定工具的可执行文件名
    let exe_name = if tool.to_lowercase().contains("ffmpeg") {
        "ffmpeg.exe"
    } else {
        "N_m3u8DL-RE.exe"
    };

    info!("[Tools] 查找可执行文件: {}", exe_name);

    let mut found_exe_path: Option<String> = None;
    let total_files = archive.len();

    for i in 0..total_files {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("读取 ZIP 条目失败: {}", e))?;
        let outpath = match file.enclosed_name() {
            Some(path) => target_dir.join(path),
            None => continue,
        };

        if file.name().ends_with('/') {
            // 创建目录
            std::fs::create_dir_all(&outpath).map_err(|e| format!("创建目录失败: {}", e))?;
        } else {
            // 创建父目录
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(p).map_err(|e| format!("创建目录失败: {}", e))?;
                }
            }

            // 提取文件
            let mut outfile =
                std::fs::File::create(&outpath).map_err(|e| format!("创建文件失败: {}", e))?;
            std::io::copy(&mut file, &mut outfile).map_err(|e| format!("写入文件失败: {}", e))?;

            // 检查是否是目标可执行文件
            if outpath
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n == exe_name)
                .unwrap_or(false)
            {
                found_exe_path = Some(outpath.to_string_lossy().to_string());
                info!("[Tools] 找到可执行文件: {:?}", outpath);
            }
        }
    }

    info!("[Tools] 解压完成，共 {} 个文件", total_files);

    found_exe_path.ok_or_else(|| format!("未找到 {} 可执行文件", exe_name))
}

/// 解析 N_m3u8DL-RE 版本号
fn parse_nm3u8dl_version(stdout: &str, stderr: &str) -> Option<String> {
    // 预编译正则表达式
    let version_re = regex::Regex::new(r"(\d+\.\d+\.\d+)").ok()?;

    // 尝试从 stdout 解析
    for line in stdout.lines() {
        // 格式：N_m3u8DL-RE version X.X.X 或 version X.X.X
        if line.contains("version") || line.contains("Version") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            for (i, part) in parts.iter().enumerate() {
                if part.to_lowercase().contains("version") && i + 1 < parts.len() {
                    return Some(parts[i + 1].to_string());
                }
            }
            // 直接查找版本号模式
            if let Some(cap) = version_re.captures(line) {
                return Some(cap.get(1)?.as_str().to_string());
            }
        }
    }

    // 尝试从 stderr 解析（有些版本信息可能在 stderr）
    for line in stderr.lines() {
        if let Some(cap) = version_re.captures(line) {
            return Some(cap.get(1)?.as_str().to_string());
        }
    }

    None
}

/// 解析 FFmpeg 版本号
fn parse_ffmpeg_version(stdout: &str) -> Option<String> {
    // 格式：ffmpeg version X.X.X ...
    let first_line = stdout.lines().next()?;
    let re = regex::Regex::new(r"ffmpeg\s+version\s+(\d+\.\d+(?:\.\d+)?)").ok()?;
    re.captures(first_line)
        .and_then(|cap| cap.get(1).map(|m| m.as_str().to_string()))
}
