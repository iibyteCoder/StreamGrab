<script setup lang="ts">
/**
 * SettingPath - 路径选择设置项组件
 * 支持目录选择按钮
 */

import { ref } from "vue";
import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { AppIcon } from "@/components/common";
import { systemService } from "@/services";
import { useToast } from "@/composables";

interface Props {
  modelValue?: string;
  label: string;
  placeholder?: string;
  disabled?: boolean;
  /** 选择类型：文件夹或文件 */
  type?: "folder" | "file";
  /** 是否带独立行的内边距（网格内传 false） */
  padded?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: "",
  placeholder: "",
  disabled: false,
  type: "folder",
  padded: true,
});

const emit = defineEmits<{
  (e: "update:modelValue", value: string): void;
  (e: "select", path: string): void;
}>();

const toast = useToast();
const isSelecting = ref(false);

/**
 * 打开目录选择对话框
 */
const handleSelect = async () => {
  if (isSelecting.value || props.disabled) return;

  isSelecting.value = true;
  try {
    let path: string | null = null;

    if (props.type === "folder") {
      path = await systemService.selectDirectory();
    } else {
      path = await systemService.selectFile();
    }

    if (path) {
      emit("update:modelValue", path);
      emit("select", path);
    }
  } catch (error) {
    console.error("Failed to select path:", error);
    toast.error("选择路径失败");
  } finally {
    isSelecting.value = false;
  }
};
</script>

<template>
  <div class="grid gap-2" :class="padded ? 'px-5 py-4' : 'py-2.5'">
    <Label :class="{ 'opacity-50': disabled }">{{ label }}</Label>
    <div class="flex gap-2">
      <Input
        :model-value="modelValue"
        :placeholder="placeholder"
        :disabled="disabled"
        class="flex-1"
        @update:model-value="
          (val: string | number) => emit('update:modelValue', String(val))
        "
      />
      <Button
        variant="outline"
        size="icon"
        :disabled="disabled || isSelecting"
        :title="type === 'folder' ? '选择文件夹' : '选择文件'"
        @click="handleSelect"
      >
        <AppIcon
          v-if="isSelecting"
          name="Loader2"
          :size="16"
          class="animate-spin"
        />
        <AppIcon v-else name="FolderOpen" :size="16" />
      </Button>
    </div>
  </div>
</template>
