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
  TaskCreateParams,
  MediaInfo,
} from "@/types/task";
import { extractFileName, generateTimestampedFilename } from "@/utils/format";
import { MAX_CONCURRENT_TASKS } from "@/utils/constants";
import { taskService, configService } from "@/services";

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
      tasks.value = await taskService.loadAllTasks();
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
    const now = new Date();
    const id = generateId();

    const task: DownloadTask = {
      id,
      url: url.trim(),
      fileName: fileName || extractNameFromUrl(url),
      saveDir: saveDir || "",
      status: "pending",
      error: undefined,
      outputPath: undefined,
      wasInterrupted: false,
      createdAt: now,
      updatedAt: now,
      // 进度（扁平化）
      progressPercent: 0,
      progressSpeed: 0,
      progressDownloadedSize: 0,
      progressTotalSize: 0,
      progressDownloadedSegments: 0,
      progressTotalSegments: 0,
      progressEta: 0,
      progressCurrentAction: "",
      // 媒体信息（扁平化）
      mediaIsLive: false,
      mediaIsEncrypted: false,
      // 运行时配置
      config: {},
    };

    // 持久化到后端
    const params: TaskCreateParams = {
      id: task.id,
      url: task.url,
      fileName: task.fileName,
      saveDir: task.saveDir,
      status: task.status,
      createdAt: task.createdAt.toISOString(),
      updatedAt: task.updatedAt.toISOString(),
    };
    await taskService.createTask(params);

    // 更新缓存
    tasks.value.push(task);
    return task;
  }

  function getTask(taskId: string): DownloadTask | undefined {
    return tasks.value.find((t) => t.id === taskId);
  }

  /**
   * 检查文件名是否冲突（同目录下已有同名任务）
   */
  function checkFilenameConflict(saveDir: string, fileName: string): boolean {
    return tasks.value.some(
      (t) => t.saveDir === saveDir && t.fileName === fileName,
    );
  }

  /**
   * 检查 URL 是否已存在
   */
  function checkUrlExists(url: string): DownloadTask | undefined {
    const trimmedUrl = url.trim();
    return tasks.value.find((t) => t.url === trimmedUrl);
  }

  /**
   * 同步添加任务 - 立即返回，后台持久化
   * @param url 下载链接
   * @param fileName 文件名（可选）
   * @param saveDir 保存目录（可选）
   * @returns 任务对象和是否重命名了文件名
   */
  function addTaskSync(
    url: string,
    fileName?: string,
    saveDir?: string,
  ): { task: DownloadTask; wasRenamed: boolean } {
    const now = new Date();
    const id = generateId();
    const targetDir = saveDir || "";
    const baseName = fileName || extractNameFromUrl(url);

    // 检查文件名冲突，如果冲突则生成唯一文件名
    const hasConflict = checkFilenameConflict(targetDir, baseName);
    const finalFileName = hasConflict
      ? generateTimestampedFilename(baseName)
      : baseName;

    const task: DownloadTask = {
      id,
      url: url.trim(),
      fileName: finalFileName,
      saveDir: targetDir,
      status: "pending",
      error: undefined,
      outputPath: undefined,
      wasInterrupted: false,
      createdAt: now,
      updatedAt: now,
      // 进度（扁平化）
      progressPercent: 0,
      progressSpeed: 0,
      progressDownloadedSize: 0,
      progressTotalSize: 0,
      progressDownloadedSegments: 0,
      progressTotalSegments: 0,
      progressEta: 0,
      progressCurrentAction: "",
      // 媒体信息（扁平化）
      mediaIsLive: false,
      mediaIsEncrypted: false,
      // 运行时配置
      config: {},
    };

    // 更新缓存
    tasks.value.push(task);

    // 后台持久化
    const params: TaskCreateParams = {
      id: task.id,
      url: task.url,
      fileName: task.fileName,
      saveDir: task.saveDir,
      status: task.status,
      createdAt: task.createdAt.toISOString(),
      updatedAt: task.updatedAt.toISOString(),
    };
    taskService.createTask(params).catch(console.error);

    return { task, wasRenamed: hasConflict };
  }

  /**
   * 更新任务配置 - 仅内存
   */
  function updateTaskConfig(
    taskId: string,
    config: Record<string, unknown>,
  ): void {
    const task = tasks.value.find((t) => t.id === taskId);
    if (!task) return;

    // 更新配置（存储到 task.config 对象）
    if (!task.config) {
      task.config = {};
    }
    Object.assign(task.config, config);
    task.updatedAt = new Date();
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

    // 同步到后端
    taskService.updateTaskStatus(taskId, status).catch(console.error);
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

    // 更新扁平化字段
    // 优先使用 overallPercent（视频+音频总体进度），避免进度跳动
    if (progress.overallPercent !== undefined) {
      task.progressPercent = progress.overallPercent;
    } else if (progress.percent !== undefined) {
      task.progressPercent = progress.percent;
    }
    if (progress.speed !== undefined) task.progressSpeed = progress.speed;
    if (progress.downloadedSize !== undefined)
      task.progressDownloadedSize = progress.downloadedSize;
    if (progress.totalSize !== undefined)
      task.progressTotalSize = progress.totalSize;
    // 优先使用 totalDownloadedSegments（视频+音频总已下载分片）
    if (progress.totalDownloadedSegments !== undefined) {
      task.progressDownloadedSegments = progress.totalDownloadedSegments;
    } else if (progress.downloadedSegments !== undefined) {
      task.progressDownloadedSegments = progress.downloadedSegments;
    }
    if (progress.totalSegments !== undefined)
      task.progressTotalSegments = progress.totalSegments;
    if (progress.eta !== undefined) task.progressEta = progress.eta;
    if (progress.currentAction !== undefined)
      task.progressCurrentAction = progress.currentAction;
    task.updatedAt = new Date();

    // 收集进度更新，批量同步
    pendingProgressUpdates.set(taskId, {
      percent: task.progressPercent,
      speed: task.progressSpeed,
      downloadedSize: task.progressDownloadedSize,
      totalSize: task.progressTotalSize,
      downloadedSegments: task.progressDownloadedSegments,
      totalSegments: task.progressTotalSegments,
      eta: task.progressEta,
      currentAction: task.progressCurrentAction,
    });

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
    taskService.updateTaskOutputPath(taskId, outputPath).catch(console.error);
  }

  /**
   * 更新任务媒体信息 - 同步到后端
   */
  function updateTaskMediaInfo(
    taskId: string,
    mediaInfo: Partial<MediaInfo>,
  ): void {
    const task = tasks.value.find((t) => t.id === taskId);
    if (!task) return;

    // 更新扁平化字段
    if (mediaInfo.resolution !== undefined)
      task.mediaResolution = mediaInfo.resolution;
    if (mediaInfo.width !== undefined) task.mediaWidth = mediaInfo.width;
    if (mediaInfo.height !== undefined) task.mediaHeight = mediaInfo.height;
    if (mediaInfo.frameRate !== undefined)
      task.mediaFrameRate = mediaInfo.frameRate;
    if (mediaInfo.videoCodec !== undefined)
      task.mediaVideoCodec = mediaInfo.videoCodec;
    if (mediaInfo.videoRange !== undefined)
      task.mediaVideoRange = mediaInfo.videoRange;
    if (mediaInfo.audioCodec !== undefined)
      task.mediaAudioCodec = mediaInfo.audioCodec;
    if (mediaInfo.audioChannels !== undefined)
      task.mediaAudioChannels = mediaInfo.audioChannels;
    if (mediaInfo.audioLanguage !== undefined)
      task.mediaAudioLanguage = mediaInfo.audioLanguage;
    if (mediaInfo.duration !== undefined)
      task.mediaDuration = mediaInfo.duration;
    if (mediaInfo.segmentCount !== undefined)
      task.mediaSegmentCount = mediaInfo.segmentCount;
    if (mediaInfo.isLive !== undefined) task.mediaIsLive = mediaInfo.isLive;
    if (mediaInfo.isEncrypted !== undefined)
      task.mediaIsEncrypted = mediaInfo.isEncrypted;
    if (mediaInfo.fileFormat !== undefined)
      task.mediaFileFormat = mediaInfo.fileFormat;
    task.updatedAt = new Date();

    // 同步到后端
    taskService.updateTaskMediaInfo(taskId, mediaInfo).catch(console.error);
  }

  function retryTask(taskId: string): void {
    const task = tasks.value.find((t) => t.id === taskId);
    if (!task) return;

    task.status = "pending";
    task.error = undefined;
    // 重置进度
    task.progressPercent = 0;
    task.progressSpeed = 0;
    task.progressDownloadedSize = 0;
    task.progressTotalSize = 0;
    task.progressDownloadedSegments = 0;
    task.progressTotalSegments = 0;
    task.progressEta = 0;
    task.progressCurrentAction = "";
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

  /**
   * 检查文件是否存在（实时检查，不缓存）
   */
  async function checkFileExists(taskId: string): Promise<boolean> {
    const task = tasks.value.find((t) => t.id === taskId);
    if (!task?.outputPath) return false;

    try {
      return await configService.fileExists(task.outputPath);
    } catch {
      return false;
    }
  }

  // ==========================================
  // Actions - 日志管理（仅内存）
  // ==========================================

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
    checkUrlExists,
    getTask,
    updateTaskConfig,
    updateTaskStatus,
    updateTaskProgress,
    updateTaskError,
    updateTaskOutput,
    updateTaskMediaInfo,
    retryTask,
    removeTask,
    clearCompleted,
    clearFailed,
    clearAll,
    reorderTasks,
    setMaxConcurrent,
    checkFileExists,

    // Log Actions
    addTaskLog,
    getTaskLogs,
    clearTaskLogs,
  };
});
