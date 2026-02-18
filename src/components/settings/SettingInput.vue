<script setup lang="ts">
/**
 * SettingInput - 输入框设置项组件
 */

import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";

interface Props {
  modelValue?: string | number;
  label: string;
  placeholder?: string;
  type?: string;
  disabled?: boolean;
  min?: number;
  max?: number;
  inputClass?: string;
}

defineProps<Props>();

const emit = defineEmits<{
  (e: "update:modelValue", value: string | number): void;
}>();
</script>

<template>
  <div class="grid gap-2">
    <Label :class="{ 'opacity-50': disabled }">{{ label }}</Label>
    <Input
      :model-value="modelValue"
      :placeholder="placeholder"
      :type="type"
      :disabled="disabled"
      :min="min"
      :max="max"
      :class="inputClass"
      @update:model-value="emit('update:modelValue', $event)"
    />
  </div>
</template>
