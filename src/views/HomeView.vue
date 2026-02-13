<script setup lang="ts">
/**
 * HomeView - 主页面
 * 包含 URL 输入和任务列表
 * 统一的多行输入框，支持单链接和多链接
 */

import { ref, computed, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { open } from '@tauri-apps/plugin-dialog';
import { readTextFile } from '@tauri-apps/plugin-fs';
import { Download, Settings, Play, Trash2, RotateCcw, FileUp, History } from 'lucide-vue-next';
import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';
import { Progress } from '@/components/ui/progress';
import { Card, CardContent } from '@/components/ui/card';
import { ScrollArea } from '@/components/ui/scroll-area';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { useTasks, useDownloader, useToast, useSettings } from '@/composables';
import { TASK_STATUS_TEXT } from '@/utils/constants';
import type { TaskStatus } from '@/types';

const router = useRouter();
const taskStore = useTasks();
const { addTask, hasTasks, stats, tasks, removeTask, clearCompleted } = taskStore;
const { startDownload, startPendingTasks, checkDownloaderAvailable, getDownloaderVersion, retryDownload } = useDownloader();
const toast = useToast();
const { settings, loadSettings } = useSettings();

// URL 输入（多行，支持批量）
const urlInput = ref('');
// 提交状态
const isSubmitting = ref(false);
// 下载器版本
const downloaderVersion = ref('');

// 计算有效的 URL 数量
const urlCount = computed(() => {
  if (!urlInput.value.trim()) return 0;
  return parseUrls(urlInput.value).length;
});

// 初始化
onMounted(async () => {
  await loadSettings();

  const available = await checkDownloaderAvailable();
  if (available) {
    downloaderVersion.value = await getDownloaderVersion();
  } else {
    toast.warning('N_m3u8DL-RE 未找到，请确保已安装并添加到 PATH');
  }
});

/**
 * 从文本中解析 URL 列表
 */
const parseUrls = (text: string): string[] => {
  return text
    .split('\n')
    .map(line => line.trim())
    .filter(line => line.length > 0 && (line.startsWith('http://') || line.startsWith('https://')));
};

/**
 * 添加任务并开始下载（支持单个和批量）
 */
const handleDownload = async () => {
  const urls = parseUrls(urlInput.value);

  if (urls.length === 0) {
    toast.warning('请输入有效的下载链接');
    return;
  }

  if (isSubmitting.value) return;
  isSubmitting.value = true;

  let successCount = 0;
  let failCount = 0;

  try {
    for (const url of urls) {
      try {
        const task = addTask(url, undefined, settings.value.general.saveDir);
        successCount++;

        if (settings.value.general.autoStartDownload) {
          // 异步启动下载，不等待
          startDownload(task);
        }
      } catch {
        failCount++;
      }
    }

    // 清空输入
    urlInput.value = '';

    // 触发队列处理
    if (settings.value.general.autoStartDownload) {
      await startPendingTasks();
    }

    // 显示结果
    if (successCount > 0) {
      toast.success(successCount > 1 ? `已添加 ${successCount} 个任务` : '已添加任务');
    }
    if (failCount > 0) {
      toast.warning(`${failCount} 个任务添加失败`);
    }
  } catch (error) {
    toast.error(`添加任务失败: ${error instanceof Error ? error.message : '未知错误'}`);
  } finally {
    isSubmitting.value = false;
  }
};

// 打开设置页面
const openSettings = () => {
  router.push('/settings');
};

// 开始所有等待中的任务
const handleStartAll = async () => {
  await startPendingTasks();
};

/**
 * 从文件导入 URL
 */
const handleImportFile = async () => {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: '文本文件', extensions: ['txt', 'text'] }],
      title: '选择 URL 列表文件',
    });

    if (!selected) return;

    // open with multiple: false returns string | null
    const filePath = selected;
    if (!filePath) return;

    const content = await readTextFile(filePath);
    if (!content.trim()) {
      toast.warning('文件内容为空');
      return;
    }

    // 解析 URL
    const urls = parseUrls(content);
    if (urls.length === 0) {
      toast.warning('文件中未找到有效的链接');
      return;
    }

    // 追加到输入框
    if (urlInput.value.trim()) {
      urlInput.value += '\n';
    }
    urlInput.value += urls.join('\n');

    toast.success(`已导入 ${urls.length} 个链接`);
  } catch (error) {
    console.error('Import file error:', error);
    toast.error(`导入失败: ${error instanceof Error ? error.message : '未知错误'}`);
  }
};

// 格式化速度
const formatSpeed = (bytesPerSecond: number): string => {
  if (bytesPerSecond === 0) return '0 B/s';
  const k = 1024;
  const sizes = ['B/s', 'KB/s', 'MB/s', 'GB/s'];
  const i = Math.floor(Math.log(bytesPerSecond) / Math.log(k));
  return `${parseFloat((bytesPerSecond / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
};

// 格式化大小
const formatSize = (bytes: number): string => {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
};

// 格式化时间
const formatEta = (seconds: number): string => {
  if (seconds <= 0) return '';
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = Math.floor(seconds % 60);
  if (h > 0) return `${h}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
  return `${m}:${s.toString().padStart(2, '0')}`;
};

// 获取状态颜色类
const getStatusColorClass = (status: TaskStatus): string => {
  const colors: Record<TaskStatus, string> = {
    pending: 'text-muted-foreground',
    analyzing: 'text-blue-400',
    downloading: 'text-blue-500',
    paused: 'text-yellow-500',
    merging: 'text-purple-400',
    muxing: 'text-purple-500',
    completed: 'text-green-500',
    failed: 'text-destructive',
    cancelled: 'text-muted-foreground',
  };
  return colors[status] || 'text-muted-foreground';
};
</script>

<template>
  <div class="flex h-full flex-col bg-background">
    <!-- 头部区域 -->
    <header class="border-b p-4">
      <!-- 标题栏 -->
      <div class="flex items-center justify-between mb-4">
        <div class="flex items-center gap-3">
          <div class="flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10">
            <Download class="h-5 w-5 text-primary" />
          </div>
          <div>
            <h1 class="text-xl font-semibold">StreamGrab</h1>
            <p class="text-xs text-muted-foreground">M3U8 视频流下载器</p>
          </div>
        </div>
        <div class="flex items-center gap-1">
          <Button variant="ghost" size="icon" @click="router.push('/history')">
            <History class="h-5 w-5" />
          </Button>
          <Button variant="ghost" size="icon" @click="openSettings">
            <Settings class="h-5 w-5" />
          </Button>
        </div>
      </div>

      <!-- URL 输入区 - 统一的多行输入 -->
      <div class="space-y-2">
        <Textarea
          v-model="urlInput"
          placeholder="输入下载链接，每行一个&#10;例如:&#10;https://example.com/video1.m3u8&#10;https://example.com/video2.m3u8"
          class="min-h-[80px] resize-none"
          @keydown.ctrl.enter="handleDownload"
        />
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <span class="text-xs text-muted-foreground">
              <template v-if="urlCount > 0">已输入 {{ urlCount }} 个链接</template>
              <template v-else>Ctrl + Enter 快速添加</template>
            </span>
            <Button variant="ghost" size="sm" class="h-6 px-2 text-xs" @click="handleImportFile">
              <FileUp class="mr-1 h-3 w-3" />
              导入
            </Button>
          </div>
          <Button
            :loading="isSubmitting"
            :disabled="urlCount === 0"
            @click="handleDownload"
          >
            <Download class="mr-2 h-4 w-4" />
            {{ urlCount > 1 ? `下载 (${urlCount})` : '下载' }}
          </Button>
        </div>
      </div>

      <!-- 统计信息 -->
      <div v-if="hasTasks" class="flex items-center gap-4 mt-3 text-sm text-muted-foreground">
        <span>总计: {{ stats.total }}</span>
        <span>已完成: {{ stats.completed }}</span>
        <span>进度: {{ stats.percent }}%</span>
      </div>
    </header>

    <!-- 任务列表区域 -->
    <ScrollArea class="flex-1">
      <div class="p-4 space-y-3">
        <!-- 任务卡片 -->
        <Card v-for="task in tasks" :key="task.id" class="overflow-hidden">
          <CardContent class="p-4">
            <!-- 任务头部 -->
            <div class="flex items-start justify-between mb-2">
              <div class="flex-1 min-w-0 mr-3">
                <h3 class="font-medium truncate">{{ task.fileName }}</h3>
                <p class="text-xs text-muted-foreground truncate">{{ task.url }}</p>
              </div>
              <div class="flex items-center gap-1">
                <span :class="['text-xs font-medium', getStatusColorClass(task.status)]">
                  {{ TASK_STATUS_TEXT[task.status] }}
                </span>
                <!-- 更多操作 -->
                <DropdownMenu>
                  <DropdownMenuTrigger as-child>
                    <Button variant="ghost" size="icon" class="h-8 w-8">
                      <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 5v.01M12 12v.01M12 19v.01M12 6a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2z" />
                      </svg>
                    </Button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent align="end">
                    <DropdownMenuItem v-if="task.status === 'failed'" @click="retryDownload(task)">
                      <RotateCcw class="mr-2 h-4 w-4" />
                      重试
                    </DropdownMenuItem>
                    <DropdownMenuSeparator />
                    <DropdownMenuItem class="text-destructive" @click="removeTask(task.id)">
                      <Trash2 class="mr-2 h-4 w-4" />
                      删除
                    </DropdownMenuItem>
                  </DropdownMenuContent>
                </DropdownMenu>
              </div>
            </div>

            <!-- 进度条 -->
            <div v-if="task.status === 'downloading'" class="space-y-2">
              <Progress :value="task.progress.percent" class="h-2" />
              <div class="flex items-center justify-between text-xs text-muted-foreground">
                <span>{{ task.progress.percent.toFixed(1) }}%</span>
                <span>{{ formatSpeed(task.progress.speed) }}</span>
                <span>{{ formatSize(task.progress.downloadedSize) }} / {{ formatSize(task.progress.totalSize) || '未知' }}</span>
                <span v-if="task.progress.eta > 0">剩余: {{ formatEta(task.progress.eta) }}</span>
              </div>
            </div>

            <!-- 错误信息 -->
            <p v-if="task.error" class="mt-2 text-xs text-destructive">
              {{ task.error }}
            </p>
          </CardContent>
        </Card>

        <!-- 空状态 -->
        <div v-if="!hasTasks" class="flex flex-col items-center justify-center py-16 text-center">
          <div class="flex h-16 w-16 items-center justify-center rounded-full bg-muted mb-4">
            <Download class="h-8 w-8 text-muted-foreground" />
          </div>
          <p class="text-muted-foreground">暂无下载任务</p>
          <p class="text-sm text-muted-foreground mt-1">输入链接开始下载</p>
        </div>
      </div>
    </ScrollArea>

    <!-- 底部工具栏 -->
    <footer v-if="hasTasks" class="border-t p-3 flex items-center justify-between">
      <div class="flex items-center gap-2">
        <Button variant="outline" size="sm" @click="handleStartAll">
          <Play class="mr-2 h-4 w-4" />
          开始全部
        </Button>
        <Button variant="outline" size="sm" @click="clearCompleted">
          <Trash2 class="mr-2 h-4 w-4" />
          清除已完成
        </Button>
      </div>
      <span class="text-xs text-muted-foreground">
        {{ downloaderVersion || 'N_m3u8DL-RE' }}
      </span>
    </footer>
  </div>
</template>
