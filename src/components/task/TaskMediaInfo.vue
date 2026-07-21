<script setup lang="ts">
/**
 * TaskMediaInfo - 媒体/文件信息展示
 * 使用领域模型嵌套 mediaInfo 数据
 */

import { computed } from "vue";
import { formatFileSize, formatDuration } from "@/utils/format";
import type { DownloadTask } from "@/domain";

interface Props {
  task: DownloadTask;
}

const props = defineProps<Props>();

// 是否有媒体信息可显示
const hasMediaInfo = computed(() => {
  const m = props.task.mediaInfo;
  return (
    m?.resolution ||
    m?.duration ||
    m?.videoCodec ||
    m?.frameRate ||
    m?.segmentCount ||
    m?.isEncrypted ||
    props.task.progress.totalSize ||
    props.task.progress.downloadedSize ||
    m?.fileFormat ||
    m?.fileSize
  );
});

// 文件大小显示文本
const fileSizeText = computed(() => {
  const m = props.task.mediaInfo;
  if (m?.fileSize) {
    return formatFileSize(m.fileSize);
  }
  if (props.task.progress.totalSize) {
    return formatFileSize(props.task.progress.totalSize);
  }
  if (props.task.progress.downloadedSize) {
    return `${formatFileSize(props.task.progress.downloadedSize)} (已下载)`;
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
      <div
        v-if="task.mediaInfo?.resolution"
        class="bg-muted/40 rounded-lg p-2.5"
      >
        <p class="text-muted-foreground text-xs mb-0.5">分辨率</p>
        <p class="font-medium text-sm">{{ task.mediaInfo.resolution }}</p>
      </div>
      <!-- 时长 -->
      <div v-if="task.mediaInfo?.duration" class="bg-muted/40 rounded-lg p-2.5">
        <p class="text-muted-foreground text-xs mb-0.5">时长</p>
        <p class="font-medium text-sm">
          {{ formatDuration(task.mediaInfo.duration) }}
        </p>
      </div>
      <!-- 视频编码 -->
      <div
        v-if="task.mediaInfo?.videoCodec"
        class="bg-muted/40 rounded-lg p-2.5"
      >
        <p class="text-muted-foreground text-xs mb-0.5">视频编码</p>
        <p class="font-medium text-sm uppercase">
          {{ task.mediaInfo.videoCodec }}
        </p>
      </div>
      <!-- 帧率 -->
      <div
        v-if="task.mediaInfo?.frameRate"
        class="bg-muted/40 rounded-lg p-2.5"
      >
        <p class="text-muted-foreground text-xs mb-0.5">帧率</p>
        <p class="font-medium text-sm">{{ task.mediaInfo.frameRate }} fps</p>
      </div>
      <!-- 文件大小 -->
      <div v-if="fileSizeText" class="bg-muted/40 rounded-lg p-2.5">
        <p class="text-muted-foreground text-xs mb-0.5">文件大小</p>
        <p class="font-medium text-sm">
          {{ fileSizeText }}
        </p>
      </div>
      <!-- 文件格式 -->
      <div
        v-if="task.mediaInfo?.fileFormat"
        class="bg-muted/40 rounded-lg p-2.5"
      >
        <p class="text-muted-foreground text-xs mb-0.5">格式</p>
        <p class="font-medium text-sm uppercase">
          {{ task.mediaInfo.fileFormat }}
        </p>
      </div>
      <!-- HDR -->
      <div
        v-if="task.mediaInfo?.videoRange && task.mediaInfo.videoRange !== 'SDR'"
        class="bg-muted/40 rounded-lg p-2.5"
      >
        <p class="text-muted-foreground text-xs mb-0.5">色域</p>
        <p class="font-medium text-sm">{{ task.mediaInfo.videoRange }}</p>
      </div>
      <!-- 分片数 -->
      <div
        v-if="task.mediaInfo?.segmentCount"
        class="bg-muted/40 rounded-lg p-2.5"
      >
        <p class="text-muted-foreground text-xs mb-0.5">分片数</p>
        <p class="font-medium text-sm">{{ task.mediaInfo.segmentCount }}</p>
      </div>
      <!-- 是否加密 -->
      <div
        v-if="task.mediaInfo?.isEncrypted"
        class="bg-amber-500/10 rounded-lg p-2.5"
      >
        <p class="text-amber-500 text-xs mb-0.5">加密</p>
        <p class="font-medium text-sm text-amber-500">已加密</p>
      </div>
    </div>
  </div>
</template>
