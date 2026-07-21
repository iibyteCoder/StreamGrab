/**
 * 下载服务
 *
 * 与后端 download 命令组对应。命令行参数完全由后端引擎构建——
 * 前端只传 taskId，不感知任何工具的 CLI 细节。
 */

import { invokeTauri, subscribeToEvent, type UnlistenFn } from "./tauri";
import type { MediaInfo, ProgressData, StreamInfo, UrlType } from "@/domain";
import type { FileInfo } from "./systemService";

/** 下载事件类型 */
export type DownloadEventType =
  | "progress"
  | "status"
  | "error"
  | "complete"
  | "log";

/** 下载事件（订阅回调的统一载荷） */
export interface DownloadEvent {
  type: DownloadEventType;
  taskId: string;
  data: unknown;
}

/** 状态事件数据 */
export interface StatusEventData {
  action: string;
}

/** 日志事件数据 */
export interface LogEventData {
  level: string;
  message: string;
}

class DownloadService {
  private eventListeners = new Map<string, UnlistenFn[]>();

  /** 开始下载（后端按 URL 类型自动分派引擎并构建参数） */
  startDownload(taskId: string): Promise<void> {
    return invokeTauri("start_download", { taskId });
  }

  stopDownload(taskId: string): Promise<void> {
    return invokeTauri("stop_download", { taskId });
  }

  pauseDownload(taskId: string): Promise<void> {
    return invokeTauri("pause_download", { taskId });
  }

  resumeDownload(taskId: string): Promise<void> {
    return invokeTauri("resume_download", { taskId });
  }

  /** 解析 URL 获取流信息（引擎自动分派：流媒体走 RE，直链走 ffprobe） */
  parseUrl(url: string): Promise<StreamInfo> {
    return invokeTauri<StreamInfo>("parse_url", { url });
  }

  /** 检测 URL 类型（后端权威检测） */
  detectUrlType(url: string): Promise<UrlType> {
    return invokeTauri<UrlType>("detect_url_type", { url });
  }

  getFileInfo(path: string): Promise<FileInfo> {
    return invokeTauri<FileInfo>("get_file_info", { path });
  }

  analyzeMediaFile(filePath: string): Promise<MediaInfo> {
    return invokeTauri<MediaInfo>("analyze_media_file", { filePath });
  }

  /**
   * 订阅任务的全部下载事件（进度/状态/日志/完成/错误）
   * @returns 取消订阅函数
   */
  async subscribeToTask(
    taskId: string,
    callback: (event: DownloadEvent) => void,
  ): Promise<UnlistenFn> {
    const unlisteners: UnlistenFn[] = [];

    unlisteners.push(
      await subscribeToEvent<ProgressData>(
        `download:progress:${taskId}`,
        (data) => callback({ type: "progress", taskId, data }),
      ),
      await subscribeToEvent<StatusEventData>(
        `download:status:${taskId}`,
        (data) => callback({ type: "status", taskId, data }),
      ),
      await subscribeToEvent<LogEventData>(`download:log:${taskId}`, (data) =>
        callback({ type: "log", taskId, data }),
      ),
      await subscribeToEvent<{ outputPath: string | null }>(
        `download:complete:${taskId}`,
        (data) => callback({ type: "complete", taskId, data }),
      ),
      await subscribeToEvent<{ message: string }>(
        `download:error:${taskId}`,
        (data) => callback({ type: "error", taskId, data }),
      ),
    );

    this.eventListeners.set(taskId, unlisteners);
    return () => this.unsubscribeFromTask(taskId);
  }

  unsubscribeFromTask(taskId: string): void {
    const unlisteners = this.eventListeners.get(taskId);
    if (unlisteners) {
      unlisteners.forEach((unlisten) => unlisten());
      this.eventListeners.delete(taskId);
    }
  }

  unsubscribeFromAll(): void {
    this.eventListeners.forEach((unlisteners) =>
      unlisteners.forEach((unlisten) => unlisten()),
    );
    this.eventListeners.clear();
  }
}

export const downloadService = new DownloadService();
export type { UnlistenFn };
