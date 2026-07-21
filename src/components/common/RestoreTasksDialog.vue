<script setup lang="ts">
/**
 * RestoreTasksDialog - 启动时恢复中断任务弹窗
 * 检测到上次未完成的下载任务时显示，询问是否继续下载
 */

import { computed } from "vue";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { AppIcon } from "@/components/common";
import type { DownloadTask, TaskStatus } from "@/domain";

interface Props {
  open: boolean;
  tasks: DownloadTask[];
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: "update:open", value: boolean): void;
  (e: "confirm"): void;
  (e: "cancel"): void;
}>();

const isOpen = computed({
  get: () => props.open,
  set: (value) => emit("update:open", value),
});

const STATUS_LABELS: Partial<Record<TaskStatus, string>> = {
  paused: "已暂停",
  downloading: "下载中",
  analyzing: "解析中",
  merging: "合并中",
  muxing: "混流中",
};

const statusLabel = (status: TaskStatus): string =>
  STATUS_LABELS[status] ?? status;

const handleConfirm = () => {
  emit("confirm");
  isOpen.value = false;
};

const handleCancel = () => {
  emit("cancel");
  isOpen.value = false;
};
</script>

<template>
  <Dialog v-model:open="isOpen">
    <DialogContent class="sm:max-w-[480px]">
      <DialogHeader>
        <DialogTitle class="flex items-center gap-2">
          <AppIcon name="RotateCcw" :size="20" class="text-primary" />
          恢复中断的下载
        </DialogTitle>
        <DialogDescription class="pt-2">
          检测到 {{ tasks.length }} 个未完成的下载任务，是否继续下载？
        </DialogDescription>
      </DialogHeader>

      <!-- 任务列表 -->
      <div class="max-h-[280px] overflow-y-auto space-y-2 pr-1">
        <div
          v-for="task in tasks"
          :key="task.id"
          class="p-3 bg-muted/50 rounded-lg text-sm space-y-1"
        >
          <div class="flex items-center justify-between gap-2">
            <span class="font-medium truncate">{{ task.fileName }}</span>
            <span
              class="shrink-0 text-xs px-2 py-0.5 rounded-full bg-primary/10 text-primary"
            >
              {{ statusLabel(task.status) }}
            </span>
          </div>
          <div
            v-if="task.progress.percent > 0"
            class="flex items-center justify-between text-muted-foreground text-xs"
          >
            <span class="truncate">{{ task.url }}</span>
            <span class="shrink-0 ml-2">{{ task.progress.percent }}%</span>
          </div>
        </div>
      </div>

      <!-- 操作按钮 -->
      <div class="flex justify-end gap-2">
        <Button variant="outline" @click="handleCancel">稍后</Button>
        <Button @click="handleConfirm">
          <AppIcon name="Play" :size="16" class="mr-2" />
          全部恢复
        </Button>
      </div>
    </DialogContent>
  </Dialog>
</template>
