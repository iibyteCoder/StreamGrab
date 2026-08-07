/**
 * 任务状态管理
 *
 * Store 作为内存缓存层，数据权威来源为后端 SQLite。
 * 进度防抖落盘由 taskService 负责，Store 只管内存状态。
 */

import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type {
  TaskRecord,
  TaskStatus,
  TaskOverrides,
  ProgressData,
  MediaInfo,
} from "@/domain";
import { extractFileName, generateTimestampedFilename } from "@/utils/format";
import { generateId } from "@/utils/id";
import { taskService } from "@/services";
import { useSettingsStore } from "./settingsStore";

// 最大日志条目数（每个任务）
const MAX_LOG_ENTRIES = 500;

/** 任务日志条目 */
export interface TaskLogEntry {
  timestamp: Date;
  level: string;
  message: string;
}

/** 空进度数据（新建任务时使用） */
const EMPTY_PROGRESS: ProgressData = {
  percent: 0,
  overallPercent: 0,
  speed: 0,
  downloadedSize: 0,
  totalSize: 0,
  downloadedSegments: 0,
  totalSegments: 0,
  eta: 0,
  currentAction: "",
};

export const useTaskStore = defineStore("task", () => {
  // ==========================================
  // State
  // ==========================================

  const tasks = ref<TaskRecord[]>([]);
  const isLoading = ref(false);
  const isInitialized = ref(false);
  /** 任务日志（仅内存，不持久化） */
  const taskLogs = ref<Map<string, TaskLogEntry[]>>(new Map());

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

  const downloadingTasks = computed(() =>
    tasks.value.filter((t) => t.status === "downloading"),
  );

  const completedTasks = computed(() =>
    tasks.value.filter((t) => t.status === "completed"),
  );

  const failedTasks = computed(() =>
    tasks.value.filter((t) => t.status === "failed"),
  );

  // 最大并发数来自 AppSettings（默认 5，设置页可调）
  const settingsStore = useSettingsStore();
  const canStartMore = computed(
    () =>
      activeTasks.value.length <
      Math.max(1, settingsStore.appSettings.max_concurrent_tasks),
  );

  const hasTasks = computed(() => tasks.value.length > 0);

  const totalProgress = computed(() => {
    const completed = completedTasks.value.length;
    const total = tasks.value.length;
    const percent = total > 0 ? Math.round((completed / total) * 100) : 0;
    return { completed, total, percent };
  });

  /** 按 ID 查找任务 */
  function getTaskById(taskId: string): TaskRecord | undefined {
    return tasks.value.find((t) => t.id === taskId);
  }

  /** @deprecated 使用 getTaskById */
  function getTask(taskId: string): TaskRecord | undefined {
    return getTaskById(taskId);
  }

  // ==========================================
  // Actions — 初始化
  // ==========================================

  /** 初始化：先标记中断任务，再加载全部 */
  async function initialize(): Promise<void> {
    if (isInitialized.value) return;

    isLoading.value = true;
    try {
      await taskService.markActiveTasksInterrupted();
      tasks.value = await taskService.loadAllTasks();
      isInitialized.value = true;
    } catch (error) {
      console.error("Failed to initialize task store:", error);
    } finally {
      isLoading.value = false;
    }
  }

  // ==========================================
  // Actions — 任务 CRUD
  // ==========================================

  /**
   * 添加任务
   * @returns 创建的任务和文件名是否被重命名（冲突时自动加时间戳）
   */
  async function addTask(params: {
    url: string;
    fileName?: string;
    saveDir?: string;
    overrides?: TaskOverrides;
    skipUrlCheck?: boolean;
  }): Promise<{ task: TaskRecord; wasRenamed: boolean }> {
    const now = new Date().toISOString();
    const id = generateId();
    const url = params.url.trim();
    const baseName = params.fileName || extractFileName(url);

    // 文件名冲突检测
    const hasConflict = tasks.value.some(
      (t) => t.saveDir === (params.saveDir || "") && t.fileName === baseName,
    );
    const finalFileName = hasConflict
      ? generateTimestampedFilename(baseName)
      : baseName;

    const task: TaskRecord = {
      id,
      url,
      fileName: finalFileName,
      saveDir: params.saveDir || "",
      status: "pending",
      wasInterrupted: false,
      createdAt: now,
      updatedAt: now,
      progress: { ...EMPTY_PROGRESS },
      overrides: params.overrides ?? null,
    };

    await taskService.createTask(task);
    tasks.value.push(task);

    return { task, wasRenamed: hasConflict };
  }

  /** 同步添加任务（不等待后端响应） */
  function addTaskSync(params: {
    url: string;
    fileName?: string;
    saveDir?: string;
    overrides?: TaskOverrides;
  }): { task: TaskRecord; wasRenamed: boolean } {
    const now = new Date().toISOString();
    const id = generateId();
    const url = params.url.trim();
    const baseName = params.fileName || extractFileName(url);

    const hasConflict = tasks.value.some(
      (t) => t.saveDir === (params.saveDir || "") && t.fileName === baseName,
    );
    const finalFileName = hasConflict
      ? generateTimestampedFilename(baseName)
      : baseName;

    const task: TaskRecord = {
      id,
      url,
      fileName: finalFileName,
      saveDir: params.saveDir || "",
      status: "pending",
      wasInterrupted: false,
      createdAt: now,
      updatedAt: now,
      progress: { ...EMPTY_PROGRESS },
      overrides: params.overrides ?? null,
    };

    tasks.value.push(task);
    taskService.createTask(task).catch(console.error);

    return { task, wasRenamed: hasConflict };
  }

  /** 删除任务 */
  async function removeTask(taskId: string): Promise<void> {
    const index = tasks.value.findIndex((t) => t.id === taskId);
    if (index === -1) return;

    await taskService.deleteTask(taskId);
    tasks.value.splice(index, 1);
    taskLogs.value.delete(taskId);
  }

  /** 重试任务 */
  async function retryTask(taskId: string): Promise<void> {
    const task = tasks.value.find((t) => t.id === taskId);
    if (!task) return;

    await taskService.updateTaskStatus(taskId, "pending");
    task.status = "pending";
    task.error = null;
    task.progress = { ...EMPTY_PROGRESS };
    task.updatedAt = new Date().toISOString();
  }

  /** 更新任务状态（后端 + 内存同步） */
  async function setTaskStatus(
    taskId: string,
    status: TaskStatus,
    error?: string,
  ): Promise<void> {
    const task = tasks.value.find((t) => t.id === taskId);
    if (!task) return;

    task.status = status;
    task.updatedAt = new Date().toISOString();
    if (error !== undefined) task.error = error;
    if (status === "downloading" && !task.startedAt) {
      task.startedAt = new Date().toISOString();
    }
    if (status === "completed") {
      task.completedAt = new Date().toISOString();
    }

    await taskService.updateTaskStatus(taskId, status, error ?? null);
  }

  /** 更新任务进度（仅内存 + 调度防抖落盘） */
  function setTaskProgress(taskId: string, progress: ProgressData): void {
    const task = tasks.value.find((t) => t.id === taskId);
    if (!task) return;

    task.progress = progress;
    task.updatedAt = new Date().toISOString();
    taskService.scheduleProgressFlush(taskId, progress);
  }

  /** 设置任务输出路径 */
  async function setTaskOutputPath(
    taskId: string,
    outputPath: string,
  ): Promise<void> {
    const task = tasks.value.find((t) => t.id === taskId);
    if (!task) return;

    task.outputPath = outputPath;
    task.updatedAt = new Date().toISOString();
    await taskService.updateTaskOutputPath(taskId, outputPath);
  }

  /** 设置任务媒体信息 */
  async function setTaskMediaInfo(
    taskId: string,
    info: MediaInfo,
  ): Promise<void> {
    const task = tasks.value.find((t) => t.id === taskId);
    if (!task) return;

    task.mediaInfo = info;
    task.updatedAt = new Date().toISOString();
    await taskService.updateTaskMediaInfo(taskId, info);
  }

  /** 清除已完成任务 */
  async function clearCompleted(): Promise<void> {
    await taskService.clearFinishedTasks();
    tasks.value = tasks.value.filter((t) => t.status !== "completed");
  }

  /** 清除全部任务 */
  async function clearAll(): Promise<void> {
    await taskService.clearAllTasks();
    tasks.value = [];
    taskLogs.value.clear();
  }

  /** 检查 URL 是否已存在 */
  function checkUrlExists(url: string): TaskRecord | undefined {
    const trimmedUrl = url.trim();
    return tasks.value.find((t) => t.url === trimmedUrl);
  }

  // ==========================================
  // Actions — 日志管理（仅内存）
  // ==========================================

  function addTaskLog(taskId: string, level: string, message: string): void {
    let logs = taskLogs.value.get(taskId);
    if (!logs) {
      logs = [];
      taskLogs.value.set(taskId, logs);
    }

    logs.push({ timestamp: new Date(), level, message });

    if (logs.length > MAX_LOG_ENTRIES) {
      logs.splice(0, logs.length - MAX_LOG_ENTRIES);
    }
  }

  function getTaskLogs(taskId: string): TaskLogEntry[] {
    return taskLogs.value.get(taskId) || [];
  }

  function clearTaskLogs(taskId: string): void {
    taskLogs.value.delete(taskId);
  }

  return {
    // State
    tasks,
    taskLogs,
    isLoading,
    isInitialized,

    // Getters
    activeTasks,
    pendingTasks,
    downloadingTasks,
    completedTasks,
    failedTasks,
    canStartMore,
    hasTasks,
    totalProgress,
    getTaskById,
    getTask,

    // Actions — CRUD
    initialize,
    addTask,
    addTaskSync,
    removeTask,
    retryTask,
    setTaskStatus,
    setTaskProgress,
    setTaskOutputPath,
    setTaskMediaInfo,
    clearCompleted,
    clearAll,
    checkUrlExists,

    // Actions — 日志
    addTaskLog,
    getTaskLogs,
    clearTaskLogs,
  };
});
