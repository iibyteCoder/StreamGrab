<script setup lang="ts">
/**
 * TaskActionButtons - 任务操作按钮组
 * 纯展示组件：根据任务状态显示对应的操作按钮
 */

import { computed } from "vue";
import { Button } from "@/components/ui/button";
import { AppIcon } from "@/components/common";
import type { DownloadTask } from "@/domain";

interface Props {
  task: DownloadTask;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: "start"): void;
  (e: "pause"): void;
  (e: "resume"): void;
  (e: "stop"): void;
  (e: "retry"): void;
  (e: "openFile"): void;
}>();

const isCompleted = computed(() => props.task.status === "completed");
const isDownloading = computed(() => props.task.status === "downloading");
const isPaused = computed(() => props.task.status === "paused");
const isPending = computed(() => props.task.status === "pending");
const isFailed = computed(() => props.task.status === "failed");
const canStop = computed(() => isDownloading.value || isPaused.value);
</script>

<template>
  <div class="border-t p-3 shrink-0 space-y-2">
    <!-- 已完成：打开文件 -->
    <Button
      v-if="isCompleted"
      size="sm"
      class="w-full"
      @click="emit('openFile')"
    >
      <AppIcon name="Play" :size="14" class="mr-1.5" />
      播放文件
    </Button>

    <!-- 下载中：暂停 -->
    <Button
      v-if="isDownloading"
      size="sm"
      class="w-full"
      @click="emit('pause')"
    >
      <AppIcon name="Pause" :size="14" class="mr-1.5" />
      暂停下载
    </Button>

    <!-- 暂停/等待：继续 -->
    <Button
      v-if="isPaused || isPending"
      size="sm"
      class="w-full"
      @click="isPaused ? emit('resume') : emit('start')"
    >
      <AppIcon name="Play" :size="14" class="mr-1.5" />
      {{ isPaused ? "继续下载" : "开始下载" }}
    </Button>

    <!-- 失败：重试 -->
    <Button v-if="isFailed" size="sm" class="w-full" @click="emit('retry')">
      <AppIcon name="RefreshCw" :size="14" class="mr-1.5" />
      重新下载
    </Button>

    <!-- 下载中/暂停：停止 -->
    <Button
      v-if="canStop"
      variant="outline"
      size="sm"
      class="w-full"
      @click="emit('stop')"
    >
      <AppIcon name="Square" :size="14" class="mr-1.5" />
      停止下载
    </Button>
  </div>
</template>
