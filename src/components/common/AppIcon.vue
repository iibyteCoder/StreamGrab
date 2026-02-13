<script setup lang="ts">
/**
 * AppIcon - 统一的图标组件
 * 基于 lucide-vue-next 图标库
 */

import * as Icons from 'lucide-vue-next';
import { computed } from 'vue';

// Lucide 图标名称类型（从 lucide-vue-next 导入的图标组件名）
type IconName = keyof typeof Icons;

interface Props {
  name: IconName;
  size?: number | string;
  color?: string;
  className?: string;
}

const props = withDefaults(defineProps<Props>(), {
  size: 20,
  color: 'currentColor',
  className: '',
});

// 动态图标组件
const iconComponent = computed(() => {
  return Icons[props.name];
});
</script>

<template>
  <component
    :is="iconComponent"
    :size="typeof size === 'number' ? size : undefined"
    :stroke-width="1.5"
    :class="className"
    :style="{ color }"
  />
</template>
