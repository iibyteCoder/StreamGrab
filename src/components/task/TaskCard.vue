<script setup lang="ts">
/**
 * TaskCard 任务卡片组件
 * 显示单个下载任务的信息和操作
 */

import { computed, ref, onMounted, watch } from 'vue';
import { AppProgress } from '@/components/common';
import { useTasks, useDownloader } from '@/composables';
import { useTaskStore } from '@/stores';
import { configService } from '@/services';
import { formatSpeed, formatFileSize, formatDuration } from '@/utils/format';
import { TASK_STATUS_CONFIG } from '@/utils/constants';
import { LogViewer } from '@/components/task';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
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

const { removeTask } = useTasks();
const { startDownload, stopDownload, pauseDownload, resumeDownload, retryDownload } = useDownloader();
const taskStore = useTaskStore();

// 日志查看器状态
const showLogViewer = ref(false);

// 删除确认对话框
const showDeleteDialog = ref(false);
const deleteWithFile = ref(false);
const isDeleting = ref(false);

// 文件存在状态
const fileExists = ref<boolean | null>(null);
const folderExists = ref<boolean | null>(null);

// 检查文件是否存在
const checkFileExists = async () => {
  if (props.task.outputPath) {
    try {
      fileExists.value = await configService.fileExists(props.task.outputPath);
    } catch {
      fileExists.value = false;
    }
  } else {
    fileExists.value = null;
  }
};

// 检查文件夹是否存在
const checkFolderExists = async () => {
  if (props.task.saveDir) {
    try {
      folderExists.value = await configService.fileExists(props.task.saveDir);
    } catch {
      folderExists.value = false;
    }
  } else {
    folderExists.value = null;
  }
};

// 组件挂载时检查文件
onMounted(() => {
  if (props.task.status === 'completed') {
    checkFileExists();
  }
  checkFolderExists();
});

// 监听任务状态变化
watch(() => props.task.status, (newStatus) => {
  if (newStatus === 'completed') {
    checkFileExists();
  }
});

// 监听输出路径变化
watch(() => props.task.outputPath, () => {
  if (props.task.status === 'completed') {
    checkFileExists();
  }
});

// 检查是否有日志
const hasLogs = computed(() => {
  const logs = taskStore.getTaskLogs(props.task.id);
  return logs.length > 0;
});

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
  if (downloaded > 0) {
    return formatFileSize(downloaded);
  }
  return '';
});

const etaText = computed(() => {
  if (props.task.status !== 'downloading' || !props.task.progress.eta) return '';
  return formatDuration(props.task.progress.eta);
});

// 分片进度文本
const segmentsText = computed(() => {
  const downloaded = props.task.progress.downloadedSegments || 0;
  const total = props.task.progress.totalSegments || 0;
  if (total > 0) {
    return `${downloaded}/${total}`;
  }
  return '';
});

// 是否显示文件丢失提示
const showFileMissingHint = computed(() => {
  return props.task.status === 'completed' && fileExists.value === false;
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

// 打开删除确认对话框
const handleRemoveClick = () => {
  // 如果任务已完成且文件存在，显示选项
  if (props.task.status === 'completed' && fileExists.value) {
    deleteWithFile.value = false;
    showDeleteDialog.value = true;
  } else {
    // 其他情况直接删除
    performDelete(false);
  }
};

// 执行删除
const performDelete = async (withFile: boolean) => {
  isDeleting.value = true;
  try {
    // 如果需要删除文件，调用后端删除
    if (withFile && props.task.outputPath) {
      try {
        await configService.deleteFileOrFolder(props.task.outputPath);
      } catch (error) {
        console.error('Failed to delete file:', error);
        // 文件删除失败不影响记录删除
      }
    }
    removeTask(props.task.id);
  } finally {
    isDeleting.value = false;
    showDeleteDialog.value = false;
  }
};

// 确认删除
const handleConfirmDelete = () => {
  performDelete(deleteWithFile.value);
};

/**
 * 打开保存目录
 */
const handleOpenFolder = async () => {
  const path = props.task.saveDir;
  if (path) {
    try {
      await configService.openInExplorer(path);
    } catch (error) {
      console.error('Failed to open folder:', error);
    }
  }
};

/**
 * 打开下载完成的文件
 */
const handleOpenFile = async () => {
  const path = props.task.outputPath;
  if (path && fileExists.value) {
    try {
      await configService.openInExplorer(path);
    } catch (error) {
      console.error('Failed to open file:', error);
    }
  }
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
    class="task-card bg-bg-surface border border-border-default rounded-lg p-3 hover:border-border-hover transition-all duration-200"
    :class="{ 'opacity-75': task.status === 'cancelled' }"
    @click="handleClick"
    @contextmenu="handleContextmenu"
  >
    <!-- 主内容区 -->
    <div class="flex items-center gap-3">
      <!-- 左侧：文件信息 -->
      <div class="flex-1 min-w-0">
        <!-- 文件名和状态 -->
        <div class="flex items-center gap-2">
          <h4 class="text-sm font-medium text-text-primary truncate flex-1">
            {{ task.fileName || '未命名文件' }}
          </h4>
          <span
            class="flex-shrink-0 px-1.5 py-0.5 rounded text-xs"
            :style="{
              backgroundColor: `${statusConfig?.color ?? '#888'}20`,
              color: statusConfig?.color ?? '#888',
            }"
          >
            {{ statusConfig?.text ?? '未知' }}
          </span>
        </div>

        <!-- 进度条 -->
        <div class="mt-1.5">
          <AppProgress
            :percent="task.progress.percent"
            :variant="progressVariant"
            size="sm"
          />
        </div>

        <!-- 进度信息行 -->
        <div class="flex items-center gap-2 mt-1 text-xs text-text-secondary">
          <span v-if="sizeText">{{ sizeText }}</span>
          <span v-if="segmentsText" class="text-text-muted">{{ segmentsText }}</span>
          <span v-if="speedText" class="text-accent-primary font-medium">{{ speedText }}</span>
          <span v-if="etaText">剩余{{ etaText }}</span>

          <!-- 文件丢失提示 -->
          <span v-if="showFileMissingHint" class="text-amber-500 flex items-center gap-0.5">
            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
            </svg>
            文件已移除
          </span>
        </div>
      </div>

      <!-- 右侧：操作按钮 -->
      <div class="flex items-center gap-0.5 flex-shrink-0">
        <!-- 打开文件夹 -->
        <button
          v-if="task.saveDir && folderExists !== false"
          class="p-1.5 rounded hover:bg-bg-elevated text-text-secondary hover:text-accent-primary transition-colors"
          title="打开目录"
          @click.stop="handleOpenFolder"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
          </svg>
        </button>

        <!-- 播放文件（完成后且文件存在） -->
        <button
          v-if="task.status === 'completed' && fileExists"
          class="p-1.5 rounded hover:bg-bg-elevated text-text-secondary hover:text-accent-success transition-colors"
          title="播放"
          @click.stop="handleOpenFile"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        </button>

        <!-- 查看日志 -->
        <button
          v-if="hasLogs || task.status === 'downloading'"
          class="p-1.5 rounded hover:bg-bg-elevated text-text-secondary hover:text-text-primary transition-colors"
          title="日志"
          @click.stop="showLogViewer = true"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
        </button>

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

        <!-- 已取消/失败：删除 -->
        <button
          v-if="['cancelled', 'failed'].includes(task.status)"
          class="p-1.5 rounded hover:bg-bg-elevated text-text-secondary hover:text-accent-error transition-colors"
          title="删除"
          @click.stop="handleRemoveClick"
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

    <!-- 日志查看器 -->
    <LogViewer
      v-model:open="showLogViewer"
      :task-id="task.id"
    />

    <!-- 删除确认对话框 -->
    <Dialog v-model:open="showDeleteDialog">
      <DialogContent class="sm:max-w-[400px]">
        <DialogHeader>
          <DialogTitle>确认删除</DialogTitle>
          <DialogDescription>
            确定要删除此任务记录吗？
          </DialogDescription>
        </DialogHeader>

        <div class="py-4">
          <label class="flex items-center gap-2 cursor-pointer select-none">
            <input
              type="checkbox"
              v-model="deleteWithFile"
              class="w-4 h-4 rounded border-border-default accent-primary"
            />
            <span class="text-sm">同时删除下载的文件</span>
          </label>
          <p v-if="deleteWithFile && task.outputPath" class="mt-2 text-xs text-muted-foreground truncate">
            {{ task.outputPath }}
          </p>
        </div>

        <DialogFooter>
          <Button variant="outline" @click="showDeleteDialog = false">取消</Button>
          <Button
            variant="destructive"
            :disabled="isDeleting"
            @click="handleConfirmDelete"
          >
            {{ isDeleting ? '删除中...' : '确认删除' }}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </div>
</template>

<style scoped>
.task-card {
  cursor: pointer;
}

.task-card:hover {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}
</style>
