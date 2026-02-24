/**
 * 下载器组合式函数
 * 封装下载的核心逻辑，包括启动、停止、暂停、恢复等
 * 支持并发控制，自动启动等待中的任务
 */

import { ref, onUnmounted } from "vue";
import { useTaskStore, useSettingsStore } from "@/stores";
import {
  downloadService,
  type DownloadEvent,
  type UnlistenFn,
} from "@/services";
import { useToast } from "./useToast";
import { useNotification } from "./useNotification";
import type { DownloadTask, TaskConfig, StreamInfo } from "@/types";

/**
 * 下载器组合式函数
 */
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

  const processQueue = async (): Promise<void> => {
    if (isProcessingQueue) return;
    isProcessingQueue = true;

    try {
      while (taskStore.canStartMore && taskStore.pendingTasks.length > 0) {
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

  /**
   * 构建任务配置
   */
  const buildTaskConfig = (
    task: DownloadTask,
    taskData?: DownloadTask,
  ): TaskConfig => {
    const { appSettings, m3u8dlSettings, networkSettings, networkHeaders } =
      settingsStore;

    return {
      saveDir:
        taskData?.saveDir || task.saveDir || appSettings.default_save_dir,
      saveName: task.fileName,
      threadCount: m3u8dlSettings.thread_count,
      retryCount: m3u8dlSettings.retry_count,
      timeout: m3u8dlSettings.timeout,
      maxSpeed: m3u8dlSettings.max_speed,
      autoSelect: m3u8dlSettings.auto_select,
      selectVideo: m3u8dlSettings.select_video ?? undefined,
      selectAudio: m3u8dlSettings.select_audio ?? undefined,
      selectSubtitle: m3u8dlSettings.select_subtitle ?? undefined,
      dropVideo: m3u8dlSettings.drop_video ?? undefined,
      dropAudio: m3u8dlSettings.drop_audio ?? undefined,
      dropSubtitle: m3u8dlSettings.drop_subtitle ?? undefined,
      muxFormat: m3u8dlSettings.mux_format,
      muxAfterDone: !m3u8dlSettings.skip_merge,
      skipMerge: m3u8dlSettings.skip_merge,
      delAfterDone: m3u8dlSettings.del_after_done,
      checkSegmentsCount: m3u8dlSettings.check_segments_count,
      headers: networkHeaders
        .filter((h) => h.enabled)
        .map((h) => ({
          key: h.name,
          value: h.value,
          enabled: h.enabled,
        })),
      proxy: networkSettings.custom_proxy ?? undefined,
    };
  };

  const startDownload = async (task: DownloadTask): Promise<void> => {
    if (startingTasks.value.has(task.id)) return;

    if (!taskStore.canStartMore) {
      taskStore.updateTaskStatus(task.id, "pending");
      return;
    }

    startingTasks.value.add(task.id);

    try {
      taskStore.updateTaskStatus(task.id, "analyzing");

      const taskData = taskStore.getTask(task.id);
      const config = buildTaskConfig(task, taskData);

      const unlisten = await downloadService.subscribeToTask(
        task.id,
        handleDownloadEvent,
      );
      unlisteners.set(task.id, unlisten);

      // 获取完整配置
      const allConfig = settingsStore.getAllConfig();
      await downloadService.startDownload(task, config, allConfig);

      taskStore.updateTaskStatus(task.id, "downloading");

      // 解析流信息
      try {
        const streamInfo = await downloadService.parseUrl(task.url, allConfig);
        const bestVideo = streamInfo.videos?.[0];
        const bestAudio = streamInfo.audios?.[0];

        taskStore.updateTaskMediaInfo(task.id, {
          resolution: bestVideo?.resolution,
          width: bestVideo?.width,
          height: bestVideo?.height,
          frameRate: bestVideo?.frameRate,
          videoCodec: bestVideo?.codecs,
          videoRange: bestVideo?.videoRange,
          audioCodec: bestAudio?.codecs,
          audioChannels: bestAudio?.channels,
          audioLanguage: bestAudio?.language,
          duration: streamInfo.duration,
          segmentCount: streamInfo.segmentCount,
          isLive: streamInfo.isLive,
          isEncrypted: streamInfo.isEncrypted,
          fileFormat: config.muxFormat,
        });
      } catch (parseError) {
        console.warn("Failed to parse stream info for task:", parseError);
      }

      toast.success(`开始下载: ${task.fileName || task.url}`);
    } catch (e) {
      console.error("Failed to start download:", e);
      taskStore.updateTaskStatus(task.id, "failed");

      const errorMessage =
        e instanceof Error ? e.message : typeof e === "string" ? e : "未知错误";
      taskStore.updateTaskError(task.id, errorMessage);
      toast.error(`下载失败: ${errorMessage}`);

      processQueue();
    } finally {
      startingTasks.value.delete(task.id);
    }
  };

  const stopDownload = async (taskId: string): Promise<void> => {
    try {
      await downloadService.stopDownload(taskId);
      taskStore.updateTaskStatus(taskId, "cancelled");
      unsubscribeTask(taskId);
      toast.info("下载已取消");
      processQueue();
    } catch (e) {
      console.error("Failed to stop download:", e);
      toast.error(`停止失败: ${e instanceof Error ? e.message : "未知错误"}`);
    }
  };

  const pauseDownload = async (taskId: string): Promise<void> => {
    try {
      await downloadService.pauseDownload(taskId);
      taskStore.updateTaskStatus(taskId, "paused");
      unsubscribeTask(taskId);
      toast.info("下载已暂停");
      processQueue();
    } catch (e) {
      console.error("Failed to pause download:", e);
      toast.error(`暂停失败: ${e instanceof Error ? e.message : "未知错误"}`);
    }
  };

  const resumeDownload = async (task: DownloadTask): Promise<void> => {
    try {
      await startDownload(task);
      toast.success("下载已恢复");
    } catch (e) {
      console.error("Failed to resume download:", e);
      toast.error(`恢复失败: ${e instanceof Error ? e.message : "未知错误"}`);
    }
  };

  const retryDownload = async (task: DownloadTask): Promise<void> => {
    taskStore.retryTask(task.id);
    await startDownload(taskStore.getTask(task.id)!);
  };

  const parseUrl = async (url: string): Promise<StreamInfo | null> => {
    isParsing.value = true;
    parsedStreamInfo.value = null;

    try {
      const allConfig = settingsStore.getAllConfig();
      const info = await downloadService.parseUrl(url, allConfig);
      parsedStreamInfo.value = info;
      return info;
    } catch (e) {
      console.error("Failed to parse URL:", e);
      toast.error(`解析失败: ${e instanceof Error ? e.message : "未知错误"}`);
      return null;
    } finally {
      isParsing.value = false;
    }
  };

  const analyzeAndUpdateMediaInfo = async (
    taskId: string,
    filePath: string,
  ): Promise<void> => {
    try {
      const result = await downloadService.analyzeMediaFile(filePath);

      taskStore.updateTaskMediaInfo(taskId, {
        resolution: result.resolution,
        width: result.width,
        height: result.height,
        frameRate: result.frameRate,
        videoCodec: result.videoCodec,
        videoRange: result.videoRange,
        audioCodec: result.audioCodec,
        audioChannels: result.audioChannels,
        audioLanguage: result.audioLanguage,
        duration: result.duration,
      });

      const task = taskStore.getTask(taskId);
      if (task && result.fileSize) {
        taskStore.updateTaskProgress(taskId, { totalSize: result.fileSize });
      }

      console.log("Media file analyzed successfully:", result);
    } catch (e) {
      console.warn("Failed to analyze media file:", e);
    }
  };

  const handleDownloadEvent = (event: DownloadEvent): void => {
    const { type, taskId, data } = event;

    switch (type) {
      case "progress": {
        const progressData = data as {
          percent: number;
          overallPercent?: number;
          speed: number;
          downloadedSize: number;
          totalSize: number;
          eta: number;
        };
        taskStore.updateTaskProgress(taskId, {
          percent: progressData.percent,
          overallPercent: progressData.overallPercent,
          speed: progressData.speed,
          downloadedSize: progressData.downloadedSize,
          totalSize: progressData.totalSize,
          eta: progressData.eta,
        });
        break;
      }

      case "status": {
        const statusData = data as { status: string; message?: string };
        taskStore.updateTaskStatus(
          taskId,
          statusData.status as DownloadTask["status"],
        );
        break;
      }

      case "error": {
        const errorData = data as { message: string };
        const task = taskStore.getTask(taskId);
        taskStore.updateTaskStatus(taskId, "failed");
        taskStore.updateTaskError(taskId, errorData.message);
        toast.error(`下载出错: ${errorData.message}`);
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
        const completeData = data as { outputPath: string };
        const task = taskStore.getTask(taskId);
        taskStore.updateTaskStatus(taskId, "completed");
        taskStore.updateTaskOutput(taskId, completeData.outputPath);
        toast.success("下载完成!");
        if (task) {
          notification.sendDownloadCompleteNotification(
            task.fileName || "文件",
          );
        }

        if (completeData.outputPath) {
          analyzeAndUpdateMediaInfo(taskId, completeData.outputPath).catch(
            (e) => {
              console.warn("Failed to analyze media file:", e);
            },
          );
        }

        unsubscribeTask(taskId);
        processQueue();
        break;
      }

      case "log": {
        const logData = data as { level: string; message: string };
        const level = logData.level as "info" | "warn" | "error" | "debug";
        taskStore.addTaskLog(taskId, level || "info", logData.message);
        break;
      }
    }
  };

  const unsubscribeTask = (taskId: string): void => {
    const unlisten = unlisteners.get(taskId);
    if (unlisten) {
      unlisten();
      unlisteners.delete(taskId);
    }
  };

  const startPendingTasks = async (): Promise<void> => {
    await processQueue();
  };

  const addAndStartTask = async (
    url: string,
    fileName?: string,
    saveDir?: string,
  ): Promise<DownloadTask> => {
    const task = await taskStore.addTask(url, fileName, saveDir);

    if (settingsStore.appSettings.auto_start_download) {
      await processQueue();
    }

    return task;
  };

  const checkDownloaderAvailable = async (): Promise<boolean> => {
    try {
      return await downloadService.checkDownloaderAvailable();
    } catch {
      return false;
    }
  };

  const getDownloaderVersion = async (): Promise<string> => {
    try {
      return await downloadService.getDownloaderVersion();
    } catch {
      return "未知版本";
    }
  };

  onUnmounted(() => {
    unlisteners.forEach((unlisten) => unlisten());
    unlisteners.clear();
    downloadService.unsubscribeFromAll();
  });

  return {
    // State
    startingTasks,
    isParsing,
    parsedStreamInfo,

    // Actions
    startDownload,
    stopDownload,
    pauseDownload,
    resumeDownload,
    retryDownload,
    parseUrl,
    startPendingTasks,
    addAndStartTask,
    processQueue,

    // Helpers
    checkDownloaderAvailable,
    getDownloaderVersion,
  };
}
