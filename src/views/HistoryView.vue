<script setup lang="ts">
/**
 * HistoryView - 历史记录页面
 * 显示下载历史（任务终态快照）
 */

import { onMounted, ref } from "vue";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { AppIcon } from "@/components/common";
import { useHistoryStore } from "@/stores";
import { useDownloader } from "@/composables";
import { systemService } from "@/services";
import { formatFileSize, formatDateTime } from "@/utils/format";
import { TASK_STATUS_CONFIG } from "@/utils/constants";
import type { HistoryRecord, TaskStatus } from "@/domain";

const historyStore = useHistoryStore();
const { addAndStartTask } = useDownloader();

// 清空确认对话框
const showClearDialog = ref(false);

// 挂载时加载历史
onMounted(() => {
  if (!historyStore.loaded) {
    historyStore.loadHistory();
  }
});

// 状态徽章配置
const getStatusConfig = (status: TaskStatus) => {
  return TASK_STATUS_CONFIG[status] ?? TASK_STATUS_CONFIG.pending!;
};

// 重新下载
const handleReDownload = async (record: HistoryRecord) => {
  try {
    const overrides = record.overrides ?? undefined;
    await addAndStartTask(
      record.url,
      record.fileName,
      record.saveDir,
      overrides,
    );
  } catch (e) {
    console.error("Failed to re-download:", e);
  }
};

// 打开文件夹
const handleOpenFolder = async (record: HistoryRecord) => {
  if (record.outputPath) {
    try {
      await systemService.openFileInExplorer(record.outputPath);
    } catch (e) {
      console.error("Failed to open folder:", e);
    }
  }
};

// 删除记录
const handleDeleteRecord = async (record: HistoryRecord) => {
  await historyStore.removeRecord(record.id);
};

// 清空历史
const handleClearAll = async () => {
  await historyStore.clearAll();
  showClearDialog.value = false;
};

// 截断 URL 显示
const truncateUrl = (url: string, maxLen = 60) => {
  if (url.length <= maxLen) return url;
  return url.slice(0, maxLen - 3) + "...";
};
</script>

<template>
  <div class="h-full flex flex-col">
    <!-- 头部 -->
    <div class="border-b px-6 py-4 shrink-0 flex items-center justify-between">
      <div>
        <h1 class="text-lg font-semibold">下载历史</h1>
        <p class="text-sm text-muted-foreground mt-0.5">
          共 {{ historyStore.count }} 条记录
        </p>
      </div>
      <Button
        v-if="historyStore.count > 0"
        variant="outline"
        size="sm"
        class="text-destructive hover:text-destructive"
        @click="showClearDialog = true"
      >
        <AppIcon name="Trash2" :size="14" class="mr-2" />
        清空历史
      </Button>
    </div>

    <!-- 内容区 -->
    <div class="flex-1 min-h-0 overflow-y-auto">
      <!-- 加载中 -->
      <div
        v-if="!historyStore.loaded"
        class="flex items-center justify-center h-64"
      >
        <AppIcon
          name="Loader2"
          :size="24"
          class="animate-spin text-muted-foreground"
        />
      </div>

      <!-- 空状态 -->
      <div
        v-else-if="historyStore.records.length === 0"
        class="flex flex-col items-center justify-center h-64 text-center"
      >
        <div
          class="w-16 h-16 rounded-full bg-muted/50 flex items-center justify-center mb-4"
        >
          <AppIcon name="History" :size="28" class="text-muted-foreground/60" />
        </div>
        <p class="text-sm font-medium text-muted-foreground">暂无下载记录</p>
        <p class="text-xs text-muted-foreground/70 mt-1">
          完成的下载任务会出现在这里
        </p>
      </div>

      <!-- 历史列表 -->
      <div v-else class="p-4 space-y-2">
        <TransitionGroup name="list">
          <div
            v-for="record in historyStore.records"
            :key="record.id"
            class="group flex items-center gap-3 p-3 rounded-lg border bg-card hover:bg-accent/50 transition-colors"
          >
            <!-- 状态指示器 -->
            <div
              class="flex h-9 w-9 shrink-0 items-center justify-center rounded-full"
              :style="{
                backgroundColor: `${getStatusConfig(record.status).color}20`,
              }"
            >
              <AppIcon
                :name="
                  record.status === 'completed'
                    ? 'CheckCircle'
                    : record.status === 'failed'
                      ? 'XCircle'
                      : 'X'
                "
                :size="16"
                :style="{ color: getStatusConfig(record.status).color }"
              />
            </div>

            <!-- 信息 -->
            <div class="flex-1 min-w-0">
              <p class="text-sm font-medium truncate">
                {{ record.fileName }}
              </p>
              <div
                class="flex items-center gap-2 mt-0.5 text-xs text-muted-foreground"
              >
                <span :title="record.url">{{ truncateUrl(record.url) }}</span>
                <span v-if="record.fileSize">
                  · {{ formatFileSize(record.fileSize) }}
                </span>
                <span>· {{ formatDateTime(record.completedAt) }}</span>
              </div>
            </div>

            <!-- 状态徽章 -->
            <span
              class="shrink-0 px-2 py-0.5 rounded-full text-xs font-medium"
              :style="{
                backgroundColor: `${getStatusConfig(record.status).color}20`,
                color: getStatusConfig(record.status).color,
              }"
            >
              {{ getStatusConfig(record.status).text }}
            </span>

            <!-- 操作按钮 -->
            <div
              class="shrink-0 flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity"
            >
              <button
                class="p-1.5 rounded-md hover:bg-accent text-muted-foreground hover:text-foreground transition-colors cursor-pointer"
                title="重新下载"
                @click="handleReDownload(record)"
              >
                <AppIcon name="RefreshCw" :size="15" />
              </button>
              <button
                v-if="record.outputPath"
                class="p-1.5 rounded-md hover:bg-accent text-muted-foreground hover:text-foreground transition-colors cursor-pointer"
                title="打开文件夹"
                @click="handleOpenFolder(record)"
              >
                <AppIcon name="FolderOpen" :size="15" />
              </button>
              <button
                class="p-1.5 rounded-md hover:bg-accent text-destructive transition-colors cursor-pointer"
                title="删除记录"
                @click="handleDeleteRecord(record)"
              >
                <AppIcon name="Trash2" :size="15" />
              </button>
            </div>
          </div>
        </TransitionGroup>
      </div>
    </div>

    <!-- 清空确认对话框 -->
    <Dialog v-model:open="showClearDialog">
      <DialogContent class="sm:max-w-[400px]">
        <DialogHeader>
          <DialogTitle>清空历史</DialogTitle>
          <DialogDescription>
            确定要清空所有下载历史吗？此操作不可撤销。
          </DialogDescription>
        </DialogHeader>
        <DialogFooter class="flex-col sm:flex-row gap-2">
          <Button
            variant="outline"
            class="w-full sm:w-auto"
            @click="showClearDialog = false"
          >
            取消
          </Button>
          <Button
            variant="destructive"
            class="w-full sm:w-auto"
            @click="handleClearAll"
          >
            确认清空
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </div>
</template>

<style scoped>
.list-enter-active,
.list-leave-active {
  transition: all 0.2s ease-out;
}
.list-enter-from {
  opacity: 0;
  transform: translateY(-8px);
}
.list-leave-to {
  opacity: 0;
  transform: translateX(8px);
}
.list-move {
  transition: transform 0.2s ease-out;
}
</style>
