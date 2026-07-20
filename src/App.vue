<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { RouterView } from "vue-router";
import { Toaster } from "@/components/ui/toast";
import { RestoreTasksDialog } from "@/components/common";
import { useSettingsStore, useTaskStore } from "@/stores";
import { taskService } from "@/services";
import { useDownloader } from "@/composables";
import type { DownloadTask, TaskStatus } from "@/types";

// 初始化 Stores
const settingsStore = useSettingsStore();
const taskStore = useTaskStore();
const { resumeDownload } = useDownloader();

// 恢复任务弹窗状态
const showRestoreDialog = ref(false);
const recoverableTasks = ref<DownloadTask[]>([]);

/** 可恢复的任务状态（未完成的活跃状态） */
const RECOVERABLE_STATUSES: TaskStatus[] = [
  "paused",
  "downloading",
  "analyzing",
  "merging",
  "muxing",
];

onMounted(async () => {
  // 1. 并行初始化所有 Store（从后端加载数据）
  await Promise.all([settingsStore.loadSettings(), taskStore.initialize()]);

  // 2. 应用主题
  settingsStore.initTheme();

  // 3. 检查可恢复任务并弹窗询问
  try {
    const tasks = await taskService.loadRecoverableTasks();
    const unfinished = tasks.filter((t) =>
      RECOVERABLE_STATUSES.includes(t.status),
    );
    if (unfinished.length > 0) {
      recoverableTasks.value = unfinished;
      showRestoreDialog.value = true;
    }
  } catch (error) {
    console.error("Failed to load recoverable tasks:", error);
  }
});

// 恢复所有中断的任务
const handleRestore = async () => {
  for (const task of recoverableTasks.value) {
    await resumeDownload(task);
  }
};

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
    <RestoreTasksDialog
      v-model:open="showRestoreDialog"
      :tasks="recoverableTasks"
      @confirm="handleRestore"
    />
    <Toaster />
  </div>
</template>
