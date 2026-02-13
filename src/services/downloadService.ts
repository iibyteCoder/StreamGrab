/**
 * 下载服务
 * 封装下载相关的业务逻辑
 */

import { invokeTauri, subscribeToEvent, type UnlistenFn } from './tauri';
import { buildCommandArgs, buildMuxImportArgs, buildKeyArgs } from '@/utils/commandBuilder';
import type { DownloadTask, TaskConfig, AppSettings, StreamInfo, TaskProgress, TaskStatus } from '@/types';

/**
 * 下载事件类型
 */
export type DownloadEventType =
  | 'progress'
  | 'status'
  | 'error'
  | 'complete'
  | 'log';

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
  speed: number;
  downloadedSize: number;
  totalSize: number;
  eta: number;
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
  level: 'info' | 'warn' | 'error' | 'debug';
  message: string;
}

/**
 * 下载服务类
 */
class DownloadService {
  private eventListeners: Map<string, UnlistenFn[]> = new Map();

  /**
   * 开始下载
   * @param task 下载任务
   * @param config 任务配置
   * @param settings 应用设置
   */
  async startDownload(
    task: DownloadTask,
    config: TaskConfig,
    settings: AppSettings
  ): Promise<void> {
    // 构建命令行参数
    const args = buildCommandArgs(task.url, config, settings);

    // 调用 Tauri 命令启动下载
    // 使用 camelCase 参数名与 Rust 后端 serde rename 匹配
    await invokeTauri('start_download', {
      taskId: task.id,
      url: task.url,
      args,
      saveDir: config.saveDir || settings.general.saveDir,
      saveName: config.saveName,
      programPath: settings.advanced.n_m3u8dlPath || null,
    });
  }

  /**
   * 停止下载
   * @param taskId 任务 ID
   */
  async stopDownload(taskId: string): Promise<void> {
    await invokeTauri('stop_download', { task_id: taskId });
  }

  /**
   * 暂停下载
   * @param taskId 任务 ID
   */
  async pauseDownload(taskId: string): Promise<void> {
    await invokeTauri('pause_download', { task_id: taskId });
  }

  /**
   * 继续下载
   * @param taskId 任务 ID
   */
  async resumeDownload(taskId: string): Promise<void> {
    await invokeTauri('resume_download', { task_id: taskId });
  }

  /**
   * 解析 URL 获取流信息
   * @param url 视频 URL
   * @param settings 应用设置（用于代理等）
   */
  async parseUrl(url: string, settings: AppSettings): Promise<StreamInfo> {
    return await invokeTauri<StreamInfo>('parse_url', {
      url,
      useProxy: settings.network.useSystemProxy,
      customProxy: settings.network.customProxy,
      headers: settings.network.headers.filter((h) => h.enabled),
      programPath: settings.advanced.n_m3u8dlPath || null,
    });
  }

  /**
   * 订阅任务事件
   * @param taskId 任务 ID
   * @param callback 事件回调
   */
  async subscribeToTask(
    taskId: string,
    callback: (event: DownloadEvent) => void
  ): Promise<UnlistenFn> {
    const unlisteners: UnlistenFn[] = [];

    // 订阅进度事件
    const unlistenProgress = await subscribeToEvent<ProgressEventData>(
      `download:progress:${taskId}`,
      (data) => {
        callback({
          type: 'progress',
          taskId,
          data,
        });
      }
    );
    unlisteners.push(unlistenProgress);

    // 订阅状态事件
    const unlistenStatus = await subscribeToEvent<StatusEventData>(
      `download:status:${taskId}`,
      (data) => {
        callback({
          type: 'status',
          taskId,
          data,
        });
      }
    );
    unlisteners.push(unlistenStatus);

    // 订阅错误事件
    const unlistenError = await subscribeToEvent<{ message: string }>(
      `download:error:${taskId}`,
      (data) => {
        callback({
          type: 'error',
          taskId,
          data,
        });
      }
    );
    unlisteners.push(unlistenError);

    // 订阅完成事件
    const unlistenComplete = await subscribeToEvent<{ outputPath: string }>(
      `download:complete:${taskId}`,
      (data) => {
        callback({
          type: 'complete',
          taskId,
          data,
        });
      }
    );
    unlisteners.push(unlistenComplete);

    // 订阅日志事件
    const unlistenLog = await subscribeToEvent<LogEventData>(
      `download:log:${taskId}`,
      (data) => {
        callback({
          type: 'log',
          taskId,
          data,
        });
      }
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
    return await invokeTauri<string>('get_n_m3u8dl_version', {});
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
