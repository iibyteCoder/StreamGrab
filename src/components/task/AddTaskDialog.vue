<script setup lang="ts">
/**
 * AddTaskDialog - 添加任务弹窗
 * 通过弹窗方式添加下载任务
 */

import { ref, computed } from 'vue';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { AppIcon } from '@/components/common';
import { useToast, useSettings, useDownloader, useTasks } from '@/composables';
import { StreamSelector } from '@/components/stream';
import type { StreamInfo, StreamSelection } from '@/types';

interface Props {
  open: boolean;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: 'update:open', value: boolean): void;
}>();

const toast = useToast();
const { settings } = useSettings();
const { addTask, updateTaskConfig, getTask } = useTasks();
const { startDownload, startPendingTasks, parseUrl, isParsing } = useDownloader();

// 状态
const urlInput = ref('');
const isSubmitting = ref(false);
const showStreamSelector = ref(false);
const currentStreamInfo = ref<StreamInfo | null>(null);
const pendingUrl = ref<string | null>(null);
const pendingStartAt = ref<Date | null>(null);

// 拖拽状态
const isDragging = ref(false);

const isOpen = computed({
  get: () => props.open,
  set: (value) => emit('update:open', value),
});

// 重置状态
const reset = () => {
  urlInput.value = '';
  isSubmitting.value = false;
  showStreamSelector.value = false;
  currentStreamInfo.value = null;
  pendingUrl.value = null;
  pendingStartAt.value = null;
};

// URL 解析
const parseUrls = (text: string): string[] =>
  text
    .split('\n')
    .map(line => line.trim())
    .filter(line => line.length > 0 && (line.startsWith('http://') || line.startsWith('https://')));

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
  const text = event.dataTransfer?.getData('text/plain');
  if (text) urls.push(...parseUrls(text));

  if (urls.length > 0) {
    const currentUrls = urlInput.value.trim();
    urlInput.value = currentUrls ? `${currentUrls}\n${urls.join('\n')}` : urls.join('\n');
  }
};

// 添加任务并下载
const addTaskAndDownload = (url: string, selection?: StreamSelection, startAt?: Date | null) => {
  const task = addTask(url, undefined, settings.value.general.saveDir);

  const config: Record<string, unknown> = { ...task.config };

  if (selection) {
    config.autoSelect = false;
    config.selectVideo = selection.videoIds[0] || '';
    config.selectAudio = selection.audioIds.join(',') || '';
    config.selectSubtitle = selection.subtitleIds.join(',') || '';
  }

  if (startAt) {
    config.startAt = startAt;
  }

  if (Object.keys(config).length > 0) {
    updateTaskConfig(task.id, config);
  }

  if (settings.value.general.autoStartDownload && !startAt) {
    startDownload(getTask(task.id)!);
  }

  return task;
};

// 下载处理
const handleDownload = async () => {
  const urls = parseUrls(urlInput.value);
  if (urls.length === 0) {
    toast.warning('请输入有效的下载链接');
    return;
  }

  if (isSubmitting.value) return;
  isSubmitting.value = true;

  try {
    if (urls.length === 1 && !settings.value.download.autoSelect) {
      const url = urls[0]!;
      pendingUrl.value = url;
      const info = await parseUrl(url);

      if (info) {
        currentStreamInfo.value = info;
        showStreamSelector.value = true;
      } else {
        addTaskAndDownload(url);
        handleClose();
        toast.success('已添加任务');
      }
    } else {
      let successCount = 0;
      for (const url of urls) {
        try {
          addTaskAndDownload(url);
          successCount++;
        } catch {
          // ignore
        }
      }

      if (settings.value.general.autoStartDownload) {
        await startPendingTasks();
      }

      if (successCount > 0) {
        toast.success(`已添加 ${successCount} 个任务`);
      }
      handleClose();
    }
  } catch (error) {
    toast.error(`添加任务失败: ${error instanceof Error ? error.message : '未知错误'}`);
  } finally {
    isSubmitting.value = false;
  }
};

// 流选择器
const handleStreamConfirm = (selection: StreamSelection) => {
  if (pendingUrl.value) {
    addTaskAndDownload(pendingUrl.value, selection, pendingStartAt.value);
    toast.success('已添加任务');
    handleClose();
  }
};

const handleStreamCancel = () => {
  pendingUrl.value = null;
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
          v-model="urlInput"
          placeholder="输入或粘贴下载链接（支持多个链接，每行一个）&#10;&#10;支持格式：&#10;• M3U8 / M3U&#10;• DASH / MPD&#10;• MSS / ISM"
          class="w-full h-40 px-3 py-2 text-sm bg-muted/50 border rounded-lg resize-none focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary transition-colors"
        />
      </div>

      <!-- 快捷提示 -->
      <div class="flex items-center justify-between text-xs text-muted-foreground">
        <span>支持拖放文本或 TXT 文件</span>
        <span>Ctrl + V 粘贴剪贴板链接</span>
      </div>

      <!-- 操作按钮 -->
      <div class="flex justify-end gap-2">
        <Button variant="outline" @click="handleClose">取消</Button>
        <Button :disabled="isSubmitting || !urlInput.trim()" @click="handleDownload">
          <AppIcon v-if="isSubmitting" name="Loader2" :size="16" class="mr-2 animate-spin" />
          <AppIcon v-else name="Download" :size="16" class="mr-2" />
          {{ isSubmitting ? '处理中...' : '添加任务' }}
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
    </DialogContent>
  </Dialog>
</template>
