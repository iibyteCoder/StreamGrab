<script setup lang="ts">
/**
 * TaskBasicInfo - 任务基本信息展示
 * 纯展示组件：显示创建时间、完成时间和保存位置
 */

import { computed } from "vue";
import { Button } from "@/components/ui/button";
import { formatDate } from "@/utils/format";
import type { DownloadTask } from "@/types";

interface Props {
  task: DownloadTask;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: "openFolder"): void;
}>();

const createdAt = computed(() =>
  props.task.createdAt ? formatDate(props.task.createdAt) : "",
);

const completedAt = computed(() =>
  props.task.completedAt ? formatDate(props.task.completedAt) : "",
);
</script>

<template>
  <div class="space-y-2">
    <h4
      class="text-xs font-semibold text-muted-foreground uppercase tracking-wide"
    >
      基本信息
    </h4>
    <div class="bg-muted/30 rounded-lg divide-y divide-border/50">
      <div class="flex justify-between items-center px-3 py-2 text-sm">
        <span class="text-muted-foreground">创建时间</span>
        <span class="text-xs">{{ createdAt }}</span>
      </div>
      <div
        v-if="completedAt"
        class="flex justify-between items-center px-3 py-2 text-sm"
      >
        <span class="text-muted-foreground">完成时间</span>
        <span class="text-xs">{{ completedAt }}</span>
      </div>
      <div class="flex justify-between items-center px-3 py-2 text-sm">
        <span class="text-muted-foreground">保存位置</span>
        <Button
          variant="link"
          size="sm"
          class="h-auto p-0 text-xs"
          @click="emit('openFolder')"
        >
          打开目录
        </Button>
      </div>
    </div>
  </div>
</template>
