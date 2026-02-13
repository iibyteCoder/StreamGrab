<script setup lang="ts">
/**
 * SettingSelect - 下拉选择设置项组件
 */

import { Label } from '@/components/ui/label';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';

interface Option {
  value: string;
  label: string;
}

interface Props {
  modelValue?: string;
  label: string;
  options: Option[];
  placeholder?: string;
  disabled?: boolean;
}

defineProps<Props>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void;
}>();
</script>

<template>
  <div class="grid gap-2">
    <Label :class="{ 'opacity-50': disabled }">{{ label }}</Label>
    <Select :model-value="modelValue" :disabled="disabled" @update:model-value="(val: unknown) => typeof val === 'string' && emit('update:modelValue', val)">
      <SelectTrigger class="w-full">
        <SelectValue :placeholder="placeholder" />
      </SelectTrigger>
      <SelectContent>
        <SelectItem v-for="option in options" :key="option.value" :value="option.value">
          {{ option.label }}
        </SelectItem>
      </SelectContent>
    </Select>
  </div>
</template>
