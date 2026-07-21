<script setup lang="ts">
/**
 * TaskDeleteDialog - 任务删除确认对话框
 * 纯展示组件：显示删除确认对话框
 */

import { ref, watch } from "vue";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import type { DownloadTask } from "@/domain";

interface Props {
  open: boolean;
  task: DownloadTask;
  fileExists: boolean;
  isDeleting?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  isDeleting: false,
});

const emit = defineEmits<{
  (e: "update:open", value: boolean): void;
  (e: "confirm", withFile: boolean): void;
}>();

const deleteWithFile = ref(false);

// 当对话框打开时重置选项
watch(
  () => props.open,
  (newOpen) => {
    if (newOpen) {
      deleteWithFile.value = false;
    }
  },
);

const handleConfirm = () => {
  emit("confirm", deleteWithFile.value);
};

const handleCancel = () => {
  emit("update:open", false);
};
</script>

<template>
  <Dialog :open="open" @update:open="emit('update:open', $event)">
    <DialogContent class="sm:max-w-[400px] overflow-hidden">
      <DialogHeader>
        <DialogTitle>确认删除</DialogTitle>
        <DialogDescription>确定要删除此任务记录吗？</DialogDescription>
      </DialogHeader>

      <div class="py-4">
        <label class="flex items-center gap-2 cursor-pointer select-none">
          <input
            type="checkbox"
            v-model="deleteWithFile"
            class="w-4 h-4 rounded border accent-primary shrink-0"
          />
          <span class="text-sm">同时删除下载的文件</span>
        </label>
        <p
          v-if="deleteWithFile && task.outputPath"
          class="mt-2 text-xs text-muted-foreground break-all"
        >
          {{ task.outputPath }}
        </p>
      </div>

      <DialogFooter class="flex-col sm:flex-row gap-2">
        <Button
          variant="outline"
          class="w-full sm:w-auto"
          @click="handleCancel"
        >
          取消
        </Button>
        <Button
          variant="destructive"
          class="w-full sm:w-auto"
          :disabled="isDeleting"
          @click="handleConfirm"
        >
          {{ isDeleting ? "删除中..." : "确认删除" }}
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
