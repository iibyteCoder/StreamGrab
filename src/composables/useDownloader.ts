/**
 * 下载器组合式函数
 *
 * 核心下载逻辑：启动/停止/暂停/恢复，并发控制，定时调度器。
 * 进度事件载荷直接使用后端 ProgressData（camelCase），不做字段转换。
 */

import { ref, onUnmounted } from "vue";
import { useTaskStore, useSettingsStore } from "@/stores";
import {
  downloadService,
  taskService,
  type DownloadEvent,
  type StatusEventData,
  type LogEventData,
  type UnlistenFn,
} from "@/services";
import { useToast } from "./useToast";
import { useNotification } from "./useNotification";
import type { DownloadTask, ProgressData, StreamInfo } from "@/domain";
import { MAX_CONCURRENT_TASKS } from "@/utils/constants";
import { i18n } from "@/locales";

/** 定时调度器轮询间隔（30 秒） */
const SCHEDULE_TICK_INTERVAL = 30_000;

export function useDownloader() {
  const taskStore = useTaskStore();
  const settingsStore = useSettingsStore();
  const toast = useToast();
  const notification = useNotification();

  const startingTasks = ref<Set<string>>(new Set());
  const unlisteners = new Map<string, UnlistenFn>();
  const isParsing = ref(false);
  const parsedStreamInfo = ref<StreamInfo | null>(null);

  let isProcessingQueue = false;
  let scheduleTimer: ReturnType<typeof setInterval> | null = null;

  // ==========================================
  // 队列处理
  // ==========================================

  const processQueue = async (): Promise<void> => {
    if (isProcessingQueue) return;
    isProcessingQueue = true;

    try {
      while (
        taskStore.activeTasks.length < MAX_CONCURRENT_TASKS &&
        taskStore.pendingTasks.length > 0
      ) {
        const nextTask = taskStore.pendingTasks[0];
        if (nextTask && !startingTasks.value.has(nextTask.id)) {
          await startDownload(nextTask);
        } else {
          break;
        }
      }
    } finally {
      isProcessingQueue = false;
    }
  };

  // ==========================================
  // 下载控制
  // ==========================================

  /** 启动下载（内部使用 TaskRecord；对外保持兼容） */
  const startDownload = async (task: DownloadTask): Promise<void> => {
    if (startingTasks.value.has(task.id)) return;

    if (taskStore.activeTasks.length >= MAX_CONCURRENT_TASKS) {
      await taskStore.setTaskStatus(task.id, "pending");
      return;
    }

    startingTasks.value.add(task.id);

    try {
      await taskStore.setTaskStatus(task.id, "analyzing");

      // 订阅事件
      const unlisten = await downloadService.subscribeToTask(
        task.id,
        handleDownloadEvent,
      );
      unlisteners.set(task.id, unlisten);

      // 启动下载（参数完全由后端构建，前端只传 taskId）
      await downloadService.startDownload(task.id);

      await taskStore.setTaskStatus(task.id, "downloading");
      toast.success(
        i18n.global.t("messages.downloadStartedWithName", {
          name: task.fileName || task.url,
        }),
      );
    } catch (e) {
      console.error("Failed to start download:", e);
      const errorMessage =
        e instanceof Error
          ? e.message
          : typeof e === "string"
            ? e
            : i18n.global.t("messages.unknownError");
      await taskStore.setTaskStatus(task.id, "failed", errorMessage);
      toast.error(
        i18n.global.t("messages.downloadFailed") + ": " + errorMessage,
      );
      processQueue();
    } finally {
      startingTasks.value.delete(task.id);
    }
  };

  /** 按 taskId 启动下载（供队列和外部调用） */
  const startDownloadById = async (taskId: string): Promise<void> => {
    const task = taskStore.getTaskById(taskId);
    if (!task) return;
    await startDownload(task);
  };

  const stopDownload = async (taskId: string): Promise<void> => {
    try {
      await downloadService.stopDownload(taskId);
      taskService.flushProgress(taskId);
      await taskStore.setTaskStatus(taskId, "cancelled");
      unsubscribeTask(taskId);
      toast.info(i18n.global.t("messages.downloadCancelled"));
      processQueue();
    } catch (e) {
      console.error("Failed to stop download:", e);
      toast.error(
        i18n.global.t("messages.stopFailed", {
          error:
            e instanceof Error
              ? e.message
              : i18n.global.t("messages.unknownError"),
        }),
      );
    }
  };

  const pauseDownload = async (taskId: string): Promise<void> => {
    try {
      await downloadService.pauseDownload(taskId);
      await taskStore.setTaskStatus(taskId, "paused");
      unsubscribeTask(taskId);
      toast.info(i18n.global.t("messages.downloadPaused"));
      processQueue();
    } catch (e) {
      console.error("Failed to pause download:", e);
      toast.error(
        i18n.global.t("messages.pauseFailed", {
          error:
            e instanceof Error
              ? e.message
              : i18n.global.t("messages.unknownError"),
        }),
      );
    }
  };

  const resumeDownload = async (taskId: string): Promise<void> => {
    const task = taskStore.getTaskById(taskId);
    if (!task) return;
    try {
      await startDownload(task);
      toast.success(i18n.global.t("messages.downloadResumed"));
    } catch (e) {
      console.error("Failed to resume download:", e);
      toast.error(
        i18n.global.t("messages.resumeFailed", {
          error:
            e instanceof Error
              ? e.message
              : i18n.global.t("messages.unknownError"),
        }),
      );
    }
  };

  const cancelDownload = async (taskId: string): Promise<void> => {
    await stopDownload(taskId);
  };

  // ==========================================
  // URL 解析
  // ==========================================

  const parseUrl = async (url: string): Promise<StreamInfo | null> => {
    isParsing.value = true;
    parsedStreamInfo.value = null;

    try {
      const info = await downloadService.parseUrl(url);
      parsedStreamInfo.value = info;
      return info;
    } catch (e) {
      console.error("Failed to parse URL:", e);
      toast.error(
        i18n.global.t("messages.parseFailed", {
          error:
            e instanceof Error
              ? e.message
              : i18n.global.t("messages.unknownError"),
        }),
      );
      return null;
    } finally {
      isParsing.value = false;
    }
  };

  // ==========================================
  // 媒体文件分析
  // ==========================================

  const analyzeAndUpdateMediaInfo = async (
    taskId: string,
    filePath: string,
  ): Promise<void> => {
    try {
      const result = await downloadService.analyzeMediaFile(filePath);
      await taskStore.setTaskMediaInfo(taskId, result);
    } catch (e) {
      console.warn("Failed to analyze media file:", e);
    }
  };

  // ==========================================
  // 事件处理
  // ==========================================

  const handleDownloadEvent = (event: DownloadEvent): void => {
    const { type, taskId, data } = event;

    switch (type) {
      case "progress": {
        taskStore.setTaskProgress(taskId, data as ProgressData);
        break;
      }

      case "status": {
        const statusData = data as StatusEventData;
        const action = statusData.action;
        // 只处理中间状态变更；完成由 complete 事件驱动
        if (action === "merging" || action === "muxing") {
          taskStore.setTaskStatus(taskId, action);
        }
        break;
      }

      case "error": {
        const errorData = data as { message: string };
        const task = taskStore.getTaskById(taskId);
        taskStore.setTaskStatus(taskId, "failed", errorData.message);
        taskService.flushProgress(taskId);
        toast.error(
          i18n.global.t("messages.downloadError", {
            error: errorData.message,
          }),
        );
        if (task) {
          notification.sendDownloadErrorNotification(
            task.fileName || task.url,
            errorData.message,
          );
        }
        unsubscribeTask(taskId);
        processQueue();
        break;
      }

      case "complete": {
        const completeData = data as { outputPath: string | null };
        const task = taskStore.getTaskById(taskId);

        taskService.flushProgress(taskId);

        if (completeData.outputPath) {
          taskStore.setTaskOutputPath(taskId, completeData.outputPath);
        }

        taskStore.setTaskStatus(taskId, "completed");
        toast.success(i18n.global.t("messages.downloadCompleted") + "!");

        if (task) {
          notification.sendDownloadCompleteNotification(
            task.fileName || i18n.global.t("messages.fileFallback"),
          );
        }

        if (completeData.outputPath) {
          analyzeAndUpdateMediaInfo(taskId, completeData.outputPath).catch(
            (e) => console.warn("Failed to analyze media file:", e),
          );
        }

        unsubscribeTask(taskId);
        processQueue();
        break;
      }

      case "log": {
        const logData = data as LogEventData;
        taskStore.addTaskLog(taskId, logData.level || "info", logData.message);
        break;
      }
    }
  };

  // ==========================================
  // 订阅管理
  // ==========================================

  const unsubscribeTask = (taskId: string): void => {
    const unlisten = unlisteners.get(taskId);
    if (unlisten) {
      unlisten();
      unlisteners.delete(taskId);
    }
  };

  // ==========================================
  // 便捷方法
  // ==========================================

  const startPendingTasks = async (): Promise<void> => {
    await processQueue();
  };

  const addAndStartTask = async (
    url: string,
    fileName?: string,
    saveDir?: string,
    overrides?: import("@/domain").TaskOverrides,
  ): Promise<{ task: DownloadTask; wasRenamed: boolean }> => {
    const result = await taskStore.addTask({
      url,
      fileName,
      saveDir,
      overrides,
    });

    if (settingsStore.appSettings.auto_start_download) {
      await processQueue();
    }

    return result;
  };

  // ==========================================
  // 定时调度器
  // ==========================================

  /** 扫描 pending 任务，启动已到期的定时任务 */
  const tickScheduledTasks = (): void => {
    const now = Date.now();
    for (const task of taskStore.pendingTasks) {
      const scheduledAt = task.overrides?.scheduledStartAt;
      if (scheduledAt) {
        const scheduledTime = new Date(scheduledAt).getTime();
        if (!isNaN(scheduledTime) && scheduledTime <= now) {
          startDownload(task);
        }
      }
    }
  };

  // 挂载时立即扫描一次 + 启动定时轮询
  tickScheduledTasks();
  scheduleTimer = setInterval(tickScheduledTasks, SCHEDULE_TICK_INTERVAL);

  // ==========================================
  // 清理
  // ==========================================

  const cleanup = (): void => {
    unlisteners.forEach((unlisten) => unlisten());
    unlisteners.clear();
    downloadService.unsubscribeFromAll();
    if (scheduleTimer) {
      clearInterval(scheduleTimer);
      scheduleTimer = null;
    }
  };

  onUnmounted(cleanup);

  return {
    // State
    startingTasks,
    isParsing,
    parsedStreamInfo,

    // Actions
    startDownload,
    startDownloadById,
    stopDownload,
    pauseDownload,
    resumeDownload,
    cancelDownload,
    parseUrl,
    startPendingTasks,
    addAndStartTask,
    processQueue,
    cleanup,
  };
}
