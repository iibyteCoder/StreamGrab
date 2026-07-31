<script setup lang="ts">
/**
 * 流选择器（独立弹窗形态）。薄 Dialog 外壳，包 StreamPickerInline。
 * AddTaskDialog 重构后不再直接使用；保留以供独立调用与导出稳定。
 */
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { AppIcon } from "@/components/common";
import StreamPickerInline from "./StreamPickerInline.vue";
import type { StreamInfo, StreamSelection } from "@/domain";

const props = defineProps<{
  open: boolean;
  streamInfo: StreamInfo | null;
  loading?: boolean;
}>();

const emit = defineEmits<{
  (e: "update:open", value: boolean): void;
  (e: "confirm", selection: StreamSelection): void;
  (e: "cancel"): void;
}>();

const close = () => emit("update:open", false);
const onConfirm = (s: StreamSelection) => {
  emit("confirm", s);
  close();
};
const onCancel = () => {
  emit("cancel");
  close();
};
</script>

<template>
  <Dialog :open="props.open" @update:open="emit('update:open', $event)">
    <DialogContent class="flex max-h-[85vh] max-w-2xl flex-col">
      <DialogHeader>
        <DialogTitle class="flex items-center gap-2">
          <AppIcon name="ListVideo" :size="20" />
          选择流
        </DialogTitle>
        <DialogDescription>选择要下载的视频/音频/字幕流</DialogDescription>
      </DialogHeader>
      <StreamPickerInline
        :stream-info="props.streamInfo"
        :loading="props.loading"
        @confirm="onConfirm"
        @cancel="onCancel"
      />
    </DialogContent>
  </Dialog>
</template>
