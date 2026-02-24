/**
 * 下载服务
 * 封装下载相关的业务逻辑
 */

import { invokeTauri, subscribeToEvent, type UnlistenFn } from "./tauri";
import { buildCommandArgs, buildParseArgs } from "@/utils/commandBuilder";
import type {
  DownloadTask,
  TaskConfig,
  StreamInfo,
  TaskStatus,
  UrlType,
} from "@/types";
import type { AllConfig } from "@/domain/config";
import { detectUrlType, needsFfmpeg, isStreamingType } from "@/types";

/**
 * 下载事件类型
 */
export type DownloadEventType =
  | "progress"
  | "status"
  | "error"
  | "complete"
  | "log";

/**
 * 下载事件
 */
export interface DownloadEvent {
  type: DownloadEventType;
  taskId: string;
  data: unknown;
}

/**
 * 进度事件数据
 */
export interface ProgressEventData {
  percent: number;
  overallPercent?: number;
  speed: number;
  downloadedSize: number;
  totalSize: number;
  eta: number;
  totalDownloadedSegments?: number;
  totalSegments?: number;
}

/**
 * 状态事件数据
 */
export interface StatusEventData {
  status: TaskStatus;
  message?: string;
}

/**
 * 日志事件数据
 */
export interface LogEventData {
  level: "info" | "warn" | "error" | "debug";
  message: string;
}

/**
 * 媒体文件分析结果
 */
export interface MediaFileAnalysisResult {
  resolution?: string;
  width?: number;
  height?: number;
  frameRate?: number;
  videoCodec?: string;
  videoRange?: string;
  audioCodec?: string;
  audioChannels?: string;
  audioLanguage?: string;
  duration?: number;
  fileSize?: number;
  bitRate?: number;
  format?: string;
}

/**
 * 下载服务类
 */
class DownloadService {
  private eventListeners: Map<string, UnlistenFn[]> = new Map();

  detectUrlType(url: string): UrlType {
    return detectUrlType(url);
  }

  async startDownload(
    task: DownloadTask,
    config: TaskConfig,
    allConfig: AllConfig,
  ): Promise<void> {
    const urlType = this.detectUrlType(task.url);

    if (needsFfmpeg(urlType)) {
      return this.startHttpVideoDownload(task, config, allConfig);
    }

    if (!isStreamingType(urlType)) {
      throw new Error(
        "不支持的 URL 格式。请输入 M3U8、DASH、MSS 流媒体链接或 HTTP 直链视频。",
      );
    }

    return this.startStreamDownload(task, config, allConfig);
  }

  private async startStreamDownload(
    task: DownloadTask,
    config: TaskConfig,
    allConfig: AllConfig,
  ): Promise<void> {
    const args = buildCommandArgs(task.url, config, allConfig);

    await invokeTauri("start_download", {
      taskId: task.id,
      url: task.url,
      args,
      saveDir: config.saveDir || allConfig.app.default_save_dir,
      saveName: config.saveName,
    });
  }

  private async startHttpVideoDownload(
    task: DownloadTask,
    config: TaskConfig,
    allConfig: AllConfig,
  ): Promise<void> {
    const saveName = config.saveName || task.fileName || "video.mp4";
    const finalSaveName = saveName.includes(".") ? saveName : `${saveName}.mp4`;

    await invokeTauri("start_http_video_download", {
      taskId: task.id,
      url: task.url,
      saveDir: config.saveDir || allConfig.app.default_save_dir,
      saveName: finalSaveName,
    });
  }

  async stopDownload(taskId: string): Promise<void> {
    await invokeTauri("stop_download", { taskId });
  }

  async pauseDownload(taskId: string): Promise<void> {
    await invokeTauri("pause_download", { taskId });
  }

  async resumeDownload(taskId: string): Promise<void> {
    await invokeTauri("resume_download", { taskId });
  }

  async parseUrl(url: string, allConfig: AllConfig): Promise<StreamInfo> {
    const urlType = this.detectUrlType(url);

    if (needsFfmpeg(urlType)) {
      return this.parseHttpVideoUrl(url);
    }

    if (!isStreamingType(urlType)) {
      throw new Error(
        "不支持的 URL 格式。请输入 M3U8、DASH 或 MSS 流媒体链接。",
      );
    }

    const parseId = `parse_${Date.now()}`;
    const tempDir = "streamgrab_parse";

    const args = buildParseArgs(url, allConfig, parseId, tempDir);

    return await invokeTauri<StreamInfo>("parse_url", { args });
  }

  private async parseHttpVideoUrl(url: string): Promise<StreamInfo> {
    return await invokeTauri<StreamInfo>("parse_url", { args: [url] });
  }

  async subscribeToTask(
    taskId: string,
    callback: (event: DownloadEvent) => void,
  ): Promise<UnlistenFn> {
    const unlisteners: UnlistenFn[] = [];

    const unlistenProgress = await subscribeToEvent<ProgressEventData>(
      `download:progress:${taskId}`,
      (data) => callback({ type: "progress", taskId, data }),
    );
    unlisteners.push(unlistenProgress);

    const unlistenStatus = await subscribeToEvent<StatusEventData>(
      `download:status:${taskId}`,
      (data) => callback({ type: "status", taskId, data }),
    );
    unlisteners.push(unlistenStatus);

    const unlistenError = await subscribeToEvent<{ message: string }>(
      `download:error:${taskId}`,
      (data) => callback({ type: "error", taskId, data }),
    );
    unlisteners.push(unlistenError);

    const unlistenComplete = await subscribeToEvent<{ outputPath: string }>(
      `download:complete:${taskId}`,
      (data) => callback({ type: "complete", taskId, data }),
    );
    unlisteners.push(unlistenComplete);

    const unlistenLog = await subscribeToEvent<LogEventData>(
      `download:log:${taskId}`,
      (data) => callback({ type: "log", taskId, data }),
    );
    unlisteners.push(unlistenLog);

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
    this.eventListeners.forEach((unlisteners) => {
      unlisteners.forEach((unlisten) => unlisten());
    });
    this.eventListeners.clear();
  }

  async getDownloaderVersion(): Promise<string> {
    return await invokeTauri<string>("get_n_m3u8dl_version", {});
  }

  async checkDownloaderAvailable(): Promise<boolean> {
    try {
      await this.getDownloaderVersion();
      return true;
    } catch {
      return false;
    }
  }

  async analyzeMediaFile(filePath: string): Promise<MediaFileAnalysisResult> {
    return await invokeTauri<MediaFileAnalysisResult>("analyze_media_file", {
      filePath,
    });
  }
}

export const downloadService = new DownloadService();
export type { UnlistenFn };
