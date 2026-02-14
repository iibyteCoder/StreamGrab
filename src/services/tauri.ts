/**
 * Tauri API 封装层
 * 提供类型安全的 Tauri 命令调用接口
 */

import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

/**
 * 通用 Tauri 命令调用封装
 */
export async function tauriInvoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    console.error(`Tauri command '${command}' failed:`, error);
    throw error;
  }
}

/**
 * 事件订阅封装
 */
export async function tauriListen<T>(
  event: string,
  handler: (payload: T) => void
): Promise<UnlistenFn> {
  return listen<T>(event, (event) => {
    handler(event.payload);
  });
}

// ============================================
// 下载相关命令
// ============================================

export interface StartDownloadArgs {
  taskId: string;
  url: string;
  args: string[];
}

export interface DownloadProgressEvent {
  taskId: string;
  type: 'progress' | 'completed' | 'error' | 'log';
  data: {
    downloadedSegments?: number;
    totalSegments?: number;
    percentage?: number;
    speed?: number;
    speedFormatted?: string;
    downloadedBytes?: number;
    totalBytes?: number;
    elapsedTime?: number;
    estimatedTime?: number;
    currentAction?: string;
    outputPath?: string;
    message?: string;
  };
}

/**
 * 开始下载任务
 */
export async function startDownload(args: StartDownloadArgs): Promise<void> {
  // 使用 snake_case 与 Rust 后端匹配
  return tauriInvoke('start_download', {
    task_id: args.taskId,
    url: args.url,
    args: args.args,
  });
}

/**
 * 停止下载任务
 */
export async function stopDownload(taskId: string): Promise<void> {
  return tauriInvoke('stop_download', { task_id: taskId });
}

/**
 * 暂停下载任务
 */
export async function pauseDownload(taskId: string): Promise<void> {
  return tauriInvoke('pause_download', { task_id: taskId });
}

/**
 * 恢复下载任务
 */
export async function resumeDownload(taskId: string): Promise<void> {
  return tauriInvoke('resume_download', { task_id: taskId });
}

/**
 * 解析 URL 获取流信息
 */
export async function parseUrl(url: string): Promise<unknown> {
  return tauriInvoke('parse_url', { url });
}

/**
 * 订阅下载进度事件
 */
export async function subscribeToDownloadProgress(
  taskId: string,
  handler: (event: DownloadProgressEvent) => void
): Promise<UnlistenFn> {
  return tauriListen<DownloadProgressEvent>(`download:${taskId}`, handler);
}

// ============================================
// 配置相关命令
// ============================================

/**
 * 加载应用配置
 */
export async function loadConfig(): Promise<Record<string, unknown>> {
  return tauriInvoke('load_config');
}

/**
 * 保存应用配置
 */
export async function saveConfig(config: Record<string, unknown>): Promise<void> {
  return tauriInvoke('save_config', { config });
}

/**
 * 获取默认下载目录
 */
export async function getDefaultDownloadDir(): Promise<string> {
  return tauriInvoke('get_default_download_dir');
}

/**
 * 获取默认临时目录
 */
export async function getDefaultTempDir(): Promise<string> {
  return tauriInvoke('get_default_temp_dir');
}

// ============================================
// 系统相关命令
// ============================================

/**
 * 打开文件所在目录
 */
export async function openInExplorer(path: string): Promise<void> {
  return tauriInvoke('open_in_explorer', { path });
}

/**
 * 检查文件是否存在
 */
export async function fileExists(path: string): Promise<boolean> {
  return tauriInvoke('file_exists', { path });
}

/**
 * 获取 N_m3u8DL-RE 版本
 */
export async function getN_m3u8dlVersion(): Promise<string> {
  return tauriInvoke('get_n_m3u8dl_version');
}

/**
 * 检查 FFmpeg 是否可用
 */
export async function checkFfmpegAvailable(): Promise<boolean> {
  return tauriInvoke('check_ffmpeg_available');
}

// ============================================
// 导出
// ============================================

export const tauriApi = {
  invoke: tauriInvoke,
  listen: tauriListen,
  // 下载
  startDownload,
  stopDownload,
  pauseDownload,
  resumeDownload,
  parseUrl,
  subscribeToDownloadProgress,
  // 配置
  loadConfig,
  saveConfig,
  getDefaultDownloadDir,
  getDefaultTempDir,
  // 系统
  openInExplorer,
  fileExists,
  getN_m3u8dlVersion,
  checkFfmpegAvailable,
};

// 别名导出，方便其他模块使用
export const invokeTauri = tauriInvoke;
export const subscribeToEvent = tauriListen;
export type { UnlistenFn };

export default tauriApi;
