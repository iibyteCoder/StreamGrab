<script setup lang="ts">
/**
 * SettingSlider - 滑块设置项组件
 */

import { Label } from '@/components/ui/label';
import { Slider } from '@/components/ui/slider';

interface Props {
  modelValue?: number;
  label: string;
  min?: number;
  max?: number;
  step?: number;
  disabled?: boolean;
  displayValue?: string;
}

const props = withDefaults(defineProps<Props>(), {
  min: 0,
  max: 100,
  step: 1,
});

const emit = defineEmits<{
  (e: 'update:modelValue', value: number): void;
}>();
</script>

<template>
  <div class="grid gap-3">
    <div class="flex items-center justify-between">
      <Label :class="{ 'opacity-50': disabled }">{{ label }}</Label>
      <span v-if="displayValue" class="text-sm text-muted-foreground">
        {{ displayValue }}
      </span>
    </div>
    <Slider
      :model-value="[modelValue ?? 0]"
      :min="min"
      :max="max"
      :step="step"
      :disabled="disabled"
      @update:model-value="emit('update:modelValue', $event[0])"
    />
  </div>
</template>
