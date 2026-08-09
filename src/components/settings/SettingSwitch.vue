<script setup lang="ts">
/**
 * SettingSwitch - 开关设置项组件
 * 统一的设置项布局和样式
 *
 * padded（默认）：独立成行，自带 px-5 py-4，配合 SettingsGroup 的 divide-y
 * padded=false：用于网格内，外层无水平内边距
 */

import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";

interface Props {
  modelValue?: boolean;
  label: string;
  description?: string;
  disabled?: boolean;
  /** 是否带独立行的内边距（网格内传 false） */
  padded?: boolean;
}

withDefaults(defineProps<Props>(), {
  padded: true,
});

const emit = defineEmits<{
  (e: "update:modelValue", value: boolean): void;
}>();
</script>

<template>
  <div
    class="flex items-center justify-between gap-6"
    :class="padded ? 'px-5 py-4' : 'py-2.5'"
  >
    <div class="min-w-0 space-y-0.5">
      <Label :class="{ 'opacity-50': disabled }">{{ label }}</Label>
      <p
        v-if="description"
        class="text-xs leading-relaxed text-muted-foreground"
      >
        {{ description }}
      </p>
    </div>
    <Switch
      class="shrink-0"
      :model-value="modelValue"
      :disabled="disabled"
      @update:model-value="emit('update:modelValue', $event)"
    />
  </div>
</template>
