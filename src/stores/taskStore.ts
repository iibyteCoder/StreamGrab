/**
 * 任务状态管理
 */

import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { DownloadTask, TaskStatus, TaskProgressData } from '@/types';
import { extractFileName } from '@/utils/format';
import { MAX_CONCURRENT_TASKS } from '@/utils/constants';

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
    currentAction: '',
  };
}

export const useTaskStore = defineStore('task', () => {
  // State
  const tasks = ref<DownloadTask[]>([]);
  const maxConcurrent = ref(MAX_CONCURRENT_TASKS);

  // Getters
  const activeTasks = computed(() =>
    tasks.value.filter((t) =>
      ['downloading', 'analyzing', 'merging', 'muxing'].includes(t.status)
    )
  );

  const pendingTasks = computed(() =>
    tasks.value.filter((t) => t.status === 'pending')
  );

  const completedTasks = computed(() =>
    tasks.value.filter((t) => t.status === 'completed')
  );

  const failedTasks = computed(() =>
    tasks.value.filter((t) => t.status === 'failed')
  );

  const downloadingTasks = computed(() =>
    tasks.value.filter((t) => t.status === 'downloading')
  );

  const canStartMore = computed(
    () => activeTasks.value.length < maxConcurrent.value
  );

  const hasTasks = computed(() => tasks.value.length > 0);

  const totalProgress = computed(() => {
    const completed = completedTasks.value.length;
    const total = tasks.value.length;
    const percent = total > 0 ? Math.round((completed / total) * 100) : 0;
    return { completed, total, percent };
  });

  // Actions
  function addTask(url: string, fileName?: string, saveDir?: string): DownloadTask {
    const task: DownloadTask = {
      id: generateId(),
      url: url.trim(),
      fileName: fileName || extractNameFromUrl(url),
      saveDir: saveDir || '',
      status: 'pending',
      progress: createEmptyProgress(),
      createdAt: new Date(),
      updatedAt: new Date(),
    };

    tasks.value.push(task);
    return task;
  }

  function getTask(taskId: string): DownloadTask | undefined {
    return tasks.value.find((t) => t.id === taskId);
  }

  function updateTaskStatus(taskId: string, status: TaskStatus): void {
    const task = tasks.value.find((t) => t.id === taskId);
    if (task) {
      task.status = status;
      task.updatedAt = new Date();

      // 更新相关时间戳
      if (status === 'downloading' && !task.startedAt) {
        task.startedAt = new Date();
      }
      if (status === 'completed') {
        task.completedAt = new Date();
      }
    }
  }

  function updateTaskProgress(taskId: string, progress: Partial<TaskProgressData>): void {
    const task = tasks.value.find((t) => t.id === taskId);
    if (task) {
      task.progress = {
        ...task.progress,
        ...progress,
      };
      task.updatedAt = new Date();
    }
  }

  function updateTaskError(taskId: string, error: string): void {
    const task = tasks.value.find((t) => t.id === taskId);
    if (task) {
      task.error = error;
      task.status = 'failed';
      task.updatedAt = new Date();
    }
  }

  function updateTaskOutput(taskId: string, outputPath: string): void {
    const task = tasks.value.find((t) => t.id === taskId);
    if (task) {
      task.outputPath = outputPath;
      task.updatedAt = new Date();
    }
  }

  function retryTask(taskId: string): void {
    const task = tasks.value.find((t) => t.id === taskId);
    if (task) {
      task.status = 'pending';
      task.error = undefined;
      task.progress = createEmptyProgress();
      task.updatedAt = new Date();
    }
  }

  function removeTask(taskId: string): void {
    const index = tasks.value.findIndex((t) => t.id === taskId);
    if (index !== -1) {
      tasks.value.splice(index, 1);
    }
  }

  function clearCompleted(): void {
    tasks.value = tasks.value.filter((t) => t.status !== 'completed');
  }

  function clearFailed(): void {
    tasks.value = tasks.value.filter((t) => t.status !== 'failed');
  }

  function clearAll(): void {
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

  return {
    // State
    tasks,
    maxConcurrent,

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
    addTask,
    getTask,
    updateTaskStatus,
    updateTaskProgress,
    updateTaskError,
    updateTaskOutput,
    retryTask,
    removeTask,
    clearCompleted,
    clearFailed,
    clearAll,
    reorderTasks,
    setMaxConcurrent,
  };
});
