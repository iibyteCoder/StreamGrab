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

  // 正在启动的任务
  const startingTasks = ref<Set<string>>(new Set());

  // 事件订阅清理函数
  const unlisteners = new Map<string, UnlistenFn>();

  // 是否正在解析（用于 UI 显示）
  const isParsing = ref(false);

  // 解析结果（用于 URL 输入时的流选择器预览）
  const parsedStreamInfo = ref<StreamInfo | null>(null);

  // 是否正在处理队列（防止重复触发）
  let isProcessingQueue = false;

  /**
   * 处理下载队列 - 自动启动等待中的任务
   * 确保不超过最大并发数
   */
  const processQueue = async (): Promise<void> => {
    // 防止重复触发
    if (isProcessingQueue) {
      return;
    }
    isProcessingQueue = true;

    try {
      // 持续检查并启动等待中的任务，直到达到并发上限或没有等待中的任务
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
   * 开始下载任务
   */
  const startDownload = async (task: DownloadTask): Promise<void> => {
    // 检查是否已经在启动中
    if (startingTasks.value.has(task.id)) {
      return;
    }

    // 检查并发限制
    if (!taskStore.canStartMore) {
      // 如果不能启动更多，将任务状态设为等待
      taskStore.updateTaskStatus(task.id, "pending");
      return;
    }

    startingTasks.value.add(task.id);

    try {
      // 更新任务状态为解析中
      taskStore.updateTaskStatus(task.id, "analyzing");

      // 获取任务配置 - 使用应用级设置
      const taskData = taskStore.getTask(task.id);
      const config: TaskConfig = {
        // 基础配置
        saveDir:
          taskData?.saveDir ||
          task.saveDir ||
          settingsStore.settings.general.saveDir,
        saveName: task.fileName,
        threadCount: settingsStore.settings.download.threadCount,
        retryCount: settingsStore.settings.download.retryCount,
        timeout: settingsStore.settings.download.timeout,
        maxSpeed: settingsStore.settings.download.maxSpeed,

        // 流选择
        autoSelect: settingsStore.settings.download.autoSelect,
        selectVideo: settingsStore.settings.download.selectVideo,
        selectAudio: settingsStore.settings.download.selectAudio,
        selectSubtitle: settingsStore.settings.download.selectSubtitle,

        // 流排除
        dropVideo: settingsStore.settings.download.dropVideo,
        dropAudio: settingsStore.settings.download.dropAudio,
        dropSubtitle: settingsStore.settings.download.dropSubtitle,

        // 命名模板
        savePattern: settingsStore.settings.download.savePattern,

        // 混流
        muxFormat: settingsStore.settings.mux.format,
        muxAfterDone: !settingsStore.settings.download.skipMerge,
        skipMerge: settingsStore.settings.download.skipMerge,
        delAfterDone: settingsStore.settings.download.delAfterDone,
        checkSegmentsCount: settingsStore.settings.download.checkSegmentsCount,

        // 其他选项
        headers: settingsStore.settings.network.headers.filter(
          (h) => h.enabled,
        ),
        proxy: settingsStore.settings.network.customProxy || undefined,
      };

      // 订阅任务事件
      const unlisten = await downloadService.subscribeToTask(
        task.id,
        handleDownloadEvent,
      );
      unlisteners.set(task.id, unlisten);

      // 启动下载
      await downloadService.startDownload(task, config, settingsStore.settings);

      // 更新任务状态为下载中
      taskStore.updateTaskStatus(task.id, "downloading");

      // 为每个任务单独解析流信息（确保任务隔离）
      try {
        const streamInfo = await downloadService.parseUrl(
          task.url,
          settingsStore.settings,
        );
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
        // 解析失败不影响下载，只记录日志
        console.warn("Failed to parse stream info for task:", parseError);
      }

      toast.success(`开始下载: ${task.fileName || task.url}`);
    } catch (e) {
      console.error("Failed to start download:", e);
      taskStore.updateTaskStatus(task.id, "failed");

      // 提取更详细的错误信息
      let errorMessage = "未知错误";
      if (e instanceof Error) {
        errorMessage = e.message;
      } else if (typeof e === "string") {
        errorMessage = e;
      }

      // 保存错误信息到任务
      taskStore.updateTaskError(task.id, errorMessage);
      toast.error(`下载失败: ${errorMessage}`);

      // 任务失败后尝试启动下一个等待中的任务
      processQueue();
    } finally {
      startingTasks.value.delete(task.id);
    }
  };

  /**
   * 停止下载任务
   */
  const stopDownload = async (taskId: string): Promise<void> => {
    try {
      await downloadService.stopDownload(taskId);
      taskStore.updateTaskStatus(taskId, "cancelled");

      // 取消订阅
      unsubscribeTask(taskId);

      toast.info("下载已取消");

      // 任务取消后尝试启动下一个等待中的任务
      processQueue();
    } catch (e) {
      console.error("Failed to stop download:", e);
      toast.error(`停止失败: ${e instanceof Error ? e.message : "未知错误"}`);
    }
  };

  /**
   * 暂停下载任务
   */
  const pauseDownload = async (taskId: string): Promise<void> => {
    try {
      await downloadService.pauseDownload(taskId);
      taskStore.updateTaskStatus(taskId, "paused");

      // 取消订阅（暂停时进程会停止）
      unsubscribeTask(taskId);

      toast.info("下载已暂停");

      // 任务暂停后尝试启动下一个等待中的任务
      processQueue();
    } catch (e) {
      console.error("Failed to pause download:", e);
      toast.error(`暂停失败: ${e instanceof Error ? e.message : "未知错误"}`);
    }
  };

  /**
   * 恢复下载任务
   */
  const resumeDownload = async (task: DownloadTask): Promise<void> => {
    try {
      // 恢复下载实际上是重新启动
      await startDownload(task);

      toast.success("下载已恢复");
    } catch (e) {
      console.error("Failed to resume download:", e);
      toast.error(`恢复失败: ${e instanceof Error ? e.message : "未知错误"}`);
    }
  };

  /**
   * 重试下载任务
   */
  const retryDownload = async (task: DownloadTask): Promise<void> => {
    taskStore.retryTask(task.id);
    await startDownload(taskStore.getTask(task.id)!);
  };

  /**
   * 解析 URL 获取流信息
   */
  const parseUrl = async (url: string): Promise<StreamInfo | null> => {
    isParsing.value = true;
    parsedStreamInfo.value = null;

    try {
      const info = await downloadService.parseUrl(url, settingsStore.settings);
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

  /**
   * 分析媒体文件并更新任务媒体信息
   * 在下载完成后调用，使用 ffprobe 分析已下载的文件
   */
  const analyzeAndUpdateMediaInfo = async (
    taskId: string,
    filePath: string,
  ): Promise<void> => {
    try {
      const result = await downloadService.analyzeMediaFile(filePath);

      // 更新任务的媒体信息
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
        // 更新总大小（如果分析成功）
        ...(result.fileSize
          ? { segmentCount: undefined } // 文件大小单独处理
          : {}),
      });

      // 同时更新进度中的总大小
      const task = taskStore.getTask(taskId);
      if (task && result.fileSize) {
        taskStore.updateTaskProgress(taskId, {
          totalSize: result.fileSize,
        });
      }

      console.log("Media file analyzed successfully:", result);
    } catch (e) {
      console.warn("Failed to analyze media file:", e);
      // 分析失败不影响下载完成状态，只是媒体信息可能不完整
    }
  };

  /**
   * 处理下载事件
   */
  const handleDownloadEvent = (event: DownloadEvent): void => {
    const { type, taskId, data } = event;

    switch (type) {
      case "progress": {
        // 更新进度
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
        // 进度历史由后端自动保存，前端无需处理
        break;
      }

      case "status": {
        // 更新状态
        const statusData = data as { status: string; message?: string };
        taskStore.updateTaskStatus(
          taskId,
          statusData.status as DownloadTask["status"],
        );
        break;
      }

      case "error": {
        // 处理错误
        const errorData = data as { message: string };
        const task = taskStore.getTask(taskId);
        taskStore.updateTaskStatus(taskId, "failed");
        taskStore.updateTaskError(taskId, errorData.message);
        toast.error(`下载出错: ${errorData.message}`);
        // 发送系统通知
        if (task) {
          notification.sendDownloadErrorNotification(
            task.fileName || task.url,
            errorData.message,
          );
        }
        unsubscribeTask(taskId);
        // 任务失败后尝试启动下一个等待中的任务
        processQueue();
        break;
      }

      case "complete": {
        // 下载完成 - 任务状态变为 completed 后会自动出现在历史记录中
        const completeData = data as { outputPath: string };
        const task = taskStore.getTask(taskId);
        taskStore.updateTaskStatus(taskId, "completed");
        taskStore.updateTaskOutput(taskId, completeData.outputPath);
        toast.success("下载完成!");
        // 发送系统通知
        if (task) {
          notification.sendDownloadCompleteNotification(
            task.fileName || "文件",
          );
        }

        // 下载完成后分析媒体文件，更新详细的媒体信息
        if (completeData.outputPath) {
          analyzeAndUpdateMediaInfo(taskId, completeData.outputPath).catch(
            (e) => {
              console.warn("Failed to analyze media file:", e);
            },
          );
        }

        unsubscribeTask(taskId);
        // 任务完成后尝试启动下一个等待中的任务
        processQueue();
        break;
      }

      case "log": {
        // 存储日志到任务
        const logData = data as { level: string; message: string };
        const level = logData.level as "info" | "warn" | "error" | "debug";
        taskStore.addTaskLog(taskId, level || "info", logData.message);
        break;
      }
    }
  };

  /**
   * 取消任务的事件订阅
   */
  const unsubscribeTask = (taskId: string): void => {
    const unlisten = unlisteners.get(taskId);
    if (unlisten) {
      unlisten();
      unlisteners.delete(taskId);
    }
  };

  /**
   * 启动等待中的任务
   * 手动触发队列处理
   */
  const startPendingTasks = async (): Promise<void> => {
    await processQueue();
  };

  /**
   * 添加任务并自动启动（如果设置允许）
   */
  const addAndStartTask = async (
    url: string,
    fileName?: string,
    saveDir?: string,
  ): Promise<DownloadTask> => {
    // 添加任务到 store（异步保存到后端）
    const task = await taskStore.addTask(url, fileName, saveDir);

    // 如果设置了自动开始下载，尝试启动
    if (settingsStore.settings.general.autoStartDownload) {
      await processQueue();
    }

    return task;
  };

  /**
   * 检查下载器是否可用
   */
  const checkDownloaderAvailable = async (): Promise<boolean> => {
    try {
      return await downloadService.checkDownloaderAvailable();
    } catch {
      return false;
    }
  };

  /**
   * 获取下载器版本
   */
  const getDownloaderVersion = async (): Promise<string> => {
    try {
      return await downloadService.getDownloaderVersion();
    } catch {
      return "未知版本";
    }
  };

  // 清理所有订阅
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
