<script setup lang="ts">
/**
 * TaskStatusHeader - 任务状态头部
 * 纯展示组件：显示状态图标、文件名、状态文本和关闭按钮
 */

import { computed } from "vue";
import { Button } from "@/components/ui/button";
import { AppIcon } from "@/components/common";
import { TASK_STATUS_CONFIG } from "@/utils/constants";
import type { DownloadTask } from "@/domain";

interface Props {
  task: DownloadTask;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: "close"): void;
}>();

const statusConfig = computed(() => {
  const config = TASK_STATUS_CONFIG[props.task.status];
  return config ?? TASK_STATUS_CONFIG.pending!;
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

const headerStyle = computed(() => ({
  background: `linear-gradient(135deg, ${statusConfig.value.color}08, ${statusConfig.value.color}15)`,
}));

const iconBgStyle = computed(() => ({
  backgroundColor: `${statusConfig.value.color}20`,
}));

const iconStyle = computed(() => ({
  color: statusConfig.value.color,
}));
</script>

<template>
  <div
    class="flex items-center gap-2 px-4 py-3 border-b shrink-0"
    :style="headerStyle"
  >
    <div
      class="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg"
      :style="iconBgStyle"
    >
      <AppIcon :name="statusIcon as any" :size="18" :style="iconStyle" />
    </div>
    <div class="min-w-0 flex-1">
      <p class="font-semibold text-sm truncate">
        {{ task.fileName || "未命名文件" }}
      </p>
      <p class="text-xs font-medium" :style="{ color: statusConfig.color }">
        {{ statusConfig.text }}
      </p>
    </div>
    <Button
      variant="ghost"
      size="icon"
      class="h-8 w-8 shrink-0"
      @click="emit('close')"
    >
      <AppIcon name="X" :size="16" />
    </Button>
  </div>
</template>
