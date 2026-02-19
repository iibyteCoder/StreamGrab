/**
 * 任务管理组合式函数
 * 封装任务的 CRUD 操作和状态查询
 */

import { computed } from "vue";
import { useTaskStore } from "@/stores";
import type { DownloadTask, TaskStatus, TaskProgressData } from "@/types";

/**
 * 任务组合式函数
 */
export function useTasks() {
  const store = useTaskStore();

  // 所有任务
  const tasks = computed(() => store.tasks);

  // 活跃任务（正在下载）
  const activeTasks = computed(() => store.activeTasks);

  // 等待中的任务
  const pendingTasks = computed(() => store.pendingTasks);

  // 已完成的任务
  const completedTasks = computed(() => store.completedTasks);

  // 失败的任务
  const failedTasks = computed(() => store.failedTasks);

  // 是否有任务
  const hasTasks = computed(() => store.hasTasks);

  // 是否可以开始更多任务
  const canStartMore = computed(() => store.canStartMore);

  // 最大并发数
  const maxConcurrent = computed({
    get: () => store.maxConcurrent,
    set: (value: number) => store.setMaxConcurrent(value),
  });

  // 统计信息
  const stats = computed(() => store.totalProgress);

  /**
   * 添加新任务
   */
  const addTask = (
    url: string,
    fileName?: string,
    saveDir?: string,
  ): DownloadTask => {
    return store.addTaskSync(url, fileName, saveDir);
  };

  /**
   * 批量添加任务
   */
  const addTasks = (urls: string[], saveDir?: string): DownloadTask[] => {
    return urls.map((url) => addTask(url, undefined, saveDir));
  };

  /**
   * 获取任务
   */
  const getTask = (taskId: string): DownloadTask | undefined => {
    return store.getTask(taskId);
  };

  /**
   * 更新任务状态
   */
  const updateTaskStatus = (taskId: string, status: TaskStatus): void => {
    store.updateTaskStatus(taskId, status);
  };

  /**
   * 更新任务进度
   */
  const updateTaskProgress = (
    taskId: string,
    progress: Partial<TaskProgressData>,
  ): void => {
    store.updateTaskProgress(taskId, progress);
  };

  /**
   * 更新任务配置
   */
  const updateTaskConfig = (
    taskId: string,
    config: Record<string, unknown>,
  ): void => {
    store.updateTaskConfig(taskId, config);
  };

  /**
   * 移除任务
   */
  const removeTask = (taskId: string): void => {
    store.removeTask(taskId);
  };

  /**
   * 清除已完成的任务
   */
  const clearCompleted = (): void => {
    store.clearCompleted();
  };

  /**
   * 清除所有任务
   */
  const clearAll = (): void => {
    store.clearAll();
  };

  /**
   * 重试任务
   */
  const retryTask = (taskId: string): void => {
    store.retryTask(taskId);
  };

  /**
   * 重试所有失败的任务
   */
  const retryAllFailed = (): void => {
    for (const task of failedTasks.value) {
      retryTask(task.id);
    }
  };

  /**
   * 检查任务是否正在运行
   */
  const isTaskRunning = (taskId: string): boolean => {
    const task = getTask(taskId);
    return task?.status === "downloading";
  };

  /**
   * 检查任务是否已完成
   */
  const isTaskCompleted = (taskId: string): boolean => {
    const task = getTask(taskId);
    return task?.status === "completed";
  };

  /**
   * 检查任务是否失败
   */
  const isTaskFailed = (taskId: string): boolean => {
    const task = getTask(taskId);
    return task?.status === "failed";
  };

  /**
   * 获取任务进度百分比
   */
  const getTaskProgress = (taskId: string): number => {
    const task = getTask(taskId);
    return task?.progressPercent ?? 0;
  };

  /**
   * 按状态过滤任务
   */
  const filterByStatus = (status: TaskStatus): DownloadTask[] => {
    return tasks.value.filter((task) => task.status === status);
  };

  /**
   * 搜索任务
   */
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
    maxConcurrent,
    stats,

    // Actions
    addTask,
    addTasks,
    getTask,
    updateTaskStatus,
    updateTaskProgress,
    updateTaskConfig,
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
