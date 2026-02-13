<script setup lang="ts">
import { onMounted, computed } from 'vue';
import { useRouter } from 'vue-router';
import { useHistoryStore } from '@/stores';
import { useToast } from '@/composables/useToast';
import { formatBytes, formatDate } from '@/utils/format';
import AppButton from '@/components/common/AppButton.vue';
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
  <div class="flex h-full flex-col">
    <!-- Header -->
    <header class="flex h-14 shrink-0 items-center justify-between border-b border-border px-6">
      <div class="flex items-center gap-4">
        <AppButton variant="ghost" size="sm" @click="goBack">
          <span class="i-carbon-arrow-left mr-1"></span>
          返回
        </AppButton>
        <h1 class="text-lg font-semibold">下载历史</h1>
      </div>

      <AlertDialog v-if="hasRecords">
        <AlertDialogTrigger as-child>
          <AppButton variant="destructive" size="sm">
            清除全部
          </AppButton>
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
    </header>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto p-6">
      <!-- Loading -->
      <div v-if="isLoading" class="flex items-center justify-center py-12">
        <span class="i-carbon-renew animate-spin text-2xl text-muted-foreground"></span>
      </div>

      <!-- Empty State -->
      <div
        v-else-if="!hasRecords"
        class="flex flex-col items-center justify-center py-12 text-muted-foreground"
      >
        <span class="i-carbon-document mb-4 text-4xl"></span>
        <p>暂无下载历史</p>
      </div>

      <!-- History List -->
      <div v-else class="space-y-3">
        <div
          v-for="record in records"
          :key="record.id"
          class="group flex items-center gap-4 rounded-lg border border-border bg-card p-4 transition-colors hover:bg-accent/50"
        >
          <!-- Icon -->
          <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-primary/10">
            <span class="i-carbon-video text-xl text-primary"></span>
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
            <AppButton variant="outline" size="sm" @click="handleDownloadAgain(record)">
              <span class="i-carbon-download mr-1"></span>
              重新下载
            </AppButton>
            <AppButton variant="ghost" size="sm" @click="handleDelete(record.id)">
              <span class="i-carbon-trash-can"></span>
            </AppButton>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
