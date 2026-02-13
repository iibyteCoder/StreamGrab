<script setup lang="ts">
/**
 * HomeView - 主页面
 * 包含 URL 输入和任务列表
 */

import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { Button } from '@/components/ui/button';
import { AppIcon } from '@/components/common';
import { UrlInputPanel } from '@/components/input';
import { TaskList } from '@/components/task';
import { useTasks, useDownloader, useToast, useSettings } from '@/composables';

const router = useRouter();
const taskStore = useTasks();
const { addTask, hasTasks, stats, clearCompleted } = taskStore;
const { startDownload, startPendingTasks, checkDownloaderAvailable, getDownloaderVersion } = useDownloader();
const toast = useToast();
const { settings, loadSettings } = useSettings();

// URL 输入（多行，支持批量）
const urlInput = ref('');
// 提交状态
const isSubmitting = ref(false);
// 下载器版本
const downloaderVersion = ref('');

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
</script>

<template>
  <div class="flex h-full flex-col bg-background">
    <!-- 头部区域 -->
    <header class="border-b p-4 shrink-0">
      <!-- 标题栏 -->
      <div class="flex items-center justify-between mb-4">
        <div class="flex items-center gap-3">
          <div class="flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10">
            <AppIcon name="Download" :size="20" class="text-primary" />
          </div>
          <div>
            <h1 class="text-xl font-semibold">StreamGrab</h1>
            <p class="text-xs text-muted-foreground">M3U8 视频流下载器</p>
          </div>
        </div>
        <div class="flex items-center gap-1">
          <Button variant="ghost" size="icon" @click="router.push('/history')">
            <AppIcon name="History" :size="20" />
          </Button>
          <Button variant="ghost" size="icon" @click="openSettings">
            <AppIcon name="Settings" :size="20" />
          </Button>
        </div>
      </div>

      <!-- URL 输入区 - 统一的多行输入 -->
      <UrlInputPanel
        v-model="urlInput"
        :is-submitting="isSubmitting"
        @download="handleDownload"
      />

      <!-- 统计信息 -->
      <div v-if="hasTasks" class="flex items-center gap-4 mt-3 text-sm text-muted-foreground">
        <span>总计: {{ stats.total }}</span>
        <span>已完成: {{ stats.completed }}</span>
        <span>进度: {{ stats.percent }}%</span>
      </div>
    </header>

    <!-- 任务列表区域 -->
    <div class="flex-1 min-h-0 overflow-y-auto">
      <div class="p-4">
        <TaskList />
      </div>
    </div>

    <!-- 底部工具栏 -->
    <footer v-if="hasTasks" class="border-t p-3 flex items-center justify-between shrink-0">
      <div class="flex items-center gap-2">
        <Button variant="outline" size="sm" @click="handleStartAll">
          <AppIcon name="Play" :size="16" class="mr-2" />
          开始全部
        </Button>
        <Button variant="outline" size="sm" @click="clearCompleted">
          <AppIcon name="Trash2" :size="16" class="mr-2" />
          清除已完成
        </Button>
      </div>
      <span class="text-xs text-muted-foreground">
        {{ downloaderVersion || 'N_m3u8DL-RE' }}
      </span>
    </footer>
  </div>
</template>
