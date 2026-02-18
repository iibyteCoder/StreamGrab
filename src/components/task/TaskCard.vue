<script setup lang="ts">
/**
 * TaskCard - 任务卡片组件
 * 紧凑但信息丰富的任务展示
 */

import { computed, ref, onMounted, watch } from "vue";
import { AppProgress } from "@/components/common";
import { useTasks, useDownloader } from "@/composables";
import { useTaskStore } from "@/stores";
import { configService } from "@/services";
import {
  formatSpeed,
  formatFileSize,
  formatDuration,
  formatDate,
} from "@/utils/format";
import { TASK_STATUS_CONFIG } from "@/utils/constants";
import { LogViewer } from "@/components/task";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { AppIcon } from "@/components/common";
import type { DownloadTask } from "@/types";

interface Props {
  task: DownloadTask;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: "click", task: DownloadTask): void;
}>();

const { removeTask } = useTasks();
const {
  startDownload,
  stopDownload,
  pauseDownload,
  resumeDownload,
  retryDownload,
} = useDownloader();
const taskStore = useTaskStore();

// 对话框状态
const showLogViewer = ref(false);
const showDeleteDialog = ref(false);
const deleteWithFile = ref(false);
const isDeleting = ref(false);

// 文件存在状态
const fileExists = ref<boolean | null>(null);

const checkFileExists = async () => {
  if (props.task.outputPath) {
    try {
      fileExists.value = await configService.fileExists(props.task.outputPath);
    } catch {
      fileExists.value = false;
    }
  }
};

onMounted(() => {
  if (props.task.status === "completed") checkFileExists();
});

watch(
  () => props.task.status,
  (newStatus) => {
    if (newStatus === "completed") checkFileExists();
  },
);

// 计算属性
const hasLogs = computed(() => taskStore.getTaskLogs(props.task.id).length > 0);

const statusConfig = computed(
  () => TASK_STATUS_CONFIG[props.task.status] || TASK_STATUS_CONFIG.pending,
);

const statusIcon = computed(() => {
  const icons: Record<string, string> = {
    pending: "Clock",
    analyzing: "Search",
    downloading: "Download",
    paused: "Pause",
    merging: "Combine",
    muxing: "Combine",
    completed: "CheckCircle",
    failed: "XCircle",
    cancelled: "X",
  };
  return icons[props.task.status] || "Clock";
});

type ProgressVariant = "default" | "success" | "warning" | "error";

const progressVariant = computed((): ProgressVariant => {
  const map: Record<string, ProgressVariant> = {
    completed: "success",
    failed: "error",
    downloading: "default",
  };
  return map[props.task.status] || "default";
});

// 下载速度
const speedText = computed(() =>
  props.task.status === "downloading"
    ? formatSpeed(props.task.progress.speed)
    : "",
);

// 文件大小
const sizeText = computed(() => {
  const { downloadedSize, totalSize } = props.task.progress;
  if (totalSize > 0)
    return `${formatFileSize(downloadedSize)} / ${formatFileSize(totalSize)}`;
  if (downloadedSize > 0) return formatFileSize(downloadedSize);
  return "";
});

// 剩余时间
const etaText = computed(() =>
  props.task.status === "downloading" && props.task.progress.eta
    ? formatDuration(props.task.progress.eta)
    : "",
);

// 分片进度
const segmentsText = computed(() => {
  const { downloadedSegments = 0, totalSegments = 0 } = props.task.progress;
  return totalSegments > 0 ? `${downloadedSegments}/${totalSegments} 分片` : "";
});

// 文件丢失提示
const showFileMissingHint = computed(
  () => props.task.status === "completed" && fileExists.value === false,
);

// 完成时间
const completedTimeText = computed(() => {
  if (props.task.status === "completed" && props.task.completedAt) {
    return formatDate(props.task.completedAt);
  }
  return "";
});

// 操作处理
const handleStart = async () => await startDownload(props.task);
const handlePause = async () => await pauseDownload(props.task.id);
const handleResume = async () => await resumeDownload(props.task);
const handleStop = async () => await stopDownload(props.task.id);
const handleRetry = async () => await retryDownload(props.task);

const handleRemoveClick = () => {
  if (props.task.status === "completed" && fileExists.value) {
    deleteWithFile.value = false;
    showDeleteDialog.value = true;
  } else {
    performDelete(false);
  }
};

const performDelete = async (withFile: boolean) => {
  isDeleting.value = true;
  try {
    if (withFile && props.task.outputPath) {
      try {
        await configService.deleteFileOrFolder(props.task.outputPath);
      } catch (e) {
        console.error("Failed to delete file:", e);
      }
    }
    removeTask(props.task.id);
  } finally {
    isDeleting.value = false;
    showDeleteDialog.value = false;
  }
};

const handleConfirmDelete = () => performDelete(deleteWithFile.value);

const handleOpenFolder = async () => {
  if (props.task.saveDir) {
    try {
      await configService.openInExplorer(props.task.saveDir);
    } catch (e) {
      console.error("Failed to open folder:", e);
    }
  }
};

const handleOpenFile = async () => {
  if (props.task.outputPath && fileExists.value) {
    try {
      await configService.openInExplorer(props.task.outputPath);
    } catch (e) {
      console.error("Failed to open file:", e);
    }
  }
};

const handleClick = () => emit("click", props.task);
</script>

<template>
  <div
    class="task-card group rounded-lg border bg-card p-3 transition-all duration-200 hover:shadow-md"
    :class="{ 'opacity-60': task.status === 'cancelled' }"
    @click="handleClick"
  >
    <!-- 主内容 -->
    <div class="flex items-start gap-3">
      <!-- 状态指示器 -->
      <div
        class="flex h-8 w-8 shrink-0 items-center justify-center rounded-full mt-0.5"
        :style="{ backgroundColor: `${statusConfig?.color ?? '#888'}20` }"
      >
        <AppIcon
          :name="statusIcon as any"
          :size="16"
          :style="{ color: statusConfig?.color ?? '#888' }"
        />
      </div>

      <!-- 信息区 -->
      <div class="flex-1 min-w-0">
        <!-- 第一行：文件名 + 状态标签 -->
        <div class="flex items-center gap-2">
          <h4 class="text-sm font-medium truncate flex-1">
            {{ task.fileName || "未命名文件" }}
          </h4>
          <span
            class="shrink-0 px-1.5 py-0.5 rounded text-xs font-medium"
            :style="{
              backgroundColor: `${statusConfig?.color ?? '#888'}20`,
              color: statusConfig?.color ?? '#888',
            }"
          >
            {{ statusConfig?.text ?? "未知" }}
          </span>
        </div>

        <!-- 下载中状态：进度条 + 详细信息 -->
        <template v-if="task.status === 'downloading'">
          <div class="mt-2">
            <AppProgress
              :percent="task.progress.percent"
              :variant="progressVariant"
              size="sm"
            />
          </div>
          <div
            class="flex items-center gap-3 mt-1.5 text-xs text-muted-foreground"
          >
            <span v-if="sizeText">{{ sizeText }}</span>
            <span v-if="speedText" class="text-primary font-medium">{{
              speedText
            }}</span>
            <span v-if="etaText">剩余 {{ etaText }}</span>
            <span v-if="segmentsText" class="opacity-60">{{
              segmentsText
            }}</span>
          </div>
        </template>

        <!-- 已完成状态：文件信息 -->
        <template v-else-if="task.status === 'completed'">
          <div
            class="flex items-center gap-3 mt-1 text-xs text-muted-foreground"
          >
            <span v-if="task.progress.totalSize">{{
              formatFileSize(task.progress.totalSize)
            }}</span>
            <span>{{ completedTimeText }}</span>
            <span
              v-if="showFileMissingHint"
              class="text-amber-500 flex items-center gap-0.5"
            >
              <AppIcon name="AlertTriangle" :size="12" />
              文件已移除
            </span>
          </div>
        </template>

        <!-- 其他状态 -->
        <template v-else>
          <div
            class="flex items-center gap-3 mt-1 text-xs text-muted-foreground"
          >
            <span v-if="sizeText">{{ sizeText }}</span>
            <span v-if="task.progress.percent > 0"
              >{{ task.progress.percent }}%</span
            >
            <span v-if="segmentsText" class="opacity-60">{{
              segmentsText
            }}</span>
          </div>
        </template>

        <!-- 错误信息 -->
        <div
          v-if="task.status === 'failed' && task.error"
          class="mt-2 p-2 bg-destructive/10 rounded text-xs text-destructive break-all"
        >
          {{ task.error }}
        </div>
      </div>

      <!-- 操作按钮组 -->
      <div
        class="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity shrink-0"
      >
        <!-- 打开文件夹 -->
        <button
          v-if="task.saveDir"
          class="action-btn"
          title="打开目录"
          @click.stop="handleOpenFolder"
        >
          <AppIcon name="FolderOpen" :size="15" />
        </button>

        <!-- 播放文件 -->
        <button
          v-if="task.status === 'completed' && fileExists"
          class="action-btn text-green-500 hover:text-green-600"
          title="播放"
          @click.stop="handleOpenFile"
        >
          <AppIcon name="Play" :size="15" />
        </button>

        <!-- 查看日志 -->
        <button
          v-if="hasLogs || task.status === 'downloading'"
          class="action-btn"
          title="日志"
          @click.stop="showLogViewer = true"
        >
          <AppIcon name="FileText" :size="15" />
        </button>

        <!-- 暂停 -->
        <button
          v-if="task.status === 'downloading'"
          class="action-btn"
          title="暂停"
          @click.stop="handlePause"
        >
          <AppIcon name="Pause" :size="15" />
        </button>

        <!-- 开始/继续 -->
        <button
          v-if="task.status === 'paused' || task.status === 'pending'"
          class="action-btn text-primary"
          title="开始"
          @click.stop="
            task.status === 'paused' ? handleResume() : handleStart()
          "
        >
          <AppIcon name="Play" :size="15" />
        </button>

        <!-- 重试 -->
        <button
          v-if="task.status === 'failed'"
          class="action-btn text-primary"
          title="重试"
          @click.stop="handleRetry"
        >
          <AppIcon name="RefreshCw" :size="15" />
        </button>

        <!-- 停止 -->
        <button
          v-if="task.status === 'downloading' || task.status === 'paused'"
          class="action-btn text-destructive"
          title="停止"
          @click.stop="handleStop"
        >
          <AppIcon name="Square" :size="15" />
        </button>

        <!-- 删除 -->
        <button
          v-if="['cancelled', 'failed', 'completed'].includes(task.status)"
          class="action-btn text-destructive"
          title="删除"
          @click.stop="handleRemoveClick"
        >
          <AppIcon name="Trash2" :size="15" />
        </button>
      </div>
    </div>

    <!-- 日志查看器 -->
    <LogViewer v-model:open="showLogViewer" :task-id="task.id" />

    <!-- 删除确认对话框 -->
    <Dialog v-model:open="showDeleteDialog">
      <DialogContent class="sm:max-w-[400px]">
        <DialogHeader>
          <DialogTitle>确认删除</DialogTitle>
          <DialogDescription>确定要删除此任务记录吗？</DialogDescription>
        </DialogHeader>

        <div class="py-4">
          <label class="flex items-center gap-2 cursor-pointer select-none">
            <input
              type="checkbox"
              v-model="deleteWithFile"
              class="w-4 h-4 rounded border accent-primary"
            />
            <span class="text-sm">同时删除下载的文件</span>
          </label>
          <p
            v-if="deleteWithFile && task.outputPath"
            class="mt-2 text-xs text-muted-foreground truncate"
          >
            {{ task.outputPath }}
          </p>
        </div>

        <DialogFooter>
          <Button variant="outline" @click="showDeleteDialog = false"
            >取消</Button
          >
          <Button
            variant="destructive"
            :disabled="isDeleting"
            @click="handleConfirmDelete"
          >
            {{ isDeleting ? "删除中..." : "确认删除" }}
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

.action-btn {
  @apply p-1.5 rounded-md hover:bg-accent text-muted-foreground hover:text-foreground transition-colors;
}
</style>
