/**
 * 历史记录状态管理
 *
 * 注意：历史记录就是已完成的任务（completed 状态）
 * 数据来源于 taskStore，不需要单独的 history 表
 */

import { defineStore } from 'pinia';
import { computed } from 'vue';
import { useTaskStore } from './taskStore';
import type { HistoryRecord, DownloadTask } from '@/types';

/**
 * 将 DownloadTask 转换为 HistoryRecord 格式
 */
function taskToHistoryRecord(task: DownloadTask): HistoryRecord {
  return {
    id: task.id,
    url: task.url,
    file_name: task.fileName,
    save_path: task.outputPath || '',
    file_size: task.progress.totalSize || 0,
    duration: 0,
    completed_at: task.completedAt?.toISOString() || new Date().toISOString(),
    task_id: task.id,
  };
}

export const useHistoryStore = defineStore('history', () => {
  // 引用 taskStore
  const taskStore = useTaskStore();

  // Getters - 直接从 taskStore 获取已完成的任务
  const records = computed<HistoryRecord[]>(() => {
    return taskStore.completedTasks
      .map(taskToHistoryRecord)
      .sort((a, b) => new Date(b.completed_at).getTime() - new Date(a.completed_at).getTime());
  });

  const isLoading = computed(() => taskStore.isLoading);
  const isInitialized = computed(() => taskStore.isInitialized);
  const hasRecords = computed(() => records.value.length > 0);
  const recentRecords = computed(() => records.value.slice(0, 10));

  // Actions

  /**
   * 初始化 - 确保 taskStore 已初始化
   */
  async function initialize(): Promise<void> {
    if (!taskStore.isInitialized) {
      await taskStore.initialize();
    }
  }

  /**
   * 加载历史记录（兼容旧代码）
   */
  async function loadHistory(): Promise<void> {
    return initialize();
  }

  /**
   * 删除历史记录（实际上是删除已完成的任务）
   */
  async function deleteRecord(id: string): Promise<void> {
    await taskStore.removeTask(id);
  }

  /**
   * 清除所有历史记录（实际上是清除所有已完成的任务）
   */
  async function clearHistory(): Promise<void> {
    await taskStore.clearCompleted();
  }

  return {
    // State（计算属性，来源于 taskStore）
    records,
    isLoading,
    isInitialized,

    // Getters
    hasRecords,
    recentRecords,

    // Actions
    initialize,
    loadHistory,
    deleteRecord,
    clearHistory,
  };
});
