<script setup lang="ts">
/**
 * TaskDetailPanel - 任务详情侧边栏
 * 在任务列表区域内从右侧滑出显示任务详细信息
 */

import { computed } from "vue";
import { Button } from "@/components/ui/button";
import { AppIcon } from "@/components/common";
import { useTasks, useDownloader } from "@/composables";
import { configService } from "@/services";
import {
  formatSpeed,
  formatFileSize,
  formatDuration,
  formatDate,
} from "@/utils/format";
import { TASK_STATUS_CONFIG } from "@/utils/constants";

interface Props {
  open: boolean;
  taskId: string | null;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: "update:open", value: boolean): void;
}>();

const { getTask } = useTasks();
const {
  startDownload,
  stopDownload,
  pauseDownload,
  resumeDownload,
  retryDownload,
} = useDownloader();

// 当前任务
const task = computed(() => (props.taskId ? getTask(props.taskId) : null));

// 状态配置
const statusConfig = computed(() =>
  task.value ? TASK_STATUS_CONFIG[task.value.status] : null,
);

// 状态图标
const statusIcon = computed(() => {
  if (!task.value) return "Clock";
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
  return icons[task.value.status] || "Clock";
});

// 格式化信息
const sizeInfo = computed(() => {
  if (!task.value) return "";
  const { downloadedSize, totalSize } = task.value.progress;
  if (totalSize > 0)
    return `${formatFileSize(downloadedSize)} / ${formatFileSize(totalSize)}`;
  if (downloadedSize > 0) return formatFileSize(downloadedSize);
  return "未知";
});

const speedInfo = computed(() =>
  task.value?.status === "downloading"
    ? formatSpeed(task.value.progress.speed)
    : "",
);

const etaInfo = computed(() =>
  task.value?.status === "downloading" && task.value.progress.eta
    ? formatDuration(task.value.progress.eta)
    : "",
);

const completedTime = computed(() =>
  task.value?.completedAt ? formatDate(task.value.completedAt) : "",
);

const createdAt = computed(() =>
  task.value?.createdAt ? formatDate(task.value.createdAt) : "",
);

// 操作处理
const handleStart = async () => {
  if (task.value) await startDownload(task.value);
};

const handlePause = async () => {
  if (task.value) await pauseDownload(task.value.id);
};

const handleResume = async () => {
  if (task.value) await resumeDownload(task.value);
};

const handleStop = async () => {
  if (task.value) await stopDownload(task.value.id);
};

const handleRetry = async () => {
  if (task.value) await retryDownload(task.value);
};

const handleOpenFolder = async () => {
  if (task.value?.saveDir) {
    try {
      await configService.openInExplorer(task.value.saveDir);
    } catch (e) {
      console.error("Failed to open folder:", e);
    }
  }
};

const handleOpenFile = async () => {
  if (task.value?.outputPath) {
    try {
      await configService.openInExplorer(task.value.outputPath);
    } catch (e) {
      console.error("Failed to open file:", e);
    }
  }
};

const handleClose = () => {
  emit("update:open", false);
};
</script>

<template>
  <div class="detail-panel-wrapper h-full flex items-stretch">
    <!-- 侧边栏卡片（使用宽度动画） -->
    <div
      class="bg-card border shadow-lg flex flex-col overflow-hidden transition-all duration-300 self-stretch"
      :class="open && task ? 'w-80 my-4 mr-4 rounded-xl' : 'w-0'"
    >
      <Transition name="fade-content">
        <div v-if="open && task" class="h-full flex flex-col w-80">
          <!-- 头部 -->
          <div
            class="flex items-center justify-between px-4 h-14 border-b shrink-0"
          >
            <h3 class="font-semibold">任务详情</h3>
            <Button
              variant="ghost"
              size="icon"
              class="h-8 w-8"
              @click="handleClose"
            >
              <AppIcon name="X" :size="18" />
            </Button>
          </div>

          <!-- 内容 -->
          <div class="flex-1 overflow-y-auto p-4 space-y-6">
            <!-- 状态 -->
            <div class="flex items-center gap-3">
              <div
                class="flex h-12 w-12 items-center justify-center rounded-xl"
                :style="{
                  backgroundColor: `${statusConfig?.color ?? '#888'}15`,
                }"
              >
                <AppIcon
                  :name="statusIcon as any"
                  :size="24"
                  :style="{ color: statusConfig?.color ?? '#888' }"
                />
              </div>
              <div class="min-w-0 flex-1">
                <p class="font-medium break-all">
                  {{ task.fileName || "未命名文件" }}
                </p>
                <p
                  class="text-sm"
                  :style="{ color: statusConfig?.color ?? '#888' }"
                >
                  {{ statusConfig?.text ?? "未知" }}
                </p>
              </div>
            </div>

            <!-- 进度信息 -->
            <div v-if="task.status !== 'completed'" class="space-y-3">
              <div class="flex justify-between text-sm">
                <span class="text-muted-foreground">进度</span>
                <span class="font-medium">{{ task.progress.percent }}%</span>
              </div>
              <div class="h-2 bg-muted rounded-full overflow-hidden">
                <div
                  class="h-full bg-primary transition-all duration-300"
                  :style="{ width: `${task.progress.percent}%` }"
                />
              </div>
              <div class="grid grid-cols-2 gap-2 text-sm">
                <div class="bg-muted/50 rounded-lg p-2">
                  <p class="text-muted-foreground text-xs">文件大小</p>
                  <p class="font-medium">{{ sizeInfo }}</p>
                </div>
                <div class="bg-muted/50 rounded-lg p-2">
                  <p class="text-muted-foreground text-xs">下载速度</p>
                  <p class="font-medium text-primary">{{ speedInfo || "-" }}</p>
                </div>
                <div class="bg-muted/50 rounded-lg p-2">
                  <p class="text-muted-foreground text-xs">剩余时间</p>
                  <p class="font-medium">{{ etaInfo || "-" }}</p>
                </div>
                <div class="bg-muted/50 rounded-lg p-2">
                  <p class="text-muted-foreground text-xs">分片进度</p>
                  <p class="font-medium">
                    {{ task.progress.totalSegments || 0 }} 个
                  </p>
                </div>
              </div>
            </div>

            <!-- 已完成信息 -->
            <div v-else class="space-y-3">
              <div class="grid grid-cols-2 gap-2 text-sm">
                <div class="bg-muted/50 rounded-lg p-2">
                  <p class="text-muted-foreground text-xs">文件大小</p>
                  <p class="font-medium">
                    {{ formatFileSize(task.progress.totalSize || 0) }}
                  </p>
                </div>
                <div class="bg-muted/50 rounded-lg p-2">
                  <p class="text-muted-foreground text-xs">完成时间</p>
                  <p class="font-medium">{{ completedTime }}</p>
                </div>
              </div>
            </div>

            <!-- 基本信息 -->
            <div class="space-y-3">
              <h4 class="text-sm font-medium text-muted-foreground">
                基本信息
              </h4>
              <div class="space-y-2 text-sm">
                <div class="flex justify-between">
                  <span class="text-muted-foreground">创建时间</span>
                  <span>{{ createdAt }}</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-muted-foreground">保存位置</span>
                  <Button
                    variant="link"
                    size="sm"
                    class="h-auto p-0 text-xs"
                    @click="handleOpenFolder"
                  >
                    打开目录
                  </Button>
                </div>
              </div>
            </div>

            <!-- URL -->
            <div class="space-y-2">
              <h4 class="text-sm font-medium text-muted-foreground">
                下载链接
              </h4>
              <div class="bg-muted/50 rounded-lg p-2">
                <p class="text-xs break-all text-muted-foreground">
                  {{ task.url }}
                </p>
              </div>
            </div>

            <!-- 错误信息 -->
            <div
              v-if="task.status === 'failed' && task.error"
              class="space-y-2"
            >
              <h4 class="text-sm font-medium text-destructive">错误信息</h4>
              <div class="bg-destructive/10 rounded-lg p-2">
                <p class="text-xs break-all text-destructive">
                  {{ task.error }}
                </p>
              </div>
            </div>
          </div>

          <!-- 底部操作 -->
          <div class="border-t p-4 shrink-0 space-y-2">
            <!-- 已完成：打开文件 -->
            <Button
              v-if="task.status === 'completed'"
              class="w-full"
              @click="handleOpenFile"
            >
              <AppIcon name="Play" :size="16" class="mr-2" />
              播放文件
            </Button>

            <!-- 下载中：暂停 -->
            <Button
              v-if="task.status === 'downloading'"
              class="w-full"
              @click="handlePause"
            >
              <AppIcon name="Pause" :size="16" class="mr-2" />
              暂停下载
            </Button>

            <!-- 暂停/等待：继续 -->
            <Button
              v-if="task.status === 'paused' || task.status === 'pending'"
              class="w-full"
              @click="task.status === 'paused' ? handleResume() : handleStart()"
            >
              <AppIcon name="Play" :size="16" class="mr-2" />
              {{ task.status === "paused" ? "继续下载" : "开始下载" }}
            </Button>

            <!-- 失败：重试 -->
            <Button
              v-if="task.status === 'failed'"
              class="w-full"
              @click="handleRetry"
            >
              <AppIcon name="RefreshCw" :size="16" class="mr-2" />
              重新下载
            </Button>

            <!-- 下载中/暂停：停止 -->
            <Button
              v-if="task.status === 'downloading' || task.status === 'paused'"
              variant="outline"
              class="w-full"
              @click="handleStop"
            >
              <AppIcon name="Square" :size="16" class="mr-2" />
              停止下载
            </Button>
          </div>
        </div>
      </Transition>
    </div>
  </div>
</template>

<style scoped>
.fade-content-enter-active,
.fade-content-leave-active {
  transition: opacity 0.2s ease;
}

.fade-content-enter-from,
.fade-content-leave-to {
  opacity: 0;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
