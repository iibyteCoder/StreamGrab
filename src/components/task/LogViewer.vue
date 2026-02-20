<script setup lang="ts">
/**
 * LogViewer - 任务日志查看器组件
 * 显示下载任务的实时日志
 */

import { computed, ref, watch, nextTick } from "vue";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { AppIcon } from "@/components/common";
import { useTaskStore } from "@/stores";

interface Props {
  open: boolean;
  taskId: string | null;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: "update:open", value: boolean): void;
}>();

const taskStore = useTaskStore();

// 自动滚动开关
const autoScroll = ref(true);

// 日志容器引用
const scrollAreaRef = ref<InstanceType<typeof ScrollArea> | null>(null);

// 获取任务
const task = computed(() => {
  if (!props.taskId) return null;
  return taskStore.getTask(props.taskId);
});

// 获取日志
const logs = computed(() => {
  if (!props.taskId) return [];
  return taskStore.getTaskLogs(props.taskId);
});

// 日志级别颜色
const levelColors: Record<string, string> = {
  info: "text-foreground",
  warn: "text-yellow-500",
  error: "text-red-500",
  debug: "text-muted-foreground",
};

// 格式化时间
const formatTime = (date: Date): string => {
  return date.toLocaleTimeString("zh-CN", {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
};

// 自动滚动到底部
watch(
  () => logs.value.length,
  async () => {
    if (autoScroll.value) {
      await nextTick();
      const viewport = scrollAreaRef.value?.$el?.querySelector(
        "[data-radix-scroll-area-viewport]",
      );
      if (viewport) {
        viewport.scrollTop = viewport.scrollHeight;
      }
    }
  },
);

// 清除日志
const clearLogs = () => {
  if (props.taskId) {
    taskStore.clearTaskLogs(props.taskId);
  }
};

// 关闭对话框
const closeDialog = () => {
  emit("update:open", false);
};
</script>

<template>
  <Dialog :open="open" @update:open="emit('update:open', $event)">
    <DialogContent class="max-w-3xl h-[70vh] flex flex-col overflow-hidden">
      <DialogHeader class="shrink-0">
        <div class="flex items-center justify-between">
          <DialogTitle class="text-base">
            任务日志
            <span v-if="task" class="text-muted-foreground font-normal ml-2">
              {{ task.fileName }}
            </span>
          </DialogTitle>
          <div class="flex items-center gap-2">
            <Button
              variant="ghost"
              size="sm"
              class="h-7 px-2 text-xs"
              @click="autoScroll = !autoScroll"
            >
              <AppIcon
                name="ArrowDown"
                :size="12"
                class="mr-1"
                :class="autoScroll ? 'text-primary' : ''"
              />
              自动滚动
            </Button>
            <Button
              variant="ghost"
              size="sm"
              class="h-7 px-2 text-xs"
              @click="clearLogs"
            >
              <AppIcon name="Trash2" :size="12" class="mr-1" />
              清除
            </Button>
          </div>
        </div>
      </DialogHeader>

      <div class="flex-1 min-h-0 border rounded-md bg-muted/30 overflow-hidden">
        <ScrollArea ref="scrollAreaRef" class="h-full">
          <div
            v-if="logs.length === 0"
            class="flex flex-col items-center justify-center h-full min-h-[300px] text-center"
          >
            <div
              class="w-14 h-14 rounded-full bg-muted/50 flex items-center justify-center mb-3"
            >
              <AppIcon
                name="ScrollText"
                :size="24"
                class="text-muted-foreground/60"
              />
            </div>
            <p class="text-sm font-medium text-muted-foreground">暂无日志</p>
            <p class="text-xs text-muted-foreground/70 mt-1">
              开始下载后将显示日志
            </p>
          </div>
          <div v-else class="p-2 font-mono text-xs space-y-0.5">
            <div
              v-for="(log, index) in logs"
              :key="index"
              class="flex items-start gap-2 py-0.5 hover:bg-muted/50 rounded px-1"
            >
              <span class="text-muted-foreground shrink-0">
                {{ formatTime(log.timestamp) }}
              </span>
              <!-- 日志级别图标 -->
              <AppIcon
                v-if="log.level === 'info'"
                name="Info"
                :size="12"
                class="shrink-0 mt-0.5 text-foreground"
              />
              <AppIcon
                v-else-if="log.level === 'warn'"
                name="AlertTriangle"
                :size="12"
                class="shrink-0 mt-0.5 text-yellow-500"
              />
              <AppIcon
                v-else-if="log.level === 'error'"
                name="XCircle"
                :size="12"
                class="shrink-0 mt-0.5 text-red-500"
              />
              <AppIcon
                v-else
                name="Bug"
                :size="12"
                class="shrink-0 mt-0.5 text-muted-foreground"
              />
              <span :class="levelColors[log.level]" class="break-all">
                {{ log.message }}
              </span>
            </div>
          </div>
        </ScrollArea>
      </div>

      <div
        class="shrink-0 flex justify-between items-center pt-2 text-xs text-muted-foreground"
      >
        <span>共 {{ logs.length }} 条日志</span>
        <Button variant="outline" size="sm" @click="closeDialog"> 关闭 </Button>
      </div>
    </DialogContent>
  </Dialog>
</template>
