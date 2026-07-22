/**
 * 任务服务
 *
 * 与后端 tasks 命令组一一对应；进度落盘的防抖调度也在此（Store 不直接管定时器）
 */

import { invokeTauri } from "./tauri";
import { extractFileName } from "@/utils/format";
import type {
  MediaInfo,
  ProgressData,
  ProgressSample,
  TaskOverrides,
  TaskRecord,
  TaskStatus,
} from "@/domain";

/** 进度落盘防抖间隔 */
const PROGRESS_FLUSH_DELAY_MS = 1000;

class TaskService {
  /** 每任务的进度落盘防抖定时器 */
  private flushTimers = new Map<string, ReturnType<typeof setTimeout>>();
  /** 防抖窗口内最新的进度数据 */
  private pendingProgress = new Map<string, ProgressData>();

  // ===== CRUD =====

  loadAllTasks(): Promise<TaskRecord[]> {
    return invokeTauri<TaskRecord[]>("load_all_tasks");
  }

  loadRecoverableTasks(): Promise<TaskRecord[]> {
    return invokeTauri<TaskRecord[]>("load_recoverable_tasks");
  }

  getTask(taskId: string): Promise<TaskRecord | null> {
    return invokeTauri<TaskRecord | null>("get_task", { taskId });
  }

  /**
   * 创建任务（服务边界规范化）
   *
   * 确保 fileName 非空（从 URL 提取兜底）、saveDir 无多余空白。
   */
  createTask(task: TaskRecord): Promise<void> {
    const normalized: TaskRecord = {
      ...task,
      fileName: task.fileName.trim() || extractFileName(task.url),
      saveDir: task.saveDir.trim(),
    };
    return invokeTauri("create_task", { task: normalized });
  }

  updateTaskStatus(
    taskId: string,
    status: TaskStatus,
    error?: string | null,
  ): Promise<void> {
    return invokeTauri("update_task_status", {
      taskId,
      status,
      error: error ?? null,
    });
  }

  updateTaskOutputPath(taskId: string, outputPath: string): Promise<void> {
    return invokeTauri("update_task_output_path", { taskId, outputPath });
  }

  updateTaskMediaInfo(taskId: string, mediaInfo: MediaInfo): Promise<void> {
    return invokeTauri("update_task_media_info", { taskId, mediaInfo });
  }

  saveTaskOverrides(taskId: string, overrides: TaskOverrides): Promise<void> {
    return invokeTauri("save_task_overrides", { taskId, overrides });
  }

  deleteTask(taskId: string): Promise<void> {
    return invokeTauri("delete_task", { taskId });
  }

  /** 清除已结束任务，返回删除数量（历史记录保留） */
  clearFinishedTasks(): Promise<number> {
    return invokeTauri<number>("clear_finished_tasks");
  }

  clearAllTasks(): Promise<void> {
    return invokeTauri("clear_all_tasks");
  }

  /** 将活跃任务标记为已中断（应用启动时调用） */
  markActiveTasksInterrupted(): Promise<number> {
    return invokeTauri<number>("mark_active_tasks_interrupted");
  }

  // ===== 进度 =====

  /** 立即写入进度（不经过防抖） */
  updateTaskProgress(taskId: string, progress: ProgressData): Promise<void> {
    return invokeTauri("update_task_progress", { taskId, progress });
  }

  /**
   * 调度进度落盘（按任务防抖）
   *
   * 高频进度事件先更新内存状态（Store 负责），经此方法防抖写库；
   * 任务结束时调用 flushProgress 立即落盘。
   */
  scheduleProgressFlush(taskId: string, progress: ProgressData): void {
    this.pendingProgress.set(taskId, progress);

    const existing = this.flushTimers.get(taskId);
    if (existing) {
      clearTimeout(existing);
    }

    const timer = setTimeout(() => {
      this.flushTimers.delete(taskId);
      const data = this.pendingProgress.get(taskId);
      this.pendingProgress.delete(taskId);
      if (data) {
        this.updateTaskProgress(taskId, data).catch((e) =>
          console.error("进度落盘失败:", e),
        );
      }
    }, PROGRESS_FLUSH_DELAY_MS);

    this.flushTimers.set(taskId, timer);
  }

  /** 立即落盘并清除防抖窗口（任务结束时调用） */
  flushProgress(taskId: string): void {
    const timer = this.flushTimers.get(taskId);
    if (timer) {
      clearTimeout(timer);
      this.flushTimers.delete(taskId);
    }
    const data = this.pendingProgress.get(taskId);
    this.pendingProgress.delete(taskId);
    if (data) {
      this.updateTaskProgress(taskId, data).catch((e) =>
        console.error("进度落盘失败:", e),
      );
    }
  }

  // ===== 速率曲线 =====

  getProgressHistory(
    taskId: string,
    limit?: number,
  ): Promise<ProgressSample[]> {
    return invokeTauri<ProgressSample[]>("get_progress_history", {
      taskId,
      limit: limit ?? null,
    });
  }

  clearProgressHistory(taskId: string): Promise<void> {
    return invokeTauri("clear_progress_history", { taskId });
  }
}

export const taskService = new TaskService();
