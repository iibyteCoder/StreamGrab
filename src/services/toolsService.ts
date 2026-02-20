/**
 * 工具管理服务
 * 提供外部工具（N_m3u8DL-RE、FFmpeg）的版本检测、下载和管理功能
 */

import { listen } from "@tauri-apps/api/event";
import { tauriInvoke } from "./tauri";

// 工具信息接口
export interface ToolInfo {
  name: string;
  installed: boolean;
  version: string | null;
  path: string | null;
  error: string | null;
}

// 工具下载进度接口
export interface DownloadProgress {
  tool: string;
  status: "downloading" | "extracting" | "complete" | "error";
  downloaded: number;
  total: number;
  percent: number;
}

// 工具版本信息接口
export interface ToolReleaseInfo {
  version: string;
  downloadUrl: string;
  filename: string;
  publishedAt: string;
}

// 下载状态回调类型
type DownloadProgressCallback = (progress: DownloadProgress) => void;

/**
 * 获取 N_m3u8DL-RE 工具信息
 */
export async function getNm3u8dlInfo(path?: string): Promise<ToolInfo> {
  return tauriInvoke("get_nm3u8dl_info", { path: path || null });
}

/**
 * 获取 FFmpeg 工具信息
 */
export async function getFfmpegInfo(path?: string): Promise<ToolInfo> {
  return tauriInvoke("get_ffmpeg_info", { path: path || null });
}

/**
 * 获取 N_m3u8DL-RE 最新版本信息
 */
export async function getNm3u8dlLatestRelease(): Promise<ToolReleaseInfo> {
  return tauriInvoke("get_nm3u8dl_latest_release");
}

/**
 * 获取 FFmpeg 最新版本信息
 */
export async function getFfmpegLatestRelease(): Promise<ToolReleaseInfo> {
  return tauriInvoke("get_ffmpeg_latest_release");
}

/**
 * 下载工具
 * @param tool 工具名称（"N_m3u8DL-RE" 或 "FFmpeg"）
 * @param downloadUrl 下载链接
 * @param targetDir 目标目录
 * @param onProgress 进度回调
 */
export async function downloadTool(
  tool: string,
  downloadUrl: string,
  targetDir: string,
  onProgress?: DownloadProgressCallback,
): Promise<string> {
  // 设置进度监听
  let unlisten: (() => void) | null = null;

  if (onProgress) {
    unlisten = await listen<DownloadProgress>(
      `tool:download:progress:${tool}`,
      (event) => {
        onProgress(event.payload);
      },
    );
  }

  try {
    const result = await tauriInvoke("download_tool", {
      tool,
      downloadUrl,
      targetDir,
    });
    return result as string;
  } finally {
    if (unlisten) {
      unlisten();
    }
  }
}

/**
 * 工具状态检查结果
 */
export interface ToolsStatus {
  nm3u8dl: ToolInfo;
  ffmpeg: ToolInfo;
}

/**
 * 检查所有工具状态
 */
export async function checkAllToolsStatus(
  nm3u8dlPath?: string,
  ffmpegPath?: string,
): Promise<ToolsStatus> {
  const [nm3u8dl, ffmpeg] = await Promise.all([
    getNm3u8dlInfo(nm3u8dlPath),
    getFfmpegInfo(ffmpegPath),
  ]);

  return { nm3u8dl, ffmpeg };
}

/**
 * 获取默认工具目录
 */
export function getDefaultToolsDir(): string {
  // 返回应用目录下的 tools 子目录
  // 这个路径会在后端根据实际应用目录解析
  return "tools";
}
