/**
 * 历史记录状态管理
 *
 * 任务终态快照，独立于任务表（清除任务不删除历史）
 */

import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type { HistoryRecord } from "@/domain";
import { historyService } from "@/services";

export const useHistoryStore = defineStore("history", () => {
  // ========================================
  // State
  // ========================================

  const records = ref<HistoryRecord[]>([]);
  const loaded = ref(false);

  // ========================================
  // Computed
  // ========================================

  const count = computed(() => records.value.length);

  // ========================================
  // Actions
  // ========================================

  /** 加载全部历史（按完成时间倒序） */
  async function loadHistory(): Promise<void> {
    try {
      records.value = await historyService.loadHistory();
      loaded.value = true;
    } catch (e) {
      console.error("Failed to load history:", e);
    }
  }

  /** 删除单条记录 */
  async function removeRecord(id: number): Promise<void> {
    await historyService.deleteHistoryRecord(id);
    records.value = records.value.filter((r) => r.id !== id);
  }

  /** 清空全部历史 */
  async function clearAll(): Promise<void> {
    await historyService.clearHistory();
    records.value = [];
  }

  return {
    // State
    records,
    loaded,

    // Computed
    count,

    // Actions
    loadHistory,
    removeRecord,
    clearAll,
  };
});
