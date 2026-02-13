<script setup lang="ts">
/**
 * SettingSwitch - 开关设置项组件
 * 统一的设置项布局和样式
 */

import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';

interface Props {
  modelValue?: boolean;
  label: string;
  description?: string;
  disabled?: boolean;
}

defineProps<Props>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
}>();
</script>

<template>
  <div class="flex items-center justify-between">
    <div class="space-y-0.5">
      <Label :class="{ 'opacity-50': disabled }">{{ label }}</Label>
      <p v-if="description" class="text-xs text-muted-foreground">
        {{ description }}
      </p>
    </div>
    <Switch
      :checked="modelValue"
      :disabled="disabled"
      @update:checked="emit('update:modelValue', $event)"
    />
  </div>
</template>
