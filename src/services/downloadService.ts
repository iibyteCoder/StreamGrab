/**
 * 下载服务
 * 封装下载相关的业务逻辑
 */

import { invokeTauri, subscribeToEvent, type UnlistenFn } from "./tauri";
import { buildCommandArgs, buildParseArgs } from "@/utils/commandBuilder";
import type {
  DownloadTask,
  TaskConfig,
  AppSettings,
  StreamInfo,
  TaskStatus,
  UrlType,
} from "@/types";
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
  /** 单流进度百分比 */
  percent: number;
  /** 总体进度百分比（推荐使用） */
  overallPercent?: number;
  speed: number;
  downloadedSize: number;
  totalSize: number;
  eta: number;
  /** 总已下载分片数 */
  totalDownloadedSegments?: number;
  /** 总分片数 */
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
 * 下载服务类
 */
class DownloadService {
  private eventListeners: Map<string, UnlistenFn[]> = new Map();

  /**
   * 检测 URL 类型
   * @param url 视频 URL
   */
  detectUrlType(url: string): UrlType {
    return detectUrlType(url);
  }

  /**
   * 开始下载（自动检测 URL 类型并选择下载方式）
   * @param task 下载任务
   * @param config 任务配置
   * @param settings 应用设置
   */
  async startDownload(
    task: DownloadTask,
    config: TaskConfig,
    settings: AppSettings,
  ): Promise<void> {
    const urlType = this.detectUrlType(task.url);

    // 如果是 HTTP 直链视频，使用 ffmpeg 下载
    if (needsFfmpeg(urlType)) {
      return this.startHttpVideoDownload(task, config, settings);
    }

    // 如果不是流媒体格式，返回错误
    if (!isStreamingType(urlType)) {
      throw new Error(
        "不支持的 URL 格式。请输入 M3U8、DASH、MSS 流媒体链接或 HTTP 直链视频。",
      );
    }

    // 使用 N_m3u8DL-RE 下载流媒体
    return this.startStreamDownload(task, config, settings);
  }

  /**
   * 使用 N_m3u8DL-RE 下载流媒体
   */
  private async startStreamDownload(
    task: DownloadTask,
    config: TaskConfig,
    settings: AppSettings,
  ): Promise<void> {
    // 构建命令行参数
    const args = buildCommandArgs(task.url, config, settings);

    // 调用 Tauri 命令启动下载
    await invokeTauri("start_download", {
      taskId: task.id,
      url: task.url,
      args,
      saveDir: config.saveDir || settings.general.saveDir,
      saveName: config.saveName,
      programPath: settings.advanced.n_m3u8dlPath || null,
    });
  }

  /**
   * 使用 FFmpeg 下载 HTTP 直链视频
   */
  private async startHttpVideoDownload(
    task: DownloadTask,
    config: TaskConfig,
    settings: AppSettings,
  ): Promise<void> {
    const saveName = config.saveName || task.fileName || "video.mp4";

    // 确保文件名有扩展名
    const finalSaveName = saveName.includes(".") ? saveName : `${saveName}.mp4`;

    await invokeTauri("start_http_video_download", {
      taskId: task.id,
      url: task.url,
      saveDir: config.saveDir || settings.general.saveDir,
      saveName: finalSaveName,
      ffmpegPath: settings.advanced.ffmpegPath || null,
    });
  }

  /**
   * 停止下载
   * @param taskId 任务 ID
   */
  async stopDownload(taskId: string): Promise<void> {
    await invokeTauri("stop_download", { task_id: taskId });
  }

  /**
   * 暂停下载
   * @param taskId 任务 ID
   */
  async pauseDownload(taskId: string): Promise<void> {
    await invokeTauri("pause_download", { task_id: taskId });
  }

  /**
   * 继续下载
   * @param taskId 任务 ID
   */
  async resumeDownload(taskId: string): Promise<void> {
    await invokeTauri("resume_download", { task_id: taskId });
  }

  /**
   * 解析 URL 获取流信息
   * 复用应用设置中的网络、解密等配置
   * @param url 视频 URL
   * @param settings 应用设置
   */
  async parseUrl(url: string, settings: AppSettings): Promise<StreamInfo> {
    const urlType = this.detectUrlType(url);

    // 如果是 HTTP 直链视频，使用 ffprobe 解析
    if (needsFfmpeg(urlType)) {
      return this.parseHttpVideoUrl(url, settings);
    }

    // 如果不是流媒体格式，返回错误
    if (!isStreamingType(urlType)) {
      throw new Error(
        "不支持的 URL 格式。请输入 M3U8、DASH 或 MSS 流媒体链接。",
      );
    }

    // 生成解析 ID
    const parseId = `parse_${Date.now()}`;
    const tempDir = "streamgrab_parse"; // 后端会拼接完整路径

    // 使用统一的参数构建函数，复用所有相关配置
    const args = buildParseArgs(url, settings, parseId, tempDir);

    return await invokeTauri<StreamInfo>("parse_url", {
      args,
      programPath: settings.advanced.n_m3u8dlPath || null,
      ffmpegPath: settings.advanced.ffmpegPath || null,
    });
  }

  /**
   * 使用 ffprobe 解析 HTTP 直链视频
   */
  private async parseHttpVideoUrl(
    url: string,
    settings: AppSettings,
  ): Promise<StreamInfo> {
    return await invokeTauri<StreamInfo>("parse_url", {
      args: [url], // 对于 HTTP 视频，只需要 URL
      programPath: null,
      ffmpegPath: settings.advanced.ffmpegPath || null,
    });
  }

  /**
   * 订阅任务事件
   * @param taskId 任务 ID
   * @param callback 事件回调
   */
  async subscribeToTask(
    taskId: string,
    callback: (event: DownloadEvent) => void,
  ): Promise<UnlistenFn> {
    const unlisteners: UnlistenFn[] = [];

    // 订阅进度事件
    const unlistenProgress = await subscribeToEvent<ProgressEventData>(
      `download:progress:${taskId}`,
      (data) => {
        callback({
          type: "progress",
          taskId,
          data,
        });
      },
    );
    unlisteners.push(unlistenProgress);

    // 订阅状态事件
    const unlistenStatus = await subscribeToEvent<StatusEventData>(
      `download:status:${taskId}`,
      (data) => {
        callback({
          type: "status",
          taskId,
          data,
        });
      },
    );
    unlisteners.push(unlistenStatus);

    // 订阅错误事件
    const unlistenError = await subscribeToEvent<{ message: string }>(
      `download:error:${taskId}`,
      (data) => {
        callback({
          type: "error",
          taskId,
          data,
        });
      },
    );
    unlisteners.push(unlistenError);

    // 订阅完成事件
    const unlistenComplete = await subscribeToEvent<{ outputPath: string }>(
      `download:complete:${taskId}`,
      (data) => {
        callback({
          type: "complete",
          taskId,
          data,
        });
      },
    );
    unlisteners.push(unlistenComplete);

    // 订阅日志事件
    const unlistenLog = await subscribeToEvent<LogEventData>(
      `download:log:${taskId}`,
      (data) => {
        callback({
          type: "log",
          taskId,
          data,
        });
      },
    );
    unlisteners.push(unlistenLog);

    // 保存 unlisteners 以便清理
    this.eventListeners.set(taskId, unlisteners);

    // 返回一个取消所有订阅的函数
    return () => {
      this.unsubscribeFromTask(taskId);
    };
  }

  /**
   * 取消任务的所有事件订阅
   * @param taskId 任务 ID
   */
  unsubscribeFromTask(taskId: string): void {
    const unlisteners = this.eventListeners.get(taskId);
    if (unlisteners) {
      unlisteners.forEach((unlisten) => unlisten());
      this.eventListeners.delete(taskId);
    }
  }

  /**
   * 取消所有事件订阅
   */
  unsubscribeFromAll(): void {
    this.eventListeners.forEach((unlisteners) => {
      unlisteners.forEach((unlisten) => unlisten());
    });
    this.eventListeners.clear();
  }

  /**
   * 获取 N_m3u8DL-RE 版本
   */
  async getDownloaderVersion(): Promise<string> {
    return await invokeTauri<string>("get_n_m3u8dl_version", {});
  }

  /**
   * 检查 N_m3u8DL-RE 是否可用
   */
  async checkDownloaderAvailable(): Promise<boolean> {
    try {
      await this.getDownloaderVersion();
      return true;
    } catch {
      return false;
    }
  }
}

// 导出单例
export const downloadService = new DownloadService();

// 导出类型
export type { UnlistenFn };
