/**
 * 下载器组合式函数
 * 封装下载的核心逻辑，包括启动、停止、暂停、恢复等
 * 支持并发控制，自动启动等待中的任务
 */

import { ref, onUnmounted } from 'vue';
import { useTaskStore, useSettingsStore } from '@/stores';
import { downloadService, type DownloadEvent, type UnlistenFn } from '@/services';
import { useToast } from './useToast';
import type { DownloadTask, TaskConfig, StreamInfo } from '@/types';

/**
 * 下载器组合式函数
 */
export function useDownloader() {
  const taskStore = useTaskStore();
  const settingsStore = useSettingsStore();
  const toast = useToast();

  // 正在启动的任务
  const startingTasks = ref<Set<string>>(new Set());

  // 事件订阅清理函数
  const unlisteners = new Map<string, UnlistenFn>();

  // 是否正在解析
  const isParsing = ref(false);

  // 解析结果
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
      taskStore.updateTaskStatus(task.id, 'pending');
      return;
    }

    startingTasks.value.add(task.id);

    try {
      // 更新任务状态为解析中
      taskStore.updateTaskStatus(task.id, 'analyzing');

      // 获取任务配置 - 合并任务级配置和应用级设置
      const taskData = taskStore.getTask(task.id);
      const taskConfig = taskData?.config || {};
      const config: TaskConfig = {
        // 基础配置
        saveDir: taskData?.saveDir || task.saveDir || settingsStore.settings.general.saveDir,
        saveName: task.fileName,
        threadCount: settingsStore.settings.download.threadCount,
        retryCount: settingsStore.settings.download.retryCount,
        timeout: settingsStore.settings.download.timeout,
        maxSpeed: settingsStore.settings.download.maxSpeed,

        // 流选择 - 任务级可覆盖
        autoSelect: taskConfig.autoSelect ?? settingsStore.settings.download.autoSelect,
        selectVideo: taskConfig.selectVideo ?? settingsStore.settings.download.selectVideo,
        selectAudio: taskConfig.selectAudio ?? settingsStore.settings.download.selectAudio,
        selectSubtitle: taskConfig.selectSubtitle ?? settingsStore.settings.download.selectSubtitle,

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
        customRange: taskConfig.customRange,
        startAt: taskConfig.startAt,
        key: taskConfig.key,
        headers: settingsStore.settings.network.headers.filter(h => h.enabled),
        proxy: settingsStore.settings.network.customProxy || undefined,
      };

      // 订阅任务事件
      const unlisten = await downloadService.subscribeToTask(
        task.id,
        handleDownloadEvent
      );
      unlisteners.set(task.id, unlisten);

      // 启动下载
      await downloadService.startDownload(
        task,
        config,
        settingsStore.settings
      );

      // 更新任务状态为下载中
      taskStore.updateTaskStatus(task.id, 'downloading');

      toast.success(`开始下载: ${task.fileName || task.url}`);
    } catch (e) {
      console.error('Failed to start download:', e);
      taskStore.updateTaskStatus(task.id, 'failed');

      // 提取更详细的错误信息
      let errorMessage = '未知错误';
      if (e instanceof Error) {
        errorMessage = e.message;
      } else if (typeof e === 'string') {
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
      taskStore.updateTaskStatus(taskId, 'cancelled');

      // 取消订阅
      unsubscribeTask(taskId);

      toast.info('下载已取消');

      // 任务取消后尝试启动下一个等待中的任务
      processQueue();
    } catch (e) {
      console.error('Failed to stop download:', e);
      toast.error(`停止失败: ${e instanceof Error ? e.message : '未知错误'}`);
    }
  };

  /**
   * 暂停下载任务
   */
  const pauseDownload = async (taskId: string): Promise<void> => {
    try {
      await downloadService.pauseDownload(taskId);
      taskStore.updateTaskStatus(taskId, 'paused');

      // 取消订阅（暂停时进程会停止）
      unsubscribeTask(taskId);

      toast.info('下载已暂停');

      // 任务暂停后尝试启动下一个等待中的任务
      processQueue();
    } catch (e) {
      console.error('Failed to pause download:', e);
      toast.error(`暂停失败: ${e instanceof Error ? e.message : '未知错误'}`);
    }
  };

  /**
   * 恢复下载任务
   */
  const resumeDownload = async (task: DownloadTask): Promise<void> => {
    try {
      // 恢复下载实际上是重新启动
      await startDownload(task);

      toast.success('下载已恢复');
    } catch (e) {
      console.error('Failed to resume download:', e);
      toast.error(`恢复失败: ${e instanceof Error ? e.message : '未知错误'}`);
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
      const info = await downloadService.parseUrl(
        url,
        settingsStore.settings
      );
      parsedStreamInfo.value = info;
      return info;
    } catch (e) {
      console.error('Failed to parse URL:', e);
      toast.error(`解析失败: ${e instanceof Error ? e.message : '未知错误'}`);
      return null;
    } finally {
      isParsing.value = false;
    }
  };

  /**
   * 处理下载事件
   */
  const handleDownloadEvent = (event: DownloadEvent): void => {
    const { type, taskId, data } = event;

    switch (type) {
      case 'progress':
        // 更新进度
        taskStore.updateTaskProgress(taskId, {
          percent: (data as { percent: number }).percent,
          speed: (data as { speed: number }).speed,
          downloadedSize: (data as { downloadedSize: number }).downloadedSize,
          totalSize: (data as { totalSize: number }).totalSize,
          eta: (data as { eta: number }).eta,
        });
        break;

      case 'status': {
        // 更新状态
        const statusData = data as { status: string; message?: string };
        taskStore.updateTaskStatus(taskId, statusData.status as DownloadTask['status']);
        break;
      }

      case 'error': {
        // 处理错误
        const errorData = data as { message: string };
        taskStore.updateTaskStatus(taskId, 'failed');
        taskStore.updateTaskError(taskId, errorData.message);
        toast.error(`下载出错: ${errorData.message}`);
        unsubscribeTask(taskId);
        // 任务失败后尝试启动下一个等待中的任务
        processQueue();
        break;
      }

      case 'complete': {
        // 下载完成 - 任务状态变为 completed 后会自动出现在历史记录中
        taskStore.updateTaskStatus(taskId, 'completed');
        const completeData = data as { outputPath: string };
        taskStore.updateTaskOutput(taskId, completeData.outputPath);
        toast.success('下载完成!');

        unsubscribeTask(taskId);
        // 任务完成后尝试启动下一个等待中的任务
        processQueue();
        break;
      }

      case 'log': {
        // 存储日志到任务
        const logData = data as { level: string; message: string };
        const level = logData.level as 'info' | 'warn' | 'error' | 'debug';
        taskStore.addTaskLog(taskId, level || 'info', logData.message);
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
  const addAndStartTask = async (url: string, fileName?: string, saveDir?: string): Promise<DownloadTask> => {
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
      return '未知版本';
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
