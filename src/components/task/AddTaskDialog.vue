<script setup lang="ts">
/**
 * AddTaskDialog - 添加任务弹窗
 * 通过弹窗方式添加下载任务
 */

import { ref, computed, watch, nextTick } from "vue";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { AppIcon, UrlDuplicateDialog } from "@/components/common";
import { useToast, useSettings, useDownloader, useTasks } from "@/composables";
import { StreamSelector } from "@/components/stream";
import type { StreamInfo, StreamSelection, DownloadTask } from "@/types";

interface Props {
  open: boolean;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: "update:open", value: boolean): void;
}>();

const toast = useToast();
const { appSettings, m3u8dlSettings } = useSettings();
const { addTask, forceAddTask, updateTaskConfig, getTask } = useTasks();
const { startDownload, startPendingTasks, parseUrl, isParsing } =
  useDownloader();

// 状态
const urlInput = ref("");
const textareaRef = ref<HTMLTextAreaElement | null>(null);
const isSubmitting = ref(false);
const showStreamSelector = ref(false);
const currentStreamInfo = ref<StreamInfo | null>(null);
const pendingUrl = ref<string | null>(null);
const pendingStartAt = ref<Date | null>(null);
const pendingSelection = ref<StreamSelection | undefined>();

// URL 重复弹窗状态
const showUrlDuplicateDialog = ref(false);
const duplicateTask = ref<DownloadTask | null>(null);

// 拖拽状态
const isDragging = ref(false);

const isOpen = computed({
  get: () => props.open,
  set: (value) => emit("update:open", value),
});

// 弹窗打开时自动聚焦输入框
watch(isOpen, async (open) => {
  if (open) {
    await nextTick();
    textareaRef.value?.focus();
  }
});

// 重置状态
const reset = () => {
  urlInput.value = "";
  isSubmitting.value = false;
  showStreamSelector.value = false;
  currentStreamInfo.value = null;
  pendingUrl.value = null;
  pendingStartAt.value = null;
  pendingSelection.value = undefined;
  showUrlDuplicateDialog.value = false;
  duplicateTask.value = null;
};

// URL 解析
const parseUrls = (text: string): string[] =>
  text
    .split("\n")
    .map((line) => line.trim())
    .filter(
      (line) =>
        line.length > 0 &&
        (line.startsWith("http://") || line.startsWith("https://")),
    );

// 拖拽处理
const handleDragOver = (event: DragEvent) => {
  event.preventDefault();
  isDragging.value = true;
};

const handleDragLeave = () => {
  isDragging.value = false;
};

const handleDrop = async (event: DragEvent) => {
  event.preventDefault();
  isDragging.value = false;

  const urls: string[] = [];
  const text = event.dataTransfer?.getData("text/plain");
  if (text) urls.push(...parseUrls(text));

  if (urls.length > 0) {
    const currentUrls = urlInput.value.trim();
    urlInput.value = currentUrls
      ? `${currentUrls}\n${urls.join("\n")}`
      : urls.join("\n");
  }
};

// 创建任务并启动下载
const createTaskAndDownload = (
  task: DownloadTask,
  selection?: StreamSelection,
  startAt?: Date | null,
) => {
  const config: Record<string, unknown> = { ...task.config };

  if (selection) {
    config.autoSelect = false;
    config.selectVideo = selection.videoIds[0] || "";
    config.selectAudio = selection.audioIds.join(",") || "";
    config.selectSubtitle = selection.subtitleIds.join(",") || "";
  }

  if (startAt) {
    config.startAt = startAt;
  }

  if (Object.keys(config).length > 0) {
    updateTaskConfig(task.id, config);
  }

  if (appSettings.value.auto_start_download && !startAt) {
    startDownload(getTask(task.id)!);
  }

  return task;
};

// 添加任务（带 URL 冲突检测）
const addTaskAndDownload = (
  url: string,
  selection?: StreamSelection,
  startAt?: Date | null,
  skipUrlCheck = false,
): { task?: DownloadTask; duplicateUrl?: boolean } => {
  const saveDir = appSettings.value.default_save_dir;

  // 如果跳过 URL 检查，直接强制添加
  if (skipUrlCheck) {
    const { task } = forceAddTask(url, undefined, saveDir);
    if (task) {
      createTaskAndDownload(task, selection, startAt);
    }
    return { task };
  }

  // 正常添加（带冲突检测）
  const result = addTask(url, undefined, saveDir);

  if (result.duplicateUrl && result.existingTask) {
    return { duplicateUrl: true, task: result.existingTask };
  }

  if (result.task) {
    createTaskAndDownload(result.task, selection, startAt);
  }

  return { task: result.task };
};

// 处理 URL 重复确认
const handleUrlDuplicateConfirm = () => {
  if (pendingUrl.value) {
    // 用户确认仍然下载，跳过 URL 检查
    const { task } = addTaskAndDownload(
      pendingUrl.value,
      pendingSelection.value,
      pendingStartAt.value,
      true, // skipUrlCheck
    );
    if (task) {
      toast.success("已添加任务");
    }
    handleClose();
  }
};

const handleUrlDuplicateCancel = () => {
  // 用户取消，关闭弹窗但保持添加任务对话框打开
  showUrlDuplicateDialog.value = false;
  isSubmitting.value = false;
};

// 下载处理
const handleDownload = async () => {
  const urls = parseUrls(urlInput.value);
  if (urls.length === 0) {
    toast.warning("请输入有效的下载链接");
    return;
  }

  if (isSubmitting.value) return;
  isSubmitting.value = true;

  try {
    if (urls.length === 1 && !m3u8dlSettings.value.auto_select) {
      const url = urls[0]!;
      pendingUrl.value = url;
      const info = await parseUrl(url);

      if (info) {
        currentStreamInfo.value = info;
        showStreamSelector.value = true;
      } else {
        // 尝试添加任务
        const result = addTaskAndDownload(url);
        if (result.duplicateUrl && result.task) {
          // URL 重复，显示确认弹窗
          duplicateTask.value = result.task;
          showUrlDuplicateDialog.value = true;
        } else if (result.task) {
          handleClose();
          toast.success("已添加任务");
        }
      }
    } else {
      let successCount = 0;
      let duplicateCount = 0;

      for (const url of urls) {
        try {
          const result = addTaskAndDownload(url);
          if (result.task) {
            successCount++;
          } else if (result.duplicateUrl) {
            duplicateCount++;
          }
        } catch {
          // ignore
        }
      }

      if (appSettings.value.auto_start_download) {
        await startPendingTasks();
      }

      if (successCount > 0) {
        toast.success(`已添加 ${successCount} 个任务`);
      }
      if (duplicateCount > 0) {
        toast.warning(`${duplicateCount} 个链接已存在，已跳过`);
      }
      handleClose();
    }
  } catch (error) {
    toast.error(
      `添加任务失败: ${error instanceof Error ? error.message : "未知错误"}`,
    );
  } finally {
    if (!showUrlDuplicateDialog.value) {
      isSubmitting.value = false;
    }
  }
};

// 流选择器
const handleStreamConfirm = (selection: StreamSelection) => {
  if (pendingUrl.value) {
    pendingSelection.value = selection;
    // 尝试添加任务
    const result = addTaskAndDownload(
      pendingUrl.value,
      selection,
      pendingStartAt.value,
    );
    if (result.duplicateUrl && result.task) {
      // URL 重复，显示确认弹窗
      duplicateTask.value = result.task;
      showStreamSelector.value = false;
      showUrlDuplicateDialog.value = true;
    } else if (result.task) {
      toast.success("已添加任务");
      handleClose();
    }
  }
};

const handleStreamCancel = () => {
  pendingUrl.value = null;
  pendingSelection.value = undefined;
  currentStreamInfo.value = null;
  showStreamSelector.value = false;
};

const handleClose = () => {
  reset();
  isOpen.value = false;
};
</script>

<template>
  <Dialog v-model:open="isOpen">
    <DialogContent class="sm:max-w-[560px]" @close-auto-focus="reset">
      <DialogHeader>
        <DialogTitle class="flex items-center gap-2">
          <AppIcon name="Plus" :size="20" />
          添加下载任务
        </DialogTitle>
      </DialogHeader>

      <!-- 输入区域 -->
      <div
        class="relative"
        @dragover="handleDragOver"
        @dragleave="handleDragLeave"
        @drop="handleDrop"
      >
        <!-- 拖拽遮罩 -->
        <div
          v-if="isDragging"
          class="absolute inset-0 z-10 bg-primary/10 rounded-lg border-2 border-dashed border-primary flex items-center justify-center"
        >
          <span class="text-sm font-medium text-primary">释放以添加链接</span>
        </div>

        <textarea
          ref="textareaRef"
          v-model="urlInput"
          placeholder="输入或粘贴下载链接（支持多个链接，每行一个）&#10;&#10;支持格式：&#10;• M3U8 / M3U&#10;• DASH / MPD&#10;• MSS / ISM"
          class="w-full h-40 px-3 py-2 text-sm bg-muted/50 border rounded-lg resize-none focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary transition-colors"
        />
      </div>

      <!-- 快捷提示 -->
      <div
        class="flex items-center justify-between text-xs text-muted-foreground"
      >
        <span>支持拖放文本或 TXT 文件</span>
        <span>Ctrl + V 粘贴剪贴板链接</span>
      </div>

      <!-- 操作按钮 -->
      <div class="flex justify-end gap-2">
        <Button variant="outline" @click="handleClose">取消</Button>
        <Button
          :disabled="isSubmitting || !urlInput.trim()"
          @click="handleDownload"
        >
          <AppIcon
            v-if="isSubmitting"
            name="Loader2"
            :size="16"
            class="mr-2 animate-spin"
          />
          <AppIcon v-else name="Download" :size="16" class="mr-2" />
          {{ isSubmitting ? "处理中..." : "添加任务" }}
        </Button>
      </div>

      <!-- 流选择器 -->
      <StreamSelector
        v-model:open="showStreamSelector"
        :stream-info="currentStreamInfo"
        :loading="isParsing"
        @confirm="handleStreamConfirm"
        @cancel="handleStreamCancel"
      />

      <!-- URL 重复确认弹窗 -->
      <UrlDuplicateDialog
        v-model:open="showUrlDuplicateDialog"
        :existing-task="duplicateTask"
        @confirm="handleUrlDuplicateConfirm"
        @cancel="handleUrlDuplicateCancel"
      />
    </DialogContent>
  </Dialog>
</template>
