<script setup lang="ts">
/**
 * TaskCard - 任务卡片组件
 * 紧凑但信息丰富的任务展示
 *
 * 使用扁平化任务数据结构
 */

import { computed, ref, onMounted, watch } from "vue";
import { useI18n } from "vue-i18n";
import { AppIcon } from "@/components/common";
import { useTasks, useDownloader, useToast } from "@/composables";
import { useTaskStore } from "@/stores";
import { systemService, clipboardService } from "@/services";
import {
  formatSpeed,
  formatFileSize,
  formatDuration,
  formatDate,
} from "@/utils/format";
import { TASK_STATUS_CONFIG } from "@/utils/constants";
import {
  TaskStatusBadge,
  TaskQuickActions,
  TaskDeleteDialog,
  TaskContextMenu,
  LogViewer,
} from "@/components/task";
import { ContextMenu, ContextMenuTrigger } from "@/components/ui/context-menu";
import type { DownloadTask } from "@/domain";

interface Props {
  task: DownloadTask;
  /** 是否为当前选中/查看详情的任务 */
  active?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  active: false,
});

const emit = defineEmits<{
  (e: "click", task: DownloadTask): void;
  (e: "redownload", task: DownloadTask): void;
}>();

const { removeTask } = useTasks();
const { startDownload, stopDownload, pauseDownload, resumeDownload } =
  useDownloader();
const taskStore = useTaskStore();
const { t } = useI18n();
const toast = useToast();

// 文件存在状态（实时检查）
const fileExists = ref<boolean | null>(null);
const isCheckingFile = ref(false);

const checkFileExists = async () => {
  if (props.task.status !== "completed" || !props.task.outputPath) {
    fileExists.value = null;
    return;
  }

  isCheckingFile.value = true;
  try {
    fileExists.value = await systemService.fileExists(props.task.outputPath);
  } catch {
    fileExists.value = false;
  } finally {
    isCheckingFile.value = false;
  }
};

// 组件挂载和任务状态变化时检查文件
onMounted(checkFileExists);
watch(
  () => props.task.status,
  () => {
    if (props.task.status === "completed") {
      checkFileExists();
    }
  },
);

// 对话框状态
const showLogViewer = ref(false);
const showDeleteDialog = ref(false);
const isDeleting = ref(false);

// 计算属性
const hasLogs = computed(() => taskStore.getTaskLogs(props.task.id).length > 0);

const statusConfig = computed(() => {
  const config = TASK_STATUS_CONFIG[props.task.status];
  return config ?? TASK_STATUS_CONFIG.pending;
});

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

// 是否应该显示进度区域（下载中、暂停、失败、合并中）
const shouldShowProgress = computed(() => {
  return ["downloading", "paused", "failed", "merging", "muxing"].includes(
    props.task.status,
  );
});

// 进度条颜色
const progressColor = computed(() => {
  const colors: Record<string, string> = {
    completed: "#22c55e",
    failed: "#ef4444",
    paused: "#f59e0b",
    merging: "#8b5cf6",
    muxing: "#8b5cf6",
    downloading: "#3b82f6",
  };
  return colors[props.task.status] || "#3b82f6";
});

// 进度条样式（带动画效果）
const progressStyle = computed(() => {
  const percent = Math.min(
    100,
    Math.max(0, props.task.progress.overallPercent),
  );
  return {
    width: `${percent}%`,
    backgroundColor: progressColor.value,
  };
});

// 是否显示进度条动画（下载中时显示条纹动画）
const showProgressAnimation = computed(() => {
  return props.task.status === "downloading";
});

// 下载速度
const speedText = computed(() =>
  props.task.status === "downloading"
    ? formatSpeed(props.task.progress.speed)
    : "",
);

// 剩余时间
const etaText = computed(() =>
  props.task.status === "downloading" && props.task.progress.eta
    ? formatDuration(props.task.progress.eta)
    : "",
);

// 文件丢失提示
const showFileMissingHint = computed(
  () =>
    props.task.status === "completed" &&
    !isCheckingFile.value &&
    fileExists.value === false,
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
const handleResume = async () => await resumeDownload(props.task.id);
const handleStop = async () => await stopDownload(props.task.id);
const handleRetry = async () => {
  await taskStore.retryTask(props.task.id);
  const updated = taskStore.getTaskById(props.task.id);
  if (updated) await startDownload(updated);
};

const handleOpenFolder = async () => {
  if (props.task.saveDir) {
    try {
      await systemService.openInExplorer(props.task.saveDir);
    } catch (e) {
      console.error("Failed to open folder:", e);
    }
  }
};

const handleOpenFile = async () => {
  if (props.task.outputPath && fileExists.value) {
    try {
      await systemService.openFile(props.task.outputPath);
    } catch (e) {
      console.error("Failed to open file:", e);
    }
  }
};

const handleRemoveClick = () => {
  if (props.task.status === "completed" && fileExists.value) {
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
        await systemService.deleteFileOrFolder(props.task.outputPath);
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

const handleClick = () => emit("click", props.task);

// 右键菜单操作
const handleRedownload = () => emit("redownload", props.task);

const handleCopyUrl = async () => {
  try {
    await clipboardService.writeText(props.task.url);
    toast.success(t("messages.copiedUrl", "已复制下载链接"));
  } catch (e) {
    console.error("Failed to copy URL:", e);
  }
};

const handleCopyFileName = async () => {
  try {
    await clipboardService.writeText(props.task.fileName);
    toast.success(t("messages.copiedFileName", "已复制文件名"));
  } catch (e) {
    console.error("Failed to copy file name:", e);
  }
};

const handleCopyFilePath = async () => {
  const path = props.task.outputPath;
  if (!path) return;
  try {
    await clipboardService.writeText(path);
    toast.success(t("messages.copiedFilePath", "已复制文件路径"));
  } catch (e) {
    console.error("Failed to copy file path:", e);
  }
};
</script>

<template>
  <ContextMenu>
    <ContextMenuTrigger as-child>
      <div
        class="task-card group relative rounded-lg border bg-card p-3 transition-all duration-200"
        :class="[
          task.status === 'cancelled' ? 'opacity-60' : '',
          active
            ? 'ring-2 ring-primary/50 border-primary/30 bg-primary/5 shadow-md'
            : 'hover:shadow-md',
        ]"
        @click="handleClick"
      >
        <!-- 状态标签 -->
        <TaskStatusBadge
          :text="statusConfig?.text ?? ''"
          :color="statusConfig?.color ?? '#888'"
        />

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
            <!-- 第一行：文件名 -->
            <h4 class="text-sm font-medium truncate">
              {{ task.fileName || "未命名文件" }}
            </h4>

            <!-- 进度可视化区域 -->
            <div v-if="shouldShowProgress" class="mt-2">
              <!-- 进度条 + 百分比 -->
              <div class="flex items-center gap-2">
                <div class="progress-track flex-1">
                  <div
                    class="progress-bar"
                    :class="{ 'progress-animated': showProgressAnimation }"
                    :style="progressStyle"
                  />
                </div>
                <span
                  class="text-xs font-medium tabular-nums"
                  :style="{
                    color: progressColor,
                    minWidth: '36px',
                    textAlign: 'right',
                  }"
                >
                  {{ task.progress.overallPercent }}%
                </span>
              </div>

              <!-- 进度详情行 -->
              <div
                class="flex items-center gap-3 mt-1 text-xs text-muted-foreground"
              >
                <!-- 下载中：速度 + 已下载/总大小 + 剩余时间 -->
                <template v-if="task.status === 'downloading'">
                  <span v-if="speedText" class="text-primary font-medium">{{
                    speedText
                  }}</span>
                  <span
                    v-if="
                      task.progress.downloadedSize && task.progress.totalSize
                    "
                  >
                    {{ formatFileSize(task.progress.downloadedSize) }} /
                    {{ formatFileSize(task.progress.totalSize) }}
                  </span>
                  <span v-else-if="task.progress.downloadedSize">
                    {{ formatFileSize(task.progress.downloadedSize) }}
                  </span>
                  <span v-if="etaText">剩余 {{ etaText }}</span>
                </template>

                <!-- 暂停中 -->
                <template v-else-if="task.status === 'paused'">
                  <span v-if="task.progress.downloadedSize">{{
                    formatFileSize(task.progress.downloadedSize)
                  }}</span>
                  <span class="text-amber-500">已暂停</span>
                </template>

                <!-- 失败 -->
                <template v-else-if="task.status === 'failed'">
                  <span v-if="task.progress.downloadedSize">{{
                    formatFileSize(task.progress.downloadedSize)
                  }}</span>
                  <span class="text-destructive">下载失败</span>
                </template>

                <!-- 合并中 -->
                <template
                  v-else-if="
                    task.status === 'merging' || task.status === 'muxing'
                  "
                >
                  <span class="text-purple-400">正在合并...</span>
                </template>
              </div>
            </div>

            <!-- 非进度状态：显示简单信息 -->
            <div
              v-else
              class="flex items-center gap-3 mt-1.5 h-4 text-xs text-muted-foreground"
            >
              <template v-if="task.status === 'analyzing'">
                <span class="text-primary">正在解析...</span>
              </template>
              <template v-else-if="task.status === 'pending'">
                <span>等待中</span>
              </template>
              <template v-else-if="task.status === 'cancelled'">
                <span>已取消</span>
              </template>
              <template v-else-if="task.status === 'completed'">
                <span v-if="task.progress.totalSize">{{
                  formatFileSize(task.progress.totalSize)
                }}</span>
                <span v-else-if="task.progress.downloadedSize">{{
                  formatFileSize(task.progress.downloadedSize)
                }}</span>
                <span>{{ completedTimeText }}</span>
                <span
                  v-if="showFileMissingHint"
                  class="text-amber-500 flex items-center gap-0.5"
                >
                  <AppIcon name="AlertTriangle" :size="12" />
                  文件已移除
                </span>
              </template>
            </div>

            <!-- 错误信息 -->
            <div
              v-if="task.status === 'failed' && task.error"
              class="mt-2 p-2 bg-destructive/10 rounded text-xs text-destructive break-all"
            >
              {{ task.error }}
            </div>
          </div>

          <!-- 快速操作按钮 -->
          <TaskQuickActions
            :task="task"
            :file-exists="fileExists ?? false"
            :has-logs="hasLogs"
            @open-folder="handleOpenFolder"
            @open-file="handleOpenFile"
            @show-logs="showLogViewer = true"
            @pause="handlePause"
            @start="handleStart"
            @resume="handleResume"
            @retry="handleRetry"
            @stop="handleStop"
            @delete="handleRemoveClick"
          />
        </div>

        <!-- 日志查看器 -->
        <LogViewer v-model:open="showLogViewer" :task-id="task.id" />

        <!-- 删除确认对话框 -->
        <TaskDeleteDialog
          v-model:open="showDeleteDialog"
          :task="task"
          :file-exists="fileExists ?? false"
          :is-deleting="isDeleting"
          @confirm="performDelete"
        />
      </div>
    </ContextMenuTrigger>
    <TaskContextMenu
      :task="task"
      :file-exists="fileExists ?? false"
      @redownload="handleRedownload"
      @copy-url="handleCopyUrl"
      @copy-file-name="handleCopyFileName"
      @copy-file-path="handleCopyFilePath"
      @open-detail="handleClick"
    />
  </ContextMenu>
</template>

<style scoped>
.task-card {
  cursor: pointer;
}

/* 进度条轨道 */
.progress-track {
  height: 6px;
  background: rgba(255, 255, 255, 0.08);
  border-radius: 3px;
  overflow: hidden;
}

/* 进度条 */
.progress-bar {
  height: 100%;
  border-radius: 3px;
  transition: width 0.3s ease-out;
  min-width: 0;
}

/* 进度条动画（条纹移动效果） */
.progress-animated {
  background-image: linear-gradient(
    45deg,
    rgba(255, 255, 255, 0.15) 25%,
    transparent 25%,
    transparent 50%,
    rgba(255, 255, 255, 0.15) 50%,
    rgba(255, 255, 255, 0.15) 75%,
    transparent 75%,
    transparent
  );
  background-size: 10px 10px;
  animation: progress-stripes 0.8s linear infinite;
}

@keyframes progress-stripes {
  from {
    background-position: 10px 0;
  }
  to {
    background-position: 0 0;
  }
}
</style>
