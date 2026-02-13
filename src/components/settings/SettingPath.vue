<script setup lang="ts">
/**
 * SettingPath - 路径选择设置项组件
 * 支持目录选择按钮
 */

import { Label } from '@/components/ui/label';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { AppIcon } from '@/components/common';
import { FolderOpen } from 'lucide-vue-next';

interface Props {
  modelValue?: string;
  label: string;
  placeholder?: string;
  disabled?: boolean;
}

defineProps<Props>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void;
  (e: 'select'): void;
}>();
</script>

<template>
  <div class="grid gap-2">
    <Label :class="{ 'opacity-50': disabled }">{{ label }}</Label>
    <div class="flex gap-2">
      <Input
        :model-value="modelValue"
        :placeholder="placeholder"
        :disabled="disabled"
        class="flex-1"
        @update:model-value="emit('update:modelValue', $event)"
      />
      <Button
        variant="outline"
        size="icon"
        :disabled="disabled"
        @click="emit('select')"
      >
        <AppIcon :name="FolderOpen" :size="16" />
      </Button>
    </div>
  </div>
</template>
