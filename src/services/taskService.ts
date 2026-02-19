/**
 * 任务持久化服务
 * 处理所有任务相关的后端 API 调用
 */

import { invokeTauri } from "./tauri";
import type { DownloadTask, TaskStatus, TaskCreateParams } from "@/types/task";

/**
 * 后端任务记录格式（与 Rust FullTaskRecord 匹配）
 */
interface FullTaskRecord {
  id: string;
  url: string;
  file_name: string;
  save_dir: string;
  output_path: string | null;
  status: string;
  error: string | null;
  was_interrupted: boolean;
  created_at: string;
  updated_at: string;
  started_at: string | null;
  completed_at: string | null;
  // 进度
  progress_percent: number;
  progress_speed: number;
  progress_downloaded_size: number;
  progress_total_size: number;
  progress_downloaded_segments: number;
  progress_total_segments: number;
  progress_eta: number;
  progress_current_action: string;
  // 媒体信息
  media_resolution: string | null;
  media_width: number | null;
  media_height: number | null;
  media_frame_rate: number | null;
  media_video_codec: string | null;
  media_video_range: string | null;
  media_audio_codec: string | null;
  media_audio_channels: string | null;
  media_audio_language: string | null;
  media_duration: number | null;
  media_segment_count: number | null;
  media_is_live: boolean;
  media_is_encrypted: boolean;
  media_file_format: string | null;
}

/**
 * 后端 TaskRecord 格式（用于创建任务）
 */
interface TaskRecord {
  id: string;
  url: string;
  file_name: string;
  save_dir: string;
  output_path: string | null;
  status: string;
  error: string | null;
  was_interrupted: boolean;
  created_at: string;
  updated_at: string;
  started_at: string | null;
  completed_at: string | null;
}

/**
 * 后端 TaskMediaInfo 格式
 */
interface TaskMediaInfo {
  task_id: string;
  resolution?: string;
  width?: number;
  height?: number;
  frame_rate?: number;
  video_codec?: string;
  video_range?: string;
  audio_codec?: string;
  audio_channels?: string;
  audio_language?: string;
  duration?: number;
  segment_count?: number;
  is_live: boolean;
  is_encrypted: boolean;
  file_format?: string;
}

/**
 * 进度历史记录（与 Rust ProgressHistoryRecord 匹配）
 */
export interface ProgressHistoryRecord {
  id: number;
  task_id: string;
  timestamp: string;
  percent: number;
  speed: number;
  downloaded_size: number;
}

/**
 * 任务服务类
 */
class TaskService {
  /**
   * 加载所有任务
   */
  async loadAllTasks(): Promise<DownloadTask[]> {
    const records = await invokeTauri<FullTaskRecord[]>("load_all_tasks");
    return records.map((r) => this.toDownloadTask(r));
  }

  /**
   * 加载可恢复的任务（被中断的下载）
   */
  async loadRecoverableTasks(): Promise<DownloadTask[]> {
    const records = await invokeTauri<FullTaskRecord[]>(
      "load_recoverable_tasks",
    );
    return records.map((r) => this.toDownloadTask(r));
  }

  /**
   * 创建任务
   */
  async createTask(params: TaskCreateParams): Promise<void> {
    const record: TaskRecord = {
      id: params.id,
      url: params.url,
      file_name: params.fileName,
      save_dir: params.saveDir,
      output_path: params.outputPath || null,
      status: params.status,
      error: params.error || null,
      was_interrupted: params.wasInterrupted || false,
      created_at: params.createdAt || new Date().toISOString(),
      updated_at: params.updatedAt || new Date().toISOString(),
      started_at: params.startedAt || null,
      completed_at: params.completedAt || null,
    };
    await invokeTauri("create_task", { task: record });
  }

  /**
   * 更新任务状态
   */
  async updateTaskStatus(
    taskId: string,
    status: TaskStatus,
    error?: string,
  ): Promise<void> {
    await invokeTauri("update_task_status", {
      taskId,
      status,
      error: error || null,
    });
  }

  /**
   * 更新任务输出路径
   */
  async updateTaskOutputPath(
    taskId: string,
    outputPath: string,
  ): Promise<void> {
    await invokeTauri("update_task_output_path", {
      taskId,
      outputPath,
    });
  }

  /**
   * 更新任务进度
   */
  async updateTaskProgress(
    taskId: string,
    progress: {
      percent: number;
      speed: number;
      downloadedSize: number;
      totalSize: number;
      downloadedSegments: number;
      totalSegments: number;
      eta: number;
      currentAction: string;
    },
  ): Promise<void> {
    await invokeTauri("update_task_progress", {
      taskId,
      percent: progress.percent,
      speed: progress.speed,
      downloadedSize: progress.downloadedSize,
      totalSize: progress.totalSize,
      downloadedSegments: progress.downloadedSegments,
      totalSegments: progress.totalSegments,
      eta: progress.eta,
      currentAction: progress.currentAction,
    });
  }

  /**
   * 更新任务媒体信息
   */
  async updateTaskMediaInfo(
    taskId: string,
    mediaInfo: {
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
      segmentCount?: number;
      isLive?: boolean;
      isEncrypted?: boolean;
      fileFormat?: string;
    },
  ): Promise<void> {
    const info: TaskMediaInfo = {
      task_id: taskId,
      resolution: mediaInfo.resolution,
      width: mediaInfo.width,
      height: mediaInfo.height,
      frame_rate: mediaInfo.frameRate,
      video_codec: mediaInfo.videoCodec,
      video_range: mediaInfo.videoRange,
      audio_codec: mediaInfo.audioCodec,
      audio_channels: mediaInfo.audioChannels,
      audio_language: mediaInfo.audioLanguage,
      duration: mediaInfo.duration,
      segment_count: mediaInfo.segmentCount,
      is_live: mediaInfo.isLive || false,
      is_encrypted: mediaInfo.isEncrypted || false,
      file_format: mediaInfo.fileFormat,
    };
    await invokeTauri("update_task_media_info", { taskId, mediaInfo: info });
  }

  /**
   * 删除任务
   */
  async deleteTask(taskId: string): Promise<void> {
    await invokeTauri("delete_task", { taskId });
  }

  /**
   * 清除已完成的任务
   */
  async clearFinishedTasks(): Promise<number> {
    return invokeTauri<number>("clear_finished_tasks");
  }

  /**
   * 标记活跃任务为已中断
   */
  async markActiveTasksInterrupted(): Promise<number> {
    return invokeTauri<number>("mark_active_tasks_interrupted");
  }

  /**
   * 清除所有任务
   */
  async clearAllTasks(): Promise<void> {
    await invokeTauri("clear_all_tasks");
  }

  // ========================================
  // 进度历史
  // ========================================

  /**
   * 获取进度历史
   */
  async getProgressHistory(
    taskId: string,
    limit?: number,
  ): Promise<ProgressHistoryRecord[]> {
    return invokeTauri<ProgressHistoryRecord[]>("get_progress_history", {
      taskId,
      limit,
    });
  }

  /**
   * 保存进度历史
   */
  async saveProgressHistory(
    taskId: string,
    percent: number,
    speed: number,
    downloadedSize: number,
  ): Promise<void> {
    await invokeTauri("save_progress_history", {
      taskId,
      percent,
      speed,
      downloadedSize,
    });
  }

  /**
   * 清除进度历史
   */
  async clearProgressHistory(taskId: string): Promise<void> {
    await invokeTauri("clear_progress_history", { taskId });
  }

  /**
   * 将 FullTaskRecord 转换为 DownloadTask（直接字段映射）
   */
  toDownloadTask(record: FullTaskRecord): DownloadTask {
    return {
      // 基本信息
      id: record.id,
      url: record.url,
      fileName: record.file_name,
      saveDir: record.save_dir,
      outputPath: record.output_path || undefined,
      status: record.status as TaskStatus,
      error: record.error || undefined,
      wasInterrupted: record.was_interrupted,
      // 时间戳
      createdAt: new Date(record.created_at),
      updatedAt: new Date(record.updated_at),
      startedAt: record.started_at ? new Date(record.started_at) : undefined,
      completedAt: record.completed_at
        ? new Date(record.completed_at)
        : undefined,
      // 进度（扁平化）
      progressPercent: record.progress_percent,
      progressSpeed: record.progress_speed,
      progressDownloadedSize: record.progress_downloaded_size,
      progressTotalSize: record.progress_total_size,
      progressDownloadedSegments: record.progress_downloaded_segments,
      progressTotalSegments: record.progress_total_segments,
      progressEta: record.progress_eta,
      progressCurrentAction: record.progress_current_action,
      // 媒体信息（扁平化）
      mediaResolution: record.media_resolution || undefined,
      mediaWidth: record.media_width || undefined,
      mediaHeight: record.media_height || undefined,
      mediaFrameRate: record.media_frame_rate || undefined,
      mediaVideoCodec: record.media_video_codec || undefined,
      mediaVideoRange: record.media_video_range || undefined,
      mediaAudioCodec: record.media_audio_codec || undefined,
      mediaAudioChannels: record.media_audio_channels || undefined,
      mediaAudioLanguage: record.media_audio_language || undefined,
      mediaDuration: record.media_duration || undefined,
      mediaSegmentCount: record.media_segment_count || undefined,
      mediaIsLive: record.media_is_live,
      mediaIsEncrypted: record.media_is_encrypted,
      mediaFileFormat: record.media_file_format || undefined,
    };
  }
}

export const taskService = new TaskService();
