/**
 * 历史记录状态管理
 */

import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { invokeTauri } from '@/services/tauri';
import type { HistoryRecord } from '@/types';

export const useHistoryStore = defineStore('history', () => {
  // State
  const records = ref<HistoryRecord[]>([]);
  const isLoading = ref(false);

  // Getters
  const hasRecords = computed(() => records.value.length > 0);
  const recentRecords = computed(() => records.value.slice(0, 10));

  // Actions

  /**
   * 加载历史记录
   */
  async function loadHistory(): Promise<void> {
    isLoading.value = true;
    try {
      const data = await invokeTauri<HistoryRecord[]>('load_history');
      records.value = data;
    } catch (error) {
      console.error('Failed to load history:', error);
      records.value = [];
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * 添加历史记录
   */
  async function addRecord(record: HistoryRecord): Promise<void> {
    try {
      await invokeTauri('add_history_record', { record });
      // 添加到本地列表开头
      records.value.unshift(record);
      // 限制显示数量
      if (records.value.length > 100) {
        records.value.pop();
      }
    } catch (error) {
      console.error('Failed to add history record:', error);
    }
  }

  /**
   * 清除所有历史记录
   */
  async function clearHistory(): Promise<void> {
    try {
      await invokeTauri('clear_history');
      records.value = [];
    } catch (error) {
      console.error('Failed to clear history:', error);
    }
  }

  /**
   * 删除单条历史记录
   */
  async function deleteRecord(id: string): Promise<void> {
    try {
      await invokeTauri('delete_history_record', { id });
      records.value = records.value.filter(r => r.id !== id);
    } catch (error) {
      console.error('Failed to delete history record:', error);
    }
  }

  /**
   * 从任务创建历史记录
   */
  function createRecordFromTask(task: {
    id: string;
    url: string;
    fileName: string;
    outputPath?: string;
    progress: { totalSize: number };
  }): HistoryRecord {
    return {
      id: task.id,
      url: task.url,
      file_name: task.fileName,
      save_path: task.outputPath || '',
      file_size: task.progress.totalSize,
      duration: 0,
      completed_at: new Date().toISOString(),
    };
  }

  return {
    // State
    records,
    isLoading,

    // Getters
    hasRecords,
    recentRecords,

    // Actions
    loadHistory,
    addRecord,
    clearHistory,
    deleteRecord,
    createRecordFromTask,
  };
});
