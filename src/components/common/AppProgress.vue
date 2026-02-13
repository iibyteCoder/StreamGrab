<script setup lang="ts">
/**
 * Progress 进度条组件
 */

import { computed } from 'vue';

type ProgressVariant = 'default' | 'success' | 'warning' | 'error';

interface Props {
  percent: number;
  variant?: ProgressVariant;
  showLabel?: boolean;
  striped?: boolean;
  animated?: boolean;
  size?: 'sm' | 'md' | 'lg';
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'default',
  showLabel: false,
  striped: false,
  animated: false,
  size: 'md',
});

// 限制百分比在 0-100 之间
const normalizedPercent = computed(() => {
  return Math.max(0, Math.min(100, props.percent));
});

const barClasses = computed(() => {
  const classes = ['transition-all duration-300 ease-out'];

  // 尺寸
  const sizeClasses = {
    sm: 'h-1',
    md: 'h-2',
    lg: 'h-3',
  };
  classes.push(sizeClasses[props.size]);

  // 变体颜色
  const variantClasses: Record<ProgressVariant, string> = {
    default: 'bg-accent-primary',
    success: 'bg-accent-success',
    warning: 'bg-yellow-500',
    error: 'bg-accent-error',
  };
  classes.push(variantClasses[props.variant]);

  // 条纹
  if (props.striped) {
    classes.push('bg-stripes');
  }

  return classes;
});

const trackClasses = computed(() => {
  return [
    'w-full rounded-full overflow-hidden bg-bg-elevated',
    props.size === 'sm' ? 'h-1' : props.size === 'lg' ? 'h-3' : 'h-2',
  ];
});
</script>

<template>
  <div class="w-full">
    <div class="flex items-center gap-2">
      <!-- 进度条 -->
      <div :class="trackClasses">
        <div
          :class="barClasses"
          :style="{ width: `${normalizedPercent}%` }"
          role="progressbar"
          :aria-valuenow="normalizedPercent"
          aria-valuemin="0"
          aria-valuemax="100"
        />
      </div>

      <!-- 百分比标签 -->
      <span
        v-if="showLabel"
        class="text-sm text-text-secondary min-w-[3rem] text-right"
      >
        {{ normalizedPercent.toFixed(1) }}%
      </span>
    </div>
  </div>
</template>

<style scoped>
.bg-stripes {
  background-image: linear-gradient(
    45deg,
    rgba(255, 255, 255, 0.15) 25%,
    transparent 25%,
    transparent 50%,
    rgba(255, 255, 255, 0.15) 50%,
    rgba(255, 255, 255, 0.15) 75%,
    transparent 75%,
    transparent
  );
  background-size: 1rem 1rem;
}
</style>
