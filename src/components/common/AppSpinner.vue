<script setup lang="ts">
/**
 * AppSpinner - 加载动画组件
 * 用于显示加载状态
 */

import { computed } from 'vue';

type Variant = 'default' | 'primary' | 'success' | 'destructive';

interface Props {
  size?: 'sm' | 'md' | 'lg';
  variant?: Variant;
}

const props = withDefaults(defineProps<Props>(), {
  size: 'md',
  variant: 'default',
});

// 颜色配置
const colorClass = computed(() => {
  const colors: Record<Variant, string> = {
    default: 'text-foreground',
    primary: 'text-primary',
    success: 'text-green-600 dark:text-green-400',
    destructive: 'text-destructive',
  };
  return colors[props.variant];
});

// 尺寸配置
const sizeClass = computed(() => {
  const sizes = {
    sm: 'w-4 h-4',
    md: 'w-6 h-6',
    lg: 'w-8 h-8',
  };
  return sizes[props.size];
});

// 边框宽度
const borderClass = computed(() => {
  const widths = {
    sm: 'border-2',
    md: 'border-3',
    lg: 'border-4',
  };
  return widths[props.size];
});
</script>

<template>
  <div
    class="animate-spin rounded-full border-transparent border-t-current"
    :class="[sizeClass, colorClass, borderClass]"
  />
</template>
