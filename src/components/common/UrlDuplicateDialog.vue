<script setup lang="ts">
/**
 * UrlDuplicateDialog - URL 重复确认弹窗
 * 当用户尝试下载已存在的 URL 时显示
 */

import { computed } from "vue";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { AppIcon } from "@/components/common";
import type { DownloadTask } from "@/types";

interface Props {
  open: boolean;
  existingTask: DownloadTask | null;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: "update:open", value: boolean): void;
  (e: "confirm"): void;
  (e: "cancel"): void;
}>();

const isOpen = computed({
  get: () => props.open,
  set: (value) => emit("update:open", value),
});

const handleConfirm = () => {
  emit("confirm");
  isOpen.value = false;
};

const handleCancel = () => {
  emit("cancel");
  isOpen.value = false;
};
</script>

<template>
  <Dialog v-model:open="isOpen">
    <DialogContent class="sm:max-w-[440px]">
      <DialogHeader>
        <DialogTitle class="flex items-center gap-2">
          <AppIcon name="AlertCircle" :size="20" class="text-warning" />
          链接已存在
        </DialogTitle>
        <DialogDescription class="pt-2">
          该链接已存在于任务列表中
        </DialogDescription>
      </DialogHeader>

      <!-- 已存在任务信息 -->
      <div
        v-if="existingTask"
        class="p-3 bg-muted/50 rounded-lg text-sm space-y-1"
      >
        <div class="flex justify-between">
          <span class="text-muted-foreground">文件名:</span>
          <span class="font-medium truncate ml-2 max-w-[200px]">{{
            existingTask.fileName
          }}</span>
        </div>
        <div class="flex justify-between">
          <span class="text-muted-foreground">状态:</span>
          <span class="ml-2">{{
            existingTask.status === "completed" ? "已完成" : existingTask.status
          }}</span>
        </div>
      </div>

      <!-- 操作提示 -->
      <p class="text-sm text-muted-foreground">
        是否仍要下载？这将创建一个新任务并自动重命名文件。
      </p>

      <!-- 操作按钮 -->
      <div class="flex justify-end gap-2">
        <Button variant="outline" @click="handleCancel">取消</Button>
        <Button @click="handleConfirm">
          <AppIcon name="Download" :size="16" class="mr-2" />
          仍然下载
        </Button>
      </div>
    </DialogContent>
  </Dialog>
</template>
