<script setup lang="ts">
/**
 * TemplateDeleteDialog - 模板删除确认对话框
 * 纯展示组件：确认删除模板
 */

import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";

interface Props {
  open: boolean;
  templateName?: string;
}

defineProps<Props>();

const emit = defineEmits<{
  (e: "update:open", value: boolean): void;
  (e: "confirm"): void;
  (e: "cancel"): void;
}>();

const handleCancel = () => {
  emit("cancel");
  emit("update:open", false);
};

const handleConfirm = () => {
  emit("confirm");
};
</script>

<template>
  <Dialog :open="open" @update:open="emit('update:open', $event)">
    <DialogContent class="sm:max-w-sm">
      <DialogHeader>
        <DialogTitle>确认删除</DialogTitle>
      </DialogHeader>
      <p class="text-sm text-muted-foreground">
        确定要删除模板 "{{ templateName }}" 吗？此操作不可恢复。
      </p>
      <DialogFooter>
        <Button variant="outline" @click="handleCancel">取消</Button>
        <Button variant="destructive" @click="handleConfirm">删除</Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
