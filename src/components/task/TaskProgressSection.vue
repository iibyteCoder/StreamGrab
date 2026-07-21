<script setup lang="ts">
/**
 * TaskProgressSection - 任务进度信息区
 * 使用扁平化任务数据
 */

import { computed } from "vue";
import { TASK_STATUS_CONFIG } from "@/utils/constants";
import { formatSpeed, formatFileSize } from "@/utils/format";
import type { DownloadTask } from "@/domain";

interface Props {
  task: DownloadTask;
}

const props = defineProps<Props>();

const statusConfig = computed(() => {
  const config = TASK_STATUS_CONFIG[props.task.status];
  return config ?? TASK_STATUS_CONFIG.pending!;
});

const sizeInfo = computed(() => {
  const downloadedSize = props.task.progress.downloadedSize;
  const totalSize = props.task.progress.totalSize;
  if (totalSize > 0)
    return `${formatFileSize(downloadedSize)} / ${formatFileSize(totalSize)}`;
  if (downloadedSize > 0) return formatFileSize(downloadedSize);
  return "未知";
});

const speedInfo = computed(() =>
  props.task.status === "downloading"
    ? formatSpeed(props.task.progress.speed)
    : "",
);

const progressStyle = computed(() => ({
  width: `${props.task.progress.percent}%`,
  backgroundColor: statusConfig.value.color,
}));
</script>

<template>
  <div class="space-y-3">
    <div class="flex justify-between text-sm">
      <span class="text-muted-foreground">下载进度</span>
      <span class="font-semibold">{{ task.progress.percent }}%</span>
    </div>
    <div class="h-2 bg-muted rounded-full overflow-hidden">
      <div
        class="h-full rounded-full transition-all duration-300"
        :style="progressStyle"
      />
    </div>
    <div class="grid grid-cols-2 gap-2">
      <div class="bg-muted/40 rounded-lg p-2.5">
        <p class="text-muted-foreground text-xs mb-0.5">已下载</p>
        <p class="font-medium text-sm">{{ sizeInfo }}</p>
      </div>
      <div class="bg-muted/40 rounded-lg p-2.5">
        <p class="text-muted-foreground text-xs mb-0.5">下载速度</p>
        <p class="font-medium text-sm text-primary">
          {{ speedInfo || "-" }}
        </p>
      </div>
    </div>
  </div>
</template>
