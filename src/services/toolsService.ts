/**
 * 工具管理服务
 *
 * 提供外部工具（N_m3u8DL-RE、FFmpeg）的版本检测、下载和管理功能
 *
 * ## 核心概念
 *
 * - **dirPath**: 用户配置的目录路径，包含可执行文件
 * - **exePath**: 检测到的实际可执行文件完整路径
 */

import { listen } from "@tauri-apps/api/event";
import { tauriInvoke } from "./tauri";

// ========================================
// 类型定义
// ========================================

/** 工具信息 */
export interface ToolInfo {
  /** 工具名称 */
  name: string;
  /** 是否已安装（可执行文件存在且可运行） */
  installed: boolean;
  /** 版本号 */
  version: string | null;
  /** 可执行文件完整路径（检测后获得） */
  exePath: string | null;
  /** 配置的目录路径（用户设置） */
  dirPath: string | null;
  /** 错误信息（未安装时） */
  error: string | null;
}

/** 工具下载进度 */
export interface DownloadProgress {
  tool: string;
  status: "downloading" | "extracting" | "complete" | "error";
  downloaded: number;
  total: number;
  percent: number;
}

/** 工具发布信息 */
export interface ToolReleaseInfo {
  version: string;
  downloadUrl: string;
  filename: string;
  publishedAt: string;
}

/** 工具状态检查结果 */
export interface ToolsStatus {
  downloader: ToolInfo;
  ffmpeg: ToolInfo;
  ffprobe: ToolInfo;
}

type DownloadProgressCallback = (progress: DownloadProgress) => void;

// ========================================
// API 函数
// ========================================

/** 获取 N_m3u8DL-RE 工具信息 */
export async function getNm3u8dlInfo(dirPath?: string): Promise<ToolInfo> {
  return tauriInvoke("get_nm3u8dl_info", { path: dirPath || null });
}

/** 获取 FFmpeg 工具信息 */
export async function getFfmpegInfo(dirPath?: string): Promise<ToolInfo> {
  return tauriInvoke("get_ffmpeg_info", { path: dirPath || null });
}

/** 获取 FFprobe 工具信息 */
export async function getFfprobeInfo(
  ffmpegDirPath?: string,
): Promise<ToolInfo> {
  return tauriInvoke("get_ffprobe_info", { ffmpegPath: ffmpegDirPath || null });
}

/** 获取 N_m3u8DL-RE 最新版本信息 */
export async function getNm3u8dlLatestRelease(): Promise<ToolReleaseInfo> {
  return tauriInvoke("get_nm3u8dl_latest_release");
}

/** 获取 FFmpeg 最新版本信息 */
export async function getFfmpegLatestRelease(): Promise<ToolReleaseInfo> {
  return tauriInvoke("get_ffmpeg_latest_release");
}

/** 下载工具 */
export async function downloadTool(
  tool: string,
  downloadUrl: string,
  targetDir: string,
  onProgress?: DownloadProgressCallback,
): Promise<string> {
  let unlisten: (() => void) | null = null;

  if (onProgress) {
    unlisten = await listen<DownloadProgress>(
      `tool:download:progress:${tool}`,
      (event) => onProgress(event.payload),
    );
  }

  try {
    return await tauriInvoke("download_tool", { tool, downloadUrl, targetDir });
  } finally {
    unlisten?.();
  }
}

/** 检查所有工具状态 */
export async function checkAllToolsStatus(
  downloaderDir?: string,
  ffmpegDir?: string,
): Promise<ToolsStatus> {
  const [downloader, ffmpeg, ffprobe] = await Promise.all([
    getNm3u8dlInfo(downloaderDir),
    getFfmpegInfo(ffmpegDir),
    getFfprobeInfo(ffmpegDir),
  ]);

  return { downloader, ffmpeg, ffprobe };
}
