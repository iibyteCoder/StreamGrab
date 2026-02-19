/**
 * Tauri API 封装层
 * 提供类型安全的 Tauri 命令调用接口
 */

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

/**
 * 通用 Tauri 命令调用封装
 */
export async function tauriInvoke<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
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
  handler: (payload: T) => void,
): Promise<UnlistenFn> {
  return listen<T>(event, (event) => {
    handler(event.payload);
  });
}

// ============================================
// 下载相关命令
// ============================================

export interface DownloadProgressEvent {
  taskId: string;
  type: "progress" | "completed" | "error" | "log";
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
 * 订阅下载进度事件
 */
export async function subscribeToDownloadProgress(
  taskId: string,
  handler: (event: DownloadProgressEvent) => void,
): Promise<UnlistenFn> {
  return tauriListen<DownloadProgressEvent>(`download:${taskId}`, handler);
}

// ============================================
// 系统相关命令
// ============================================

/**
 * 打开文件所在目录
 */
export async function openInExplorer(path: string): Promise<void> {
  return tauriInvoke("open_in_explorer", { path });
}

/**
 * 检查文件是否存在
 */
export async function fileExists(path: string): Promise<boolean> {
  return tauriInvoke("file_exists", { path });
}

/**
 * 获取 N_m3u8DL-RE 版本
 */
export async function getN_m3u8dlVersion(): Promise<string> {
  return tauriInvoke("get_n_m3u8dl_version");
}

// ============================================
// 导出
// ============================================

export const tauriApi = {
  invoke: tauriInvoke,
  listen: tauriListen,
  // 下载
  subscribeToDownloadProgress,
  // 系统
  openInExplorer,
  fileExists,
  getN_m3u8dlVersion,
};

// 别名导出，方便其他模块使用
export const invokeTauri = tauriInvoke;
export const subscribeToEvent = tauriListen;
export type { UnlistenFn };

export default tauriApi;
