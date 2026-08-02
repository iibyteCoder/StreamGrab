<script setup lang="ts">
/**
 * TaskDetailPanel - 任务详情侧边栏
 * 使用扁平化任务数据，实时检查文件状态
 */

import { computed, ref, watch, onMounted, onBeforeUnmount } from "vue";
import { useI18n } from "vue-i18n";
import { AppIcon } from "@/components/common";
import { Button } from "@/components/ui/button";
import { useTasks, useDownloader, useToast } from "@/composables";
import { useTaskStore } from "@/stores";
import { systemService, clipboardService } from "@/services";
import {
  TaskStatusHeader,
  TaskMediaInfo,
  TaskBasicInfo,
  TaskActionButtons,
  ProgressChart,
} from "@/components/task";

interface Props {
  open: boolean;
  taskId: string | null;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: "update:open", value: boolean): void;
}>();

const { getTask } = useTasks();
const { startDownload, stopDownload, pauseDownload, resumeDownload } =
  useDownloader();
const taskStore = useTaskStore();
const { t } = useI18n();
const toast = useToast();

// 当前任务
const task = computed(() =>
  props.taskId ? (getTask(props.taskId) ?? null) : null,
);

// 文件存在状态（实时检查）
const fileExists = ref<boolean | null>(null);

const checkFileExists = async () => {
  if (
    !task.value ||
    task.value.status !== "completed" ||
    !task.value.outputPath
  ) {
    fileExists.value = null;
    return;
  }
  try {
    fileExists.value = await systemService.fileExists(task.value.outputPath);
  } catch {
    fileExists.value = false;
  }
};

onMounted(checkFileExists);
watch(() => task.value?.status, checkFileExists);

// 操作处理
const handleOpenFolder = async () => {
  if (task.value?.saveDir) {
    try {
      await systemService.openInExplorer(task.value.saveDir);
    } catch (e) {
      console.error("Failed to open folder:", e);
    }
  }
};

const handleOpenFile = async () => {
  if (task.value?.outputPath && fileExists.value) {
    try {
      await systemService.openFile(task.value.outputPath);
    } catch (e) {
      console.error("Failed to open file:", e);
    }
  }
};

const handleStart = async () => {
  if (task.value) await startDownload(task.value);
};

const handlePause = async () => {
  if (task.value) await pauseDownload(task.value.id);
};

const handleResume = async () => {
  if (task.value) await resumeDownload(task.value.id);
};

const handleStop = async () => {
  if (task.value) await stopDownload(task.value.id);
};

const handleRetry = async () => {
  if (task.value) {
    await taskStore.retryTask(task.value.id);
    const updated = taskStore.getTaskById(task.value.id);
    if (updated) await startDownload(updated);
  }
};

const handleClose = () => {
  emit("update:open", false);
};

// 复制下载链接（图标态 1.5s 自动还原）
const copied = ref(false);
let copiedTimer: ReturnType<typeof setTimeout> | null = null;

const handleCopyUrl = async () => {
  if (!task.value) return;
  try {
    await clipboardService.writeText(task.value.url);
    copied.value = true;
    toast.success(t("messages.urlCopied", "链接已复制"));
    if (copiedTimer) clearTimeout(copiedTimer);
    copiedTimer = setTimeout(() => {
      copied.value = false;
    }, 1500);
  } catch (e) {
    console.error("Failed to copy URL:", e);
    toast.error(t("settings.preset.copyFailed", "复制失败"));
  }
};

onBeforeUnmount(() => {
  if (copiedTimer) clearTimeout(copiedTimer);
});

// 切换任务时复位复制图标态，避免上一任务的 Check 残留到新任务
watch(
  () => props.taskId,
  () => {
    copied.value = false;
    if (copiedTimer) clearTimeout(copiedTimer);
  },
);
</script>

<template>
  <div class="detail-panel-wrapper h-full flex items-stretch">
    <div
      class="bg-card border shadow-lg flex flex-col overflow-hidden transition-all duration-300 self-stretch"
      :class="open && task ? 'w-80 my-4 mr-6 rounded-xl' : 'w-0'"
    >
      <Transition name="fade-content">
        <div v-if="open && task" class="h-full flex flex-col w-80">
          <!-- 头部 -->
          <TaskStatusHeader :task="task" @close="handleClose" />

          <!-- 内容 -->
          <div class="flex-1 overflow-y-auto p-4 space-y-4">
            <!-- 进度图表 -->
            <div class="space-y-2">
              <h4
                class="text-xs font-semibold text-muted-foreground uppercase tracking-wide"
              >
                下载进度
              </h4>
              <ProgressChart
                v-if="task.id"
                :task-id="task.id"
                :height="160"
                :show-speed="true"
              />
            </div>

            <!-- 媒体/文件信息 -->
            <TaskMediaInfo :task="task" />

            <!-- 基本信息 -->
            <TaskBasicInfo :task="task" @open-folder="handleOpenFolder" />

            <!-- URL -->
            <div class="space-y-2">
              <div class="flex items-center justify-between">
                <h4
                  class="text-xs font-semibold text-muted-foreground uppercase tracking-wide"
                >
                  下载链接
                </h4>
                <Button
                  variant="ghost"
                  size="icon"
                  class="h-7 w-7 cursor-pointer"
                  :title="t('common.copy', '复制')"
                  @click="handleCopyUrl"
                >
                  <AppIcon
                    :name="copied ? 'Check' : 'Copy'"
                    :size="14"
                    :class="copied ? 'text-green-500' : ''"
                  />
                </Button>
              </div>
              <div class="bg-muted/30 rounded-lg p-2.5">
                <p
                  class="text-xs break-all text-muted-foreground leading-relaxed"
                >
                  {{ task.url }}
                </p>
              </div>
            </div>

            <!-- 文件丢失警告 -->
            <div
              v-if="task.status === 'completed' && fileExists === false"
              class="flex items-center gap-2 p-3 bg-amber-500/10 border border-amber-500/20 rounded-lg text-amber-600 text-sm"
            >
              <AppIcon name="AlertTriangle" :size="16" />
              <span>文件已被移动或删除</span>
            </div>

            <!-- 错误信息 -->
            <div
              v-if="task.status === 'failed' && task.error"
              class="space-y-2"
            >
              <h4
                class="text-xs font-semibold text-destructive uppercase tracking-wide"
              >
                错误信息
              </h4>
              <div
                class="bg-destructive/10 border border-destructive/20 rounded-lg p-2.5"
              >
                <p class="text-xs break-all text-destructive leading-relaxed">
                  {{ task.error }}
                </p>
              </div>
            </div>
          </div>

          <!-- 底部操作 -->
          <TaskActionButtons
            :task="task"
            @start="handleStart"
            @pause="handlePause"
            @resume="handleResume"
            @stop="handleStop"
            @retry="handleRetry"
            @open-file="handleOpenFile"
          />
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
</style>
