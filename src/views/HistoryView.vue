<script setup lang="ts">
/**
 * HistoryView - 下载历史页面
 * 显示已完成下载的历史记录
 */

import { onMounted, computed, ref, watch } from 'vue';
import { useRouter } from 'vue-router';
import { useHistoryStore } from '@/stores';
import { useToast } from '@/composables/useToast';
import { configService } from '@/services';
import { formatBytes, formatDate } from '@/utils/format';
import { Button } from '@/components/ui/button';
import { AppIcon } from '@/components/common';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from '@/components/ui/alert-dialog';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import type { HistoryRecord } from '@/types';

const router = useRouter();
const historyStore = useHistoryStore();
const toast = useToast();

const records = computed(() => historyStore.records);
const isLoading = computed(() => historyStore.isLoading);
const hasRecords = computed(() => historyStore.hasRecords);

// 文件存在状态映射
const fileExistsMap = ref<Map<string, boolean>>(new Map());

// 删除确认对话框
const showDeleteDialog = ref(false);
const deleteWithFile = ref(false);
const recordToDelete = ref<HistoryRecord | null>(null);
const isDeleting = ref(false);

onMounted(async () => {
  await historyStore.loadHistory();
  // 检查所有文件是否存在
  checkAllFilesExist();
});

// 监听记录变化，检查文件
watch(records, () => {
  checkAllFilesExist();
}, { deep: true });

// 检查所有文件是否存在
const checkAllFilesExist = async () => {
  for (const record of records.value) {
    if (record.save_path && !fileExistsMap.value.has(record.id)) {
      try {
        const exists = await configService.fileExists(record.save_path);
        fileExistsMap.value.set(record.id, exists);
      } catch {
        fileExistsMap.value.set(record.id, false);
      }
    }
  }
};

// 检查单个文件是否存在
const fileExists = (id: string): boolean | undefined => {
  return fileExistsMap.value.get(id);
};

const goBack = () => {
  router.push('/');
};

// 打开删除确认对话框
const handleDeleteClick = (record: HistoryRecord) => {
  const exists = fileExistsMap.value.get(record.id);
  if (exists) {
    recordToDelete.value = record;
    deleteWithFile.value = false;
    showDeleteDialog.value = true;
  } else {
    // 文件不存在，直接删除记录
    performDelete(record.id, false);
  }
};

// 执行删除
const performDelete = async (id: string, withFile: boolean) => {
  isDeleting.value = true;
  try {
    if (withFile && recordToDelete.value?.save_path) {
      try {
        await configService.deleteFileOrFolder(recordToDelete.value.save_path);
      } catch (error) {
        console.error('Failed to delete file:', error);
        toast.error('文件删除失败，但记录已删除');
      }
    }
    await historyStore.deleteRecord(id);
    toast.success('已删除记录');
  } finally {
    isDeleting.value = false;
    showDeleteDialog.value = false;
    recordToDelete.value = null;
  }
};

// 确认删除
const handleConfirmDelete = () => {
  if (recordToDelete.value) {
    performDelete(recordToDelete.value.id, deleteWithFile.value);
  }
};

const handleClearAll = async () => {
  await historyStore.clearHistory();
  fileExistsMap.value.clear();
  toast.success('已清除所有历史记录');
};

const handleDownloadAgain = (record: { url: string }) => {
  router.push({ path: '/', query: { url: record.url } });
};

/**
 * 打开文件
 */
const handleOpenFile = async (path: string) => {
  try {
    await configService.openInExplorer(path);
  } catch (error) {
    toast.error('打开文件失败');
    console.error('Failed to open file:', error);
  }
};

/**
 * 打开文件所在目录
 */
const handleOpenFolder = async (path: string) => {
  try {
    // 获取目录路径
    const lastSepIndex = Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\'));
    const dirPath = lastSepIndex > 0 ? path.substring(0, lastSepIndex) : path;
    await configService.openInExplorer(dirPath);
  } catch (error) {
    toast.error('打开目录失败');
    console.error('Failed to open folder:', error);
  }
};

const formatDuration = (seconds: number): string => {
  if (!seconds || seconds <= 0) return '-';
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = Math.floor(seconds % 60);
  if (h > 0) {
    return `${h}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
  }
  return `${m}:${s.toString().padStart(2, '0')}`;
};
</script>

<template>
  <div class="flex h-full flex-col bg-background">
    <!-- 头部区域 -->
    <header class="border-b p-4 shrink-0">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-3">
          <Button variant="ghost" size="icon" @click="goBack">
            <AppIcon name="ArrowLeft" :size="20" />
          </Button>
          <div>
            <h1 class="text-xl font-semibold">下载历史</h1>
            <p class="text-xs text-muted-foreground">查看已完成的下载记录</p>
          </div>
        </div>
        <AlertDialog v-if="hasRecords">
          <AlertDialogTrigger as-child>
            <Button variant="destructive" size="sm">
              <AppIcon name="Trash2" :size="16" class="mr-2" />
              清除全部
            </Button>
          </AlertDialogTrigger>
          <AlertDialogContent>
            <AlertDialogHeader>
              <AlertDialogTitle>确认清除</AlertDialogTitle>
              <AlertDialogDescription>
                此操作将清除所有下载历史记录，无法撤销。
              </AlertDialogDescription>
            </AlertDialogHeader>
            <AlertDialogFooter>
              <AlertDialogCancel>取消</AlertDialogCancel>
              <AlertDialogAction @click="handleClearAll">确认清除</AlertDialogAction>
            </AlertDialogFooter>
          </AlertDialogContent>
        </AlertDialog>
      </div>
    </header>

    <!-- 历史记录列表区域 -->
    <div class="flex-1 min-h-0 overflow-y-auto">
      <div class="p-4">
        <!-- Loading -->
        <div v-if="isLoading" class="flex items-center justify-center py-12">
          <AppIcon name="RefreshCw" :size="24" class="animate-spin text-muted-foreground" />
        </div>

        <!-- Empty State -->
        <div
          v-else-if="!hasRecords"
          class="flex flex-col items-center justify-center py-12 text-muted-foreground"
        >
          <AppIcon name="File" :size="48" class="mb-4 opacity-50" />
          <p>暂无下载历史</p>
        </div>

        <!-- History List -->
        <div v-else class="space-y-2">
          <div
            v-for="record in records"
            :key="record.id"
            class="group flex items-center gap-3 rounded-lg border bg-card p-3 transition-colors hover:bg-accent/50"
          >
            <!-- Icon -->
            <div class="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg bg-primary/10">
              <AppIcon name="FileVideo" :size="18" class="text-primary" />
            </div>

            <!-- Info -->
            <div class="min-w-0 flex-1">
              <div class="flex items-center gap-2">
                <h3 class="truncate font-medium text-sm">{{ record.file_name }}</h3>
                <!-- 文件丢失提示 -->
                <span
                  v-if="record.save_path && fileExists(record.id) === false"
                  class="text-xs text-amber-500 flex items-center gap-0.5"
                  title="文件已被移动或删除"
                >
                  <AppIcon name="AlertTriangle" :size="12" />
                  文件已移除
                </span>
              </div>
              <div class="mt-0.5 flex items-center gap-3 text-xs text-muted-foreground">
                <span>{{ formatBytes(record.file_size) }}</span>
                <span v-if="record.duration > 0">{{ formatDuration(record.duration) }}</span>
                <span>{{ formatDate(record.completed_at) }}</span>
              </div>
            </div>

            <!-- Actions -->
            <div class="flex shrink-0 items-center gap-1 opacity-0 transition-opacity group-hover:opacity-100">
              <!-- 播放文件（文件存在时显示） -->
              <Button
                v-if="record.save_path && fileExists(record.id)"
                variant="ghost"
                size="icon"
                title="播放"
                @click="handleOpenFile(record.save_path)"
              >
                <AppIcon name="Play" :size="16" />
              </Button>
              <!-- 打开文件夹 -->
              <Button
                v-if="record.save_path"
                variant="ghost"
                size="icon"
                title="打开目录"
                @click="handleOpenFolder(record.save_path)"
              >
                <AppIcon name="FolderOpen" :size="16" />
              </Button>
              <!-- 重新下载 -->
              <Button variant="outline" size="sm" @click="handleDownloadAgain(record)">
                <AppIcon name="Download" :size="14" class="mr-1" />
                重新下载
              </Button>
              <!-- 删除 -->
              <Button variant="ghost" size="icon" @click="handleDeleteClick(record)">
                <AppIcon name="Trash2" :size="16" />
              </Button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 删除确认对话框 -->
    <Dialog v-model:open="showDeleteDialog">
      <DialogContent class="sm:max-w-[400px]">
        <DialogHeader>
          <DialogTitle>确认删除</DialogTitle>
          <DialogDescription>
            确定要删除此记录吗？
          </DialogDescription>
        </DialogHeader>

        <div class="py-4">
          <label class="flex items-center gap-2 cursor-pointer select-none">
            <input
              type="checkbox"
              v-model="deleteWithFile"
              class="w-4 h-4 rounded border-border-default accent-primary"
            />
            <span class="text-sm">同时删除下载的文件</span>
          </label>
          <p v-if="deleteWithFile && recordToDelete?.save_path" class="mt-2 text-xs text-muted-foreground truncate">
            {{ recordToDelete.save_path }}
          </p>
        </div>

        <DialogFooter>
          <Button variant="outline" @click="showDeleteDialog = false">取消</Button>
          <Button
            variant="destructive"
            :disabled="isDeleting"
            @click="handleConfirmDelete"
          >
            {{ isDeleting ? '删除中...' : '确认删除' }}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </div>
</template>
