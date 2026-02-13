<script setup lang="ts">
import { onMounted, onUnmounted } from "vue";
import { RouterView } from "vue-router";
import { Toaster } from "@/components/ui/toast";
import { useSettingsStore, useTaskStore, useHistoryStore } from "@/stores";
import { taskService } from "@/services";

// 初始化 Stores
const settingsStore = useSettingsStore();
const taskStore = useTaskStore();
const historyStore = useHistoryStore();

onMounted(async () => {
  // 1. 并行初始化所有 Store（从后端加载数据）
  await Promise.all([
    settingsStore.loadSettings(),
    taskStore.initialize(),
    historyStore.initialize(),
  ]);

  // 2. 应用主题
  settingsStore.initTheme();

  // 3. 检查可恢复任务（可选）
  const recoverableTasks = taskStore.tasks.filter(
    (t) => t.status === "downloading" || t.status === "paused"
  );

  if (recoverableTasks.length > 0) {
    console.log(`Found ${recoverableTasks.length} recoverable tasks`);
    // TODO: 显示恢复对话框
  }
});

// 应用关闭时标记中断的任务
onUnmounted(async () => {
  try {
    await taskService.markActiveTasksInterrupted();
  } catch (error) {
    console.error("Failed to mark tasks as interrupted:", error);
  }
});
</script>

<template>
  <div class="h-screen w-screen flex flex-col bg-background">
    <div class="flex-1 min-h-0 overflow-hidden">
      <RouterView />
    </div>
    <Toaster />
  </div>
</template>
