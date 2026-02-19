<script setup lang="ts">
/**
 * TaskEmptyState - 任务空状态组件
 * 显示无任务时的占位内容
 */

import { AppIcon } from "@/components/common";

interface Props {
  type?: "active" | "completed" | "all";
  title?: string;
  description?: string;
}

const props = withDefaults(defineProps<Props>(), {
  type: "all",
});

const contentMap = {
  active: {
    icon: "Download",
    title: "没有下载任务",
    description: "输入链接开始下载",
  },
  completed: {
    icon: "CheckCircle",
    title: "没有已完成任务",
    description: "完成的下载会显示在这里",
  },
  all: {
    icon: "Inbox",
    title: "暂无任务",
    description: "输入链接开始下载",
  },
};

const content = computed(() => ({
  icon: contentMap[props.type].icon,
  title: props.title ?? contentMap[props.type].title,
  description: props.description ?? contentMap[props.type].description,
}));

import { computed } from "vue";
</script>

<template>
  <div class="flex flex-col items-center text-center">
    <div
      class="w-16 h-16 rounded-full bg-muted/50 flex items-center justify-center mb-4"
    >
      <AppIcon
        :name="content.icon as any"
        :size="28"
        class="text-muted-foreground/60"
      />
    </div>
    <p class="text-sm font-medium text-muted-foreground">{{ content.title }}</p>
    <p class="text-xs text-muted-foreground/70 mt-1">
      {{ content.description }}
    </p>
  </div>
</template>
