<script setup lang="ts">
/**
 * TaskCard 任务卡片组件
 * 显示单个下载任务的信息和操作
 */

import { computed } from 'vue';
import { AppProgress, AppButton } from '@/components/common';
import { useTasks, useDownloader } from '@/composables';
import { formatSpeed, formatFileSize, formatDuration } from '@/utils/format';
import { TASK_STATUS_CONFIG } from '@/utils/constants';
import type { DownloadTask } from '@/types';

interface Props {
  task: DownloadTask;
  compact?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  compact: false,
});

const emit = defineEmits<{
  (e: 'click', task: DownloadTask): void;
  (e: 'contextmenu', event: MouseEvent, task: DownloadTask): void;
}>();

const { getTask, removeTask } = useTasks();
const { startDownload, stopDownload, pauseDownload, resumeDownload, retryDownload } = useDownloader();

// 状态配置
const statusConfig = computed(() => {
  return TASK_STATUS_CONFIG[props.task.status] || TASK_STATUS_CONFIG.pending;
});

// 进度颜色
const progressVariant = computed(() => {
  switch (props.task.status) {
    case 'completed':
      return 'success';
    case 'failed':
      return 'error';
    case 'downloading':
      return 'default';
    default:
      return 'default';
  }
});

// 格式化数据
const speedText = computed(() => {
  if (props.task.status !== 'downloading') return '';
  return formatSpeed(props.task.progress.speed);
});

const sizeText = computed(() => {
  const downloaded = props.task.progress.downloadedSize;
  const total = props.task.progress.totalSize;

  if (total > 0) {
    return `${formatFileSize(downloaded)} / ${formatFileSize(total)}`;
  }
  return formatFileSize(downloaded);
});

const etaText = computed(() => {
  if (props.task.status !== 'downloading' || !props.task.progress.eta) return '';
  return formatDuration(props.task.progress.eta);
});

// 操作处理
const handleStart = async () => {
  await startDownload(props.task);
};

const handlePause = async () => {
  await pauseDownload(props.task.id);
};

const handleResume = async () => {
  await resumeDownload(props.task);
};

const handleStop = async () => {
  await stopDownload(props.task.id);
};

const handleRetry = async () => {
  await retryDownload(props.task);
};

const handleRemove = () => {
  removeTask(props.task.id);
};

const handleClick = () => {
  emit('click', props.task);
};

const handleContextmenu = (event: MouseEvent) => {
  emit('contextmenu', event, props.task);
};
</script>

<template>
  <div
    class="task-card bg-bg-surface border border-border-default rounded-lg p-4 hover:border-border-hover transition-all duration-200"
    :class="{ 'opacity-75': task.status === 'cancelled' }"
    @click="handleClick"
    @contextmenu="handleContextmenu"
  >
    <!-- 顶部：文件名和状态 -->
    <div class="flex items-start justify-between gap-3 mb-3">
      <div class="flex-1 min-w-0">
        <h4 class="text-sm font-medium text-text-primary truncate">
          {{ task.fileName || '未命名文件' }}
        </h4>
        <p class="text-xs text-text-muted truncate mt-0.5">
          {{ task.url }}
        </p>
      </div>

      <!-- 状态标签 -->
      <span
        class="flex-shrink-0 px-2 py-0.5 rounded text-xs font-medium"
        :style="{
          backgroundColor: `${statusConfig.color}20`,
          color: statusConfig.color,
        }"
      >
        {{ statusConfig.text }}
      </span>
    </div>

    <!-- 进度条 -->
    <div class="mb-3">
      <AppProgress
        :percent="task.progress.percent"
        :variant="progressVariant"
        size="md"
      />
    </div>

    <!-- 底部：进度信息和操作 -->
    <div class="flex items-center justify-between">
      <!-- 进度信息 -->
      <div class="flex items-center gap-3 text-xs text-text-secondary">
        <span v-if="speedText" class="flex items-center gap-1">
          <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7h8m0 0v8m0-8l-8 8-4-4-6 6" />
          </svg>
          {{ speedText }}
        </span>
        <span v-if="sizeText">{{ sizeText }}</span>
        <span v-if="etaText">剩余 {{ etaText }}</span>
      </div>

      <!-- 操作按钮 -->
      <div class="flex items-center gap-1">
        <!-- 下载中：暂停 -->
        <button
          v-if="task.status === 'downloading'"
          class="p-1.5 rounded hover:bg-bg-elevated text-text-secondary hover:text-text-primary transition-colors"
          title="暂停"
          @click.stop="handlePause"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 9v6m4-6v6m7-3a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        </button>

        <!-- 暂停/等待中：开始 -->
        <button
          v-if="task.status === 'paused' || task.status === 'pending'"
          class="p-1.5 rounded hover:bg-bg-elevated text-text-secondary hover:text-accent-primary transition-colors"
          title="开始"
          @click.stop="task.status === 'paused' ? handleResume() : handleStart()"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        </button>

        <!-- 失败：重试 -->
        <button
          v-if="task.status === 'failed'"
          class="p-1.5 rounded hover:bg-bg-elevated text-text-secondary hover:text-accent-primary transition-colors"
          title="重试"
          @click.stop="handleRetry"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
        </button>

        <!-- 下载中/暂停：停止 -->
        <button
          v-if="task.status === 'downloading' || task.status === 'paused'"
          class="p-1.5 rounded hover:bg-bg-elevated text-text-secondary hover:text-accent-error transition-colors"
          title="停止"
          @click.stop="handleStop"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 10a1 1 0 011-1h4a1 1 0 011 1v4a1 1 0 01-1 1h-4a1 1 0 01-1-1v-4z" />
          </svg>
        </button>

        <!-- 已完成/已取消/失败：删除 -->
        <button
          v-if="['completed', 'cancelled', 'failed'].includes(task.status)"
          class="p-1.5 rounded hover:bg-bg-elevated text-text-secondary hover:text-accent-error transition-colors"
          title="删除"
          @click.stop="handleRemove"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
          </svg>
        </button>
      </div>
    </div>

    <!-- 错误信息 -->
    <div
      v-if="task.status === 'failed' && task.error"
      class="mt-2 p-2 bg-accent-error/10 rounded text-xs text-accent-error"
    >
      {{ task.error }}
    </div>
  </div>
</template>

<style scoped>
.task-card {
  cursor: pointer;
}

.task-card:hover {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}
</style>
