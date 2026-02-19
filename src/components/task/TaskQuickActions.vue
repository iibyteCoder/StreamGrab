<script setup lang="ts">
/**
 * TaskQuickActions - 任务快速操作按钮组
 * 纯展示组件：悬停显示的操作按钮
 */

import { computed } from "vue";
import { AppIcon } from "@/components/common";
import type { DownloadTask } from "@/types";

interface Props {
  task: DownloadTask;
  fileExists: boolean;
  hasLogs: boolean;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: "openFolder"): void;
  (e: "openFile"): void;
  (e: "showLogs"): void;
  (e: "pause"): void;
  (e: "start"): void;
  (e: "resume"): void;
  (e: "retry"): void;
  (e: "stop"): void;
  (e: "delete"): void;
}>();

// 状态判断
const isDownloading = computed(() => props.task.status === "downloading");
const isPaused = computed(() => props.task.status === "paused");
const isPending = computed(() => props.task.status === "pending");
const isFailed = computed(() => props.task.status === "failed");
const isCompleted = computed(() => props.task.status === "completed");
const canStop = computed(() => isDownloading.value || isPaused.value);
const canDelete = computed(() =>
  ["cancelled", "failed", "completed"].includes(props.task.status),
);
</script>

<template>
  <div
    class="absolute bottom-2 right-2 flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity"
  >
    <!-- 打开文件夹 -->
    <button
      v-if="task.saveDir"
      class="action-btn"
      title="打开目录"
      @click.stop="emit('openFolder')"
    >
      <AppIcon name="FolderOpen" :size="15" />
    </button>

    <!-- 播放文件 -->
    <button
      v-if="isCompleted && fileExists"
      class="action-btn text-green-500 hover:text-green-600"
      title="播放"
      @click.stop="emit('openFile')"
    >
      <AppIcon name="Play" :size="15" />
    </button>

    <!-- 查看日志 -->
    <button
      v-if="hasLogs || isDownloading"
      class="action-btn"
      title="日志"
      @click.stop="emit('showLogs')"
    >
      <AppIcon name="FileText" :size="15" />
    </button>

    <!-- 暂停 -->
    <button
      v-if="isDownloading"
      class="action-btn"
      title="暂停"
      @click.stop="emit('pause')"
    >
      <AppIcon name="Pause" :size="15" />
    </button>

    <!-- 开始/继续 -->
    <button
      v-if="isPaused"
      class="action-btn text-primary"
      title="继续"
      @click.stop="emit('resume')"
    >
      <AppIcon name="Play" :size="15" />
    </button>
    <button
      v-else-if="isPending"
      class="action-btn text-primary"
      title="开始"
      @click.stop="emit('start')"
    >
      <AppIcon name="Play" :size="15" />
    </button>

    <!-- 重试 -->
    <button
      v-if="isFailed"
      class="action-btn text-primary"
      title="重试"
      @click.stop="emit('retry')"
    >
      <AppIcon name="RefreshCw" :size="15" />
    </button>

    <!-- 停止 -->
    <button
      v-if="canStop"
      class="action-btn text-destructive"
      title="停止"
      @click.stop="emit('stop')"
    >
      <AppIcon name="Square" :size="15" />
    </button>

    <!-- 删除 -->
    <button
      v-if="canDelete"
      class="action-btn text-destructive"
      title="删除"
      @click.stop="emit('delete')"
    >
      <AppIcon name="Trash2" :size="15" />
    </button>
  </div>
</template>

<style scoped>
.action-btn {
  @apply p-1.5 rounded-md hover:bg-accent text-muted-foreground hover:text-foreground transition-colors;
}
</style>
