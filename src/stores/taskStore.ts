/**
 * 任务状态管理
 *
 * Store 作为缓存层，数据来源于后端 SQLite
 */

import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type {
  DownloadTask,
  TaskStatus,
  TaskProgressData,
  TaskLogEntry,
} from "@/types";
import { extractFileName } from "@/utils/format";
import { MAX_CONCURRENT_TASKS } from "@/utils/constants";
import { taskService } from "@/services";

// 最大日志条目数（每个任务）
const MAX_LOG_ENTRIES = 500;

/**
 * 生成唯一 ID
 */
function generateId(): string {
  return `${Date.now()}-${Math.random().toString(36).slice(2, 11)}`;
}

/**
 * 从 URL 提取文件名
 */
function extractNameFromUrl(url: string): string {
  return extractFileName(url);
}

/**
 * 创建空进度对象
 */
function createEmptyProgress(): TaskProgressData {
  return {
    percent: 0,
    speed: 0,
    downloadedSize: 0,
    totalSize: 0,
    downloadedSegments: 0,
    totalSegments: 0,
    eta: 0,
    currentAction: "",
  };
}

export const useTaskStore = defineStore("task", () => {
  // ==========================================
  // State - 缓存层
  // ==========================================

  const tasks = ref<DownloadTask[]>([]);
  const taskLogs = ref<Map<string, TaskLogEntry[]>>(new Map());
  const maxConcurrent = ref(MAX_CONCURRENT_TASKS);
  const isLoading = ref(false);
  const isInitialized = ref(false);

  // ==========================================
  // Getters
  // ==========================================

  const activeTasks = computed(() =>
    tasks.value.filter((t) =>
      ["downloading", "analyzing", "merging", "muxing"].includes(t.status),
    ),
  );

  const pendingTasks = computed(() =>
    tasks.value.filter((t) => t.status === "pending"),
  );

  const completedTasks = computed(() =>
    tasks.value.filter((t) => t.status === "completed"),
  );

  const failedTasks = computed(() =>
    tasks.value.filter((t) => t.status === "failed"),
  );

  const downloadingTasks = computed(() =>
    tasks.value.filter((t) => t.status === "downloading"),
  );

  const canStartMore = computed(
    () => activeTasks.value.length < maxConcurrent.value,
  );

  const hasTasks = computed(() => tasks.value.length > 0);

  const totalProgress = computed(() => {
    const completed = completedTasks.value.length;
    const total = tasks.value.length;
    const percent = total > 0 ? Math.round((completed / total) * 100) : 0;
    return { completed, total, percent };
  });

  // ==========================================
  // Actions - 初始化
  // ==========================================

  /**
   * 初始化 Store - 从后端加载任务
   */
  async function initialize(): Promise<void> {
    if (isInitialized.value) return;

    isLoading.value = true;
    try {
      const records = await taskService.loadAllTasks();
      tasks.value = records.map((r) => taskService.toDownloadTask(r));
      isInitialized.value = true;
    } catch (error) {
      console.error("Failed to initialize task store:", error);
    } finally {
      isLoading.value = false;
    }
  }

  // ==========================================
  // Actions - 任务操作（同步到后端）
  // ==========================================

  /**
   * 添加任务 - 持久化到后端
   */
  async function addTask(
    url: string,
    fileName?: string,
    saveDir?: string,
  ): Promise<DownloadTask> {
    const task: DownloadTask = {
      id: generateId(),
      url: url.trim(),
      fileName: fileName || extractNameFromUrl(url),
      saveDir: saveDir || "",
      status: "pending",
      progress: createEmptyProgress(),
      createdAt: new Date(),
      updatedAt: new Date(),
    };

    // 持久化到后端
    await taskService.saveTask(task);

    // 更新缓存
    tasks.value.push(task);
    return task;
  }

  /**
   * 同步添加任务（不等待后端，用于兼容旧代码）
   */
  function addTaskSync(
    url: string,
    fileName?: string,
    saveDir?: string,
  ): DownloadTask {
    const task: DownloadTask = {
      id: generateId(),
      url: url.trim(),
      fileName: fileName || extractNameFromUrl(url),
      saveDir: saveDir || "",
      status: "pending",
      progress: createEmptyProgress(),
      createdAt: new Date(),
      updatedAt: new Date(),
    };

    tasks.value.push(task);

    // 异步保存到后端（不阻塞）
    taskService.saveTask(task).catch(console.error);

    return task;
  }

  function getTask(taskId: string): DownloadTask | undefined {
    return tasks.value.find((t) => t.id === taskId);
  }

  /**
   * 更新任务状态 - 同步到后端
   */
  async function updateTaskStatus(
    taskId: string,
    status: TaskStatus,
  ): Promise<void> {
    const task = tasks.value.find((t) => t.id === taskId);
    if (!task) return;

    const now = new Date();
    task.status = status;
    task.updatedAt = now;

    if (status === "downloading" && !task.startedAt) {
      task.startedAt = now;
    }
    if (status === "completed") {
      task.completedAt = now;
    }

    // 同步到后端（fire-and-forget）
    taskService
      .updateTaskStatus(taskId, status, task.error)
      .catch(console.error);
  }

  /**
   * 更新任务进度 - 批量同步到后端
   */
  let progressSyncTimer: ReturnType<typeof setTimeout> | null = null;
  const pendingProgressUpdates = new Map<string, TaskProgressData>();

  function updateTaskProgress(
    taskId: string,
    progress: Partial<TaskProgressData>,
  ): void {
    const task = tasks.value.find((t) => t.id === taskId);
    if (!task) return;

    task.progress = { ...task.progress, ...progress };
    task.updatedAt = new Date();

    // 收集进度更新，批量同步
    pendingProgressUpdates.set(taskId, task.progress);

    // 防抖同步（每 2 秒同步一次）
    if (!progressSyncTimer) {
      progressSyncTimer = setTimeout(() => {
        flushProgressUpdates();
        progressSyncTimer = null;
      }, 2000);
    }
  }

  /**
   * 刷新进度更新到后端
   */
  async function flushProgressUpdates(): Promise<void> {
    if (pendingProgressUpdates.size === 0) return;

    const updates = Array.from(pendingProgressUpdates.entries());
    pendingProgressUpdates.clear();

    for (const [taskId, progress] of updates) {
      taskService.updateTaskProgress(taskId, progress).catch(console.error);
    }
  }

  function updateTaskError(taskId: string, error: string): void {
    const task = tasks.value.find((t) => t.id === taskId);
    if (!task) return;

    task.error = error;
    task.status = "failed";
    task.updatedAt = new Date();

    // 同步到后端
    taskService.updateTaskStatus(taskId, "failed", error).catch(console.error);
  }

  function updateTaskOutput(taskId: string, outputPath: string): void {
    const task = tasks.value.find((t) => t.id === taskId);
    if (!task) return;

    task.outputPath = outputPath;
    task.updatedAt = new Date();

    // 同步到后端
    taskService.saveTask(task).catch(console.error);
  }

  /**
   * 更新任务配置 - 同步到后端
   */
  function updateTaskConfig(
    taskId: string,
    config: Partial<DownloadTask["config"]>,
  ): void {
    const task = tasks.value.find((t) => t.id === taskId);
    if (!task) return;

    task.config = { ...task.config, ...config };
    task.updatedAt = new Date();

    // 同步到后端
    taskService.saveTask(task).catch(console.error);
  }

  function retryTask(taskId: string): void {
    const task = tasks.value.find((t) => t.id === taskId);
    if (!task) return;

    task.status = "pending";
    task.error = undefined;
    task.progress = createEmptyProgress();
    task.updatedAt = new Date();

    // 同步到后端
    taskService.updateTaskStatus(taskId, "pending").catch(console.error);
  }

  /**
   * 删除任务 - 同步到后端
   */
  async function removeTask(taskId: string): Promise<void> {
    const index = tasks.value.findIndex((t) => t.id === taskId);
    if (index === -1) return;

    // 先删除后端数据
    await taskService.deleteTask(taskId);

    // 再更新缓存
    tasks.value.splice(index, 1);
  }

  /**
   * 清除已完成的任务
   */
  async function clearCompleted(): Promise<void> {
    await taskService.clearFinishedTasks();
    tasks.value = tasks.value.filter((t) => t.status !== "completed");
  }

  function clearFailed(): void {
    const failedIds = tasks.value
      .filter((t) => t.status === "failed")
      .map((t) => t.id);
    tasks.value = tasks.value.filter((t) => t.status !== "failed");

    // 异步删除后端数据
    for (const id of failedIds) {
      taskService.deleteTask(id).catch(console.error);
    }
  }

  async function clearAll(): Promise<void> {
    await taskService.clearAllTasks();
    tasks.value = [];
  }

  function reorderTasks(fromIndex: number, toIndex: number): void {
    const removed = tasks.value.splice(fromIndex, 1)[0];
    if (removed) {
      tasks.value.splice(toIndex, 0, removed);
    }
  }

  function setMaxConcurrent(value: number): void {
    maxConcurrent.value = Math.max(1, Math.min(MAX_CONCURRENT_TASKS, value));
  }

  // ==========================================
  // Actions - 日志管理（仅内存）
  // ==========================================

  /**
   * 添加任务日志
   */
  function addTaskLog(
    taskId: string,
    level: TaskLogEntry["level"],
    message: string,
  ): void {
    let logs = taskLogs.value.get(taskId);
    if (!logs) {
      logs = [];
      taskLogs.value.set(taskId, logs);
    }

    logs.push({
      timestamp: new Date(),
      level,
      message,
    });

    // 限制日志条目数
    if (logs.length > MAX_LOG_ENTRIES) {
      logs.splice(0, logs.length - MAX_LOG_ENTRIES);
    }
  }

  /**
   * 获取任务日志
   */
  function getTaskLogs(taskId: string): TaskLogEntry[] {
    return taskLogs.value.get(taskId) || [];
  }

  /**
   * 清除任务日志
   */
  function clearTaskLogs(taskId: string): void {
    taskLogs.value.delete(taskId);
  }

  return {
    // State
    tasks,
    taskLogs,
    maxConcurrent,
    isLoading,
    isInitialized,

    // Getters
    activeTasks,
    pendingTasks,
    completedTasks,
    failedTasks,
    downloadingTasks,
    canStartMore,
    hasTasks,
    totalProgress,

    // Actions
    initialize,
    addTask,
    addTaskSync,
    getTask,
    updateTaskStatus,
    updateTaskProgress,
    updateTaskError,
    updateTaskOutput,
    updateTaskConfig,
    retryTask,
    removeTask,
    clearCompleted,
    clearFailed,
    clearAll,
    reorderTasks,
    setMaxConcurrent,

    // Log Actions
    addTaskLog,
    getTaskLogs,
    clearTaskLogs,
  };
});
