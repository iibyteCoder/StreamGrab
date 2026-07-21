/**
 * 任务管理组合式函数
 *
 * taskStore 的薄封装，保持旧 API 名称以降低组件层迁移成本。
 * 内部全部委托 taskStore / useDownloader。
 */

import { computed } from "vue";
import { useTaskStore } from "@/stores";
import type { DownloadTask, TaskStatus, TaskOverrides } from "@/domain";

/** 添加任务的结果 */
export interface AddTaskResult {
  task?: DownloadTask;
  duplicateUrl?: boolean;
  existingTask?: DownloadTask;
  wasRenamed?: boolean;
}

export function useTasks() {
  const store = useTaskStore();

  // ==========================================
  // Computed
  // ==========================================

  const tasks = computed(() => store.tasks);
  const activeTasks = computed(() => store.activeTasks);
  const pendingTasks = computed(() => store.pendingTasks);
  const completedTasks = computed(() => store.completedTasks);
  const failedTasks = computed(() => store.failedTasks);
  const hasTasks = computed(() => store.hasTasks);
  const canStartMore = computed(() => store.canStartMore);
  const stats = computed(() => store.totalProgress);

  // ==========================================
  // Actions
  // ==========================================

  /** 检查 URL 是否已存在 */
  const checkUrlExists = (url: string): DownloadTask | undefined => {
    return store.checkUrlExists(url);
  };

  /** 添加新任务（带 URL 重复检测和文件名冲突检测） */
  const addTask = (
    url: string,
    fileName?: string,
    saveDir?: string,
    skipUrlCheck = false,
    overrides?: TaskOverrides,
  ): AddTaskResult => {
    if (!skipUrlCheck) {
      const existingTask = store.checkUrlExists(url);
      if (existingTask) {
        return { duplicateUrl: true, existingTask };
      }
    }

    const { task, wasRenamed } = store.addTaskSync({
      url,
      fileName,
      saveDir,
      overrides,
    });
    return { task, wasRenamed };
  };

  /** 强制添加任务（跳过 URL 检查） */
  const forceAddTask = (
    url: string,
    fileName?: string,
    saveDir?: string,
    overrides?: TaskOverrides,
  ): { task: DownloadTask; wasRenamed: boolean } => {
    return store.addTaskSync({ url, fileName, saveDir, overrides });
  };

  /** 批量添加任务 */
  const addTasks = (
    urls: string[],
    saveDir?: string,
  ): { tasks: DownloadTask[]; duplicateUrls: string[] } => {
    const createdTasks: DownloadTask[] = [];
    const duplicateUrls: string[] = [];

    for (const url of urls) {
      const result = addTask(url, undefined, saveDir);
      if (result.task) {
        createdTasks.push(result.task);
      } else if (result.duplicateUrl) {
        duplicateUrls.push(url);
      }
    }

    return { tasks: createdTasks, duplicateUrls };
  };

  const getTask = (taskId: string): DownloadTask | undefined => {
    return store.getTaskById(taskId);
  };

  const removeTask = (taskId: string): void => {
    store.removeTask(taskId);
  };

  const clearCompleted = (): void => {
    store.clearCompleted();
  };

  const clearAll = (): void => {
    store.clearAll();
  };

  const retryTask = (taskId: string): void => {
    store.retryTask(taskId);
  };

  const retryAllFailed = (): void => {
    for (const task of failedTasks.value) {
      retryTask(task.id);
    }
  };

  // ==========================================
  // Helpers
  // ==========================================

  const isTaskRunning = (taskId: string): boolean => {
    return getTask(taskId)?.status === "downloading";
  };

  const isTaskCompleted = (taskId: string): boolean => {
    return getTask(taskId)?.status === "completed";
  };

  const isTaskFailed = (taskId: string): boolean => {
    return getTask(taskId)?.status === "failed";
  };

  const getTaskProgress = (taskId: string): number => {
    return getTask(taskId)?.progress.overallPercent ?? 0;
  };

  const filterByStatus = (status: TaskStatus): DownloadTask[] => {
    return tasks.value.filter((task) => task.status === status);
  };

  const search = (query: string): DownloadTask[] => {
    const lowerQuery = query.toLowerCase();
    return tasks.value.filter(
      (task) =>
        task.url.toLowerCase().includes(lowerQuery) ||
        task.fileName?.toLowerCase().includes(lowerQuery),
    );
  };

  return {
    // State
    tasks,
    activeTasks,
    pendingTasks,
    completedTasks,
    failedTasks,
    hasTasks,
    canStartMore,
    stats,

    // Actions
    addTask,
    addTasks,
    forceAddTask,
    checkUrlExists,
    getTask,
    removeTask,
    clearCompleted,
    clearAll,
    retryTask,
    retryAllFailed,

    // Helpers
    isTaskRunning,
    isTaskCompleted,
    isTaskFailed,
    getTaskProgress,
    filterByStatus,
    search,
  };
}
