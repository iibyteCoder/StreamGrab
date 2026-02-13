<script setup lang="ts">
/**
 * HistoryView - 下载历史页面
 * 显示已完成下载的历史记录
 */

import { onMounted, computed } from 'vue';
import { useRouter } from 'vue-router';
import { useHistoryStore } from '@/stores';
import { useToast } from '@/composables/useToast';
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

const router = useRouter();
const historyStore = useHistoryStore();
const toast = useToast();

const records = computed(() => historyStore.records);
const isLoading = computed(() => historyStore.isLoading);
const hasRecords = computed(() => historyStore.hasRecords);

onMounted(() => {
  historyStore.loadHistory();
});

const goBack = () => {
  router.push('/');
};

const handleDelete = async (id: string) => {
  await historyStore.deleteRecord(id);
  toast.success('已删除记录');
};

const handleClearAll = async () => {
  await historyStore.clearHistory();
  toast.success('已清除所有历史记录');
};

const handleDownloadAgain = (record: { url: string }) => {
  router.push({ path: '/', query: { url: record.url } });
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
        <div v-else class="space-y-3">
          <div
            v-for="record in records"
            :key="record.id"
            class="group flex items-center gap-4 rounded-lg border bg-card p-4 transition-colors hover:bg-accent/50"
          >
            <!-- Icon -->
            <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-primary/10">
              <AppIcon name="FileVideo" :size="20" class="text-primary" />
            </div>

            <!-- Info -->
            <div class="min-w-0 flex-1">
              <div class="flex items-center gap-2">
                <h3 class="truncate font-medium">{{ record.file_name }}</h3>
              </div>
              <div class="mt-1 flex items-center gap-4 text-sm text-muted-foreground">
                <span>{{ formatBytes(record.file_size) }}</span>
                <span v-if="record.duration > 0">{{ formatDuration(record.duration) }}</span>
                <span>{{ formatDate(record.completed_at) }}</span>
              </div>
              <div class="mt-1 truncate text-xs text-muted-foreground">
                {{ record.url }}
              </div>
            </div>

            <!-- Actions -->
            <div class="flex shrink-0 items-center gap-2 opacity-0 transition-opacity group-hover:opacity-100">
              <Button variant="outline" size="sm" @click="handleDownloadAgain(record)">
                <AppIcon name="Download" :size="16" class="mr-2" />
                重新下载
              </Button>
              <Button variant="ghost" size="icon" @click="handleDelete(record.id)">
                <AppIcon name="Trash2" :size="16" />
              </Button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
