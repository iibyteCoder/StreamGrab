<script setup lang="ts">
/**
 * SettingInput - 输入框设置项组件
 */

import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
import { Tooltip } from "@/components/ui/tooltip";
import { HelpCircle } from "lucide-vue-next";

interface Props {
  modelValue?: string | number;
  label: string;
  placeholder?: string;
  type?: string;
  disabled?: boolean;
  min?: number;
  max?: number;
  inputClass?: string;
  /** 帮助提示文本，显示问号图标 hover 时展示 */
  help?: string;
}

defineProps<Props>();

const emit = defineEmits<{
  (e: "update:modelValue", value: string | number): void;
  (e: "blur"): void;
}>();
</script>

<template>
  <div class="grid gap-2">
    <div class="flex items-center gap-1.5">
      <Label :class="{ 'opacity-50': disabled }">{{ label }}</Label>
      <Tooltip v-if="help" :content="help" side="right">
        <HelpCircle
          class="h-3.5 w-3.5 text-muted-foreground cursor-help hover:text-foreground transition-colors"
        />
      </Tooltip>
    </div>
    <Input
      :model-value="modelValue"
      :placeholder="placeholder"
      :type="type"
      :disabled="disabled"
      :min="min"
      :max="max"
      :class="inputClass"
      @update:model-value="emit('update:modelValue', $event)"
      @blur="emit('blur')"
    />
  </div>
</template>
