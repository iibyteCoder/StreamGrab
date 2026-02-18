<script setup lang="ts">
/**
 * AppBadge - 状态标签组件
 * 用于显示状态、标签等
 */

import { computed } from "vue";

type Variant = "default" | "primary" | "success" | "warning" | "error" | "info";

interface Props {
  variant?: Variant;
  size?: "sm" | "md" | "lg";
  dot?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  variant: "default",
  size: "md",
  dot: false,
});

// 样式配置
const badgeStyles = computed(() => {
  const variants: Record<Variant, { bg: string; text: string }> = {
    default: { bg: "bg-muted", text: "text-muted-foreground" },
    primary: { bg: "bg-primary/10", text: "text-primary" },
    success: {
      bg: "bg-green-500/10",
      text: "text-green-600 dark:text-green-400",
    },
    warning: {
      bg: "bg-yellow-500/10",
      text: "text-yellow-600 dark:text-yellow-400",
    },
    error: { bg: "bg-destructive/10", text: "text-destructive" },
    info: { bg: "bg-blue-500/10", text: "text-blue-600 dark:text-blue-400" },
  };

  return variants[props.variant];
});

// 尺寸配置
const sizeClasses = computed(() => {
  const sizes = {
    sm: "px-1.5 py-0.5 text-xs",
    md: "px-2 py-1 text-sm",
    lg: "px-3 py-1.5 text-base",
  };
  return sizes[props.size];
});
</script>

<template>
  <span
    class="inline-flex items-center gap-1.5 rounded-full font-medium transition-colors"
    :class="[badgeStyles.bg, badgeStyles.text, sizeClasses]"
  >
    <span v-if="dot" class="h-1.5 w-1.5 rounded-full bg-current" />
    <slot />
  </span>
</template>
