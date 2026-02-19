<script setup lang="ts">
/**
 * TaskStatusBadge - 任务状态标签
 * 纯展示组件：显示在卡片右上角的状态标签
 */

interface Props {
  text: string;
  color: string;
}

defineProps<Props>();
</script>

<template>
  <span
    class="absolute top-2 right-2 shrink-0 px-1.5 py-0.5 rounded text-xs font-medium z-10"
    :style="{
      backgroundColor: `${color}20`,
      color: color,
    }"
  >
    {{ text }}
  </span>
</template>
