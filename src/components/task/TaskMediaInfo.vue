<script setup lang="ts">
/**
 * TaskMediaInfo - 媒体/文件信息展示
 * 使用扁平化任务数据
 */

import { computed } from "vue";
import { formatFileSize, formatDuration } from "@/utils/format";
import type { DownloadTask } from "@/types";

interface Props {
  task: DownloadTask;
}

const props = defineProps<Props>();

// 是否有媒体信息可显示
const hasMediaInfo = computed(() => {
  const t = props.task;
  return (
    t.mediaResolution ||
    t.mediaDuration ||
    t.mediaVideoCodec ||
    t.mediaFrameRate ||
    t.mediaSegmentCount ||
    t.mediaIsEncrypted ||
    t.progressTotalSize ||
    t.progressDownloadedSize ||
    t.mediaFileFormat
  );
});

// 文件大小显示文本
const fileSizeText = computed(() => {
  const t = props.task;
  if (t.progressTotalSize) {
    return formatFileSize(t.progressTotalSize);
  }
  if (t.progressDownloadedSize) {
    return `${formatFileSize(t.progressDownloadedSize)} (已下载)`;
  }
  return null;
});

const sectionTitle = computed(() =>
  props.task.status === "completed" ? "文件信息" : "媒体信息",
);
</script>

<template>
  <div v-if="hasMediaInfo" class="space-y-2">
    <h4
      class="text-xs font-semibold text-muted-foreground uppercase tracking-wide"
    >
      {{ sectionTitle }}
    </h4>
    <div class="grid grid-cols-2 gap-2">
      <!-- 分辨率 -->
      <div v-if="task.mediaResolution" class="bg-muted/40 rounded-lg p-2.5">
        <p class="text-muted-foreground text-xs mb-0.5">分辨率</p>
        <p class="font-medium text-sm">{{ task.mediaResolution }}</p>
      </div>
      <!-- 时长 -->
      <div v-if="task.mediaDuration" class="bg-muted/40 rounded-lg p-2.5">
        <p class="text-muted-foreground text-xs mb-0.5">时长</p>
        <p class="font-medium text-sm">
          {{ formatDuration(task.mediaDuration) }}
        </p>
      </div>
      <!-- 视频编码 -->
      <div v-if="task.mediaVideoCodec" class="bg-muted/40 rounded-lg p-2.5">
        <p class="text-muted-foreground text-xs mb-0.5">视频编码</p>
        <p class="font-medium text-sm uppercase">{{ task.mediaVideoCodec }}</p>
      </div>
      <!-- 帧率 -->
      <div v-if="task.mediaFrameRate" class="bg-muted/40 rounded-lg p-2.5">
        <p class="text-muted-foreground text-xs mb-0.5">帧率</p>
        <p class="font-medium text-sm">{{ task.mediaFrameRate }} fps</p>
      </div>
      <!-- 文件大小 -->
      <div v-if="fileSizeText" class="bg-muted/40 rounded-lg p-2.5">
        <p class="text-muted-foreground text-xs mb-0.5">文件大小</p>
        <p class="font-medium text-sm">
          {{ fileSizeText }}
        </p>
      </div>
      <!-- 文件格式 -->
      <div v-if="task.mediaFileFormat" class="bg-muted/40 rounded-lg p-2.5">
        <p class="text-muted-foreground text-xs mb-0.5">格式</p>
        <p class="font-medium text-sm uppercase">{{ task.mediaFileFormat }}</p>
      </div>
      <!-- HDR -->
      <div
        v-if="task.mediaVideoRange && task.mediaVideoRange !== 'SDR'"
        class="bg-muted/40 rounded-lg p-2.5"
      >
        <p class="text-muted-foreground text-xs mb-0.5">色域</p>
        <p class="font-medium text-sm">{{ task.mediaVideoRange }}</p>
      </div>
      <!-- 分片数 -->
      <div v-if="task.mediaSegmentCount" class="bg-muted/40 rounded-lg p-2.5">
        <p class="text-muted-foreground text-xs mb-0.5">分片数</p>
        <p class="font-medium text-sm">{{ task.mediaSegmentCount }}</p>
      </div>
      <!-- 是否加密 -->
      <div
        v-if="task.mediaIsEncrypted"
        class="bg-amber-500/10 rounded-lg p-2.5"
      >
        <p class="text-amber-500 text-xs mb-0.5">加密</p>
        <p class="font-medium text-sm text-amber-500">已加密</p>
      </div>
    </div>
  </div>
</template>
