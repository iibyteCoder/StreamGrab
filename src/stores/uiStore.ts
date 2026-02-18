/**
 * UI 状态管理
 */

import { defineStore } from "pinia";
import { ref, computed } from "vue";

/**
 * Toast 类型
 */
export type ToastType = "success" | "error" | "warning" | "info";

/**
 * Toast 配置
 */
export interface Toast {
  id: string;
  type: ToastType;
  message: string;
  duration: number;
}

/**
 * 生成唯一 ID
 */
function generateId(): string {
  return `${Date.now()}-${Math.random().toString(36).slice(2, 9)}`;
}

export const useUiStore = defineStore("ui", () => {
  // State - Toast
  const toasts = ref<Toast[]>([]);

  // State - 设置面板
  const isSettingsOpen = ref(false);
  const settingsTab = ref<string>("general");

  // State - 流选择器
  const isStreamSelectorOpen = ref(false);
  const streamSelectorTaskId = ref<string | null>(null);

  // State - 任务详情
  const expandedTaskId = ref<string | null>(null);

  // State - 模态框
  const isBatchImportOpen = ref(false);
  const isKeyManagerOpen = ref(false);

  // State - 侧边栏
  const isSidebarCollapsed = ref(false);

  // Getters
  const hasToasts = computed(() => toasts.value.length > 0);

  // Toast Actions
  function addToast(type: ToastType, message: string, duration = 3000): string {
    const id = generateId();
    const toast: Toast = { id, type, message, duration };

    toasts.value.push(toast);

    // 自动移除
    if (duration > 0) {
      setTimeout(() => {
        removeToast(id);
      }, duration);
    }

    return id;
  }

  function removeToast(id: string): void {
    const index = toasts.value.findIndex((t) => t.id === id);
    if (index !== -1) {
      toasts.value.splice(index, 1);
    }
  }

  function clearToasts(): void {
    toasts.value = [];
  }

  // 快捷方法
  function showSuccess(message: string, duration?: number): string {
    return addToast("success", message, duration);
  }

  function showError(message: string, duration?: number): string {
    return addToast("error", message, duration);
  }

  function showWarning(message: string, duration?: number): string {
    return addToast("warning", message, duration);
  }

  function showInfo(message: string, duration?: number): string {
    return addToast("info", message, duration);
  }

  // 设置面板 Actions
  function openSettings(tab = "general"): void {
    settingsTab.value = tab;
    isSettingsOpen.value = true;
  }

  function closeSettings(): void {
    isSettingsOpen.value = false;
  }

  function toggleSettings(): void {
    isSettingsOpen.value = !isSettingsOpen.value;
  }

  function setSettingsTab(tab: string): void {
    settingsTab.value = tab;
  }

  // 流选择器 Actions
  function openStreamSelector(taskId: string): void {
    streamSelectorTaskId.value = taskId;
    isStreamSelectorOpen.value = true;
  }

  function closeStreamSelector(): void {
    isStreamSelectorOpen.value = false;
    streamSelectorTaskId.value = null;
  }

  // 任务详情 Actions
  function expandTask(taskId: string): void {
    expandedTaskId.value = taskId;
  }

  function collapseTask(): void {
    expandedTaskId.value = null;
  }

  function toggleTaskExpand(taskId: string): void {
    if (expandedTaskId.value === taskId) {
      expandedTaskId.value = null;
    } else {
      expandedTaskId.value = taskId;
    }
  }

  function isTaskExpanded(taskId: string): boolean {
    return expandedTaskId.value === taskId;
  }

  // 批量导入 Actions
  function openBatchImport(): void {
    isBatchImportOpen.value = true;
  }

  function closeBatchImport(): void {
    isBatchImportOpen.value = false;
  }

  // 密钥管理 Actions
  function openKeyManager(): void {
    isKeyManagerOpen.value = true;
  }

  function closeKeyManager(): void {
    isKeyManagerOpen.value = false;
  }

  // 侧边栏 Actions
  function toggleSidebar(): void {
    isSidebarCollapsed.value = !isSidebarCollapsed.value;
  }

  function setSidebarCollapsed(collapsed: boolean): void {
    isSidebarCollapsed.value = collapsed;
  }

  return {
    // State
    toasts,
    isSettingsOpen,
    settingsTab,
    isStreamSelectorOpen,
    streamSelectorTaskId,
    expandedTaskId,
    isBatchImportOpen,
    isKeyManagerOpen,
    isSidebarCollapsed,

    // Getters
    hasToasts,

    // Toast Actions
    addToast,
    removeToast,
    clearToasts,
    showSuccess,
    showError,
    showWarning,
    showInfo,

    // 设置面板 Actions
    openSettings,
    closeSettings,
    toggleSettings,
    setSettingsTab,

    // 流选择器 Actions
    openStreamSelector,
    closeStreamSelector,

    // 任务详情 Actions
    expandTask,
    collapseTask,
    toggleTaskExpand,
    isTaskExpanded,

    // 批量导入 Actions
    openBatchImport,
    closeBatchImport,

    // 密钥管理 Actions
    openKeyManager,
    closeKeyManager,

    // 侧边栏 Actions
    toggleSidebar,
    setSidebarCollapsed,
  };
});
