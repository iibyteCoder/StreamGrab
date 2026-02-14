<script setup lang="ts">
/**
 * HomeView - 主页面
 * 包含 URL 输入和任务列表
 */

import { ref, onMounted, onUnmounted } from 'vue';
import { useRouter } from 'vue-router';
import { Button } from '@/components/ui/button';
import { AppIcon } from '@/components/common';
import { UrlInputPanel } from '@/components/input';
import { TaskList } from '@/components/task';
import { StreamSelector } from '@/components/stream';
import { useTasks, useDownloader, useToast, useSettings, useClipboardWatcher } from '@/composables';
import type { StreamInfo, StreamSelection } from '@/types';

const router = useRouter();
const taskStore = useTasks();
const { addTask, hasTasks, stats, clearCompleted } = taskStore;
const { startDownload, startPendingTasks, checkDownloaderAvailable, getDownloaderVersion, parseUrl, isParsing } = useDownloader();
const toast = useToast();
const { settings, loadSettings } = useSettings();

// 剪贴板监控
useClipboardWatcher();

// URL 输入（多行，支持批量）
const urlInput = ref('');
// 提交状态
const isSubmitting = ref(false);
// 下载器版本
const downloaderVersion = ref('');

// 拖拽状态
const isDragging = ref(false);

// 流选择器状态
const showStreamSelector = ref(false);
const currentStreamInfo = ref<StreamInfo | null>(null);
const pendingUrl = ref<string | null>(null);

// 初始化
onMounted(async () => {
  await loadSettings();

  const available = await checkDownloaderAvailable();
  if (available) {
    downloaderVersion.value = await getDownloaderVersion();
  } else {
    toast.warning('N_m3u8DL-RE 未找到，请确保已安装并添加到 PATH');
  }

  // 监听剪贴板检测到的 URL
  const handleClipboardUrls = ((event: CustomEvent<{ urls: string[] }>) => {
    const urls = event.detail.urls;
    if (urls.length > 0) {
      // 添加到输入框
      const currentUrls = urlInput.value.trim();
      urlInput.value = currentUrls ? `${currentUrls}\n${urls.join('\n')}` : urls.join('\n');
    }
  }) as EventListener;

  window.addEventListener('clipboard-urls-detected', handleClipboardUrls);

  // 清理函数存储
  cleanupClipboardListener = () => {
    window.removeEventListener('clipboard-urls-detected', handleClipboardUrls);
  };
});

// 清理函数
let cleanupClipboardListener: (() => void) | null = null;

onUnmounted(() => {
  if (cleanupClipboardListener) {
    cleanupClipboardListener();
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
 * 处理拖拽进入
 */
const handleDragOver = (event: DragEvent) => {
  event.preventDefault();
  isDragging.value = true;
};

/**
 * 处理拖拽离开
 */
const handleDragLeave = (event: DragEvent) => {
  event.preventDefault();
  // 检查是否真正离开了容器
  const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
  const x = event.clientX;
  const y = event.clientY;
  if (x < rect.left || x > rect.right || y < rect.top || y > rect.bottom) {
    isDragging.value = false;
  }
};

/**
 * 处理拖放
 */
const handleDrop = async (event: DragEvent) => {
  event.preventDefault();
  isDragging.value = false;

  const dataTransfer = event.dataTransfer;
  if (!dataTransfer) return;

  // 收集所有 URL
  const urls: string[] = [];

  // 1. 从文本数据中提取 URL
  const text = dataTransfer.getData('text/plain');
  if (text) {
    urls.push(...parseUrls(text));
  }

  // 2. 从文件中提取 URL
  const files = dataTransfer.files;
  if (files.length > 0) {
    for (const file of files) {
      if (file.type === 'text/plain' || file.name.endsWith('.txt')) {
        try {
          const content = await file.text();
          urls.push(...parseUrls(content));
        } catch (e) {
          console.error('Failed to read file:', e);
        }
      }
    }
  }

  if (urls.length > 0) {
    // 添加到输入框
    const currentUrls = urlInput.value.trim();
    urlInput.value = currentUrls ? `${currentUrls}\n${urls.join('\n')}` : urls.join('\n');
    toast.success(`已添加 ${urls.length} 个链接`);
  } else {
    toast.warning('未找到有效的下载链接');
  }
};

/**
 * 添加任务并开始下载（支持单个和批量）
 */
const handleDownload = async (options?: { startAt?: Date }) => {
  const urls = parseUrls(urlInput.value);

  if (urls.length === 0) {
    toast.warning('请输入有效的下载链接');
    return;
  }

  if (isSubmitting.value) return;
  isSubmitting.value = true;

  // 保存定时开始时间用于流选择器确认时使用
  const scheduledStartAt = options?.startAt;

  try {
    // 如果只有一个 URL 且启用了流选择，先解析
    if (urls.length === 1 && !settings.value.download.autoSelect) {
      const url = urls[0]!;
      pendingUrl.value = url;
      pendingStartAt.value = scheduledStartAt || null;
      const info = await parseUrl(url);

      if (info) {
        currentStreamInfo.value = info;
        showStreamSelector.value = true;
      } else {
        // 解析失败，直接添加任务
        addTaskAndDownload(url, undefined, scheduledStartAt);
      }
    } else {
      // 批量下载或自动选择模式，直接添加任务
      let successCount = 0;
      let failCount = 0;

      for (const url of urls) {
        try {
          addTaskAndDownload(url, undefined, scheduledStartAt);
          successCount++;
        } catch {
          failCount++;
        }
      }

      // 清空输入
      urlInput.value = '';

      // 触发队列处理
      if (settings.value.general.autoStartDownload && !scheduledStartAt) {
        await startPendingTasks();
      }

      // 显示结果
      if (successCount > 0) {
        const msg = scheduledStartAt
          ? `已添加 ${successCount} 个定时任务`
          : (successCount > 1 ? `已添加 ${successCount} 个任务` : '已添加任务');
        toast.success(msg);
      }
      if (failCount > 0) {
        toast.warning(`${failCount} 个任务添加失败`);
      }
    }
  } catch (error) {
    toast.error(`添加任务失败: ${error instanceof Error ? error.message : '未知错误'}`);
  } finally {
    isSubmitting.value = false;
  }
};

// 待处理的定时开始时间
const pendingStartAt = ref<Date | null>(null);

/**
 * 添加任务并开始下载
 */
const addTaskAndDownload = (url: string, selection?: StreamSelection, startAt?: Date | null) => {
  const task = addTask(url, undefined, settings.value.general.saveDir);

  // 构建任务配置
  const config: Record<string, unknown> = {
    ...task.config,
  };

  // 如果有流选择结果，更新任务配置
  if (selection) {
    config.autoSelect = false;
    config.selectVideo = selection.videoIds[0] || '';
    config.selectAudio = selection.audioIds.join(',') || '';
    config.selectSubtitle = selection.subtitleIds.join(',') || '';
  }

  // 如果有定时开始时间，更新任务配置
  if (startAt) {
    config.startAt = startAt;
  }

  // 只在有配置时更新
  if (Object.keys(config).length > 0) {
    taskStore.updateTaskConfig(task.id, config);
  }

  // 只有非定时任务才自动开始
  if (settings.value.general.autoStartDownload && !startAt) {
    // 异步启动下载，不等待
    startDownload(taskStore.getTask(task.id)!);
  }

  return task;
};

/**
 * 流选择器确认
 */
const handleStreamConfirm = (selection: StreamSelection) => {
  if (pendingUrl.value) {
    addTaskAndDownload(pendingUrl.value, selection, pendingStartAt.value);
    urlInput.value = '';
    pendingUrl.value = null;
    pendingStartAt.value = null;

    // 触发队列处理（非定时任务）
    if (settings.value.general.autoStartDownload && !pendingStartAt.value) {
      startPendingTasks();
    }

    toast.success('已添加任务');
  }
};

/**
 * 流选择器取消
 */
const handleStreamCancel = () => {
  pendingUrl.value = null;
  pendingStartAt.value = null;
  currentStreamInfo.value = null;
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
  <div
    class="flex h-full flex-col bg-background relative"
    @dragover="handleDragOver"
    @dragleave="handleDragLeave"
    @drop="handleDrop"
  >
    <!-- 拖拽提示遮罩 -->
    <div
      v-if="isDragging"
      class="absolute inset-0 z-50 bg-primary/10 backdrop-blur-sm flex items-center justify-center pointer-events-none"
    >
      <div class="bg-background border-2 border-dashed border-primary rounded-xl p-8 text-center">
        <AppIcon name="Download" :size="48" class="text-primary mx-auto mb-4" />
        <p class="text-lg font-medium">释放以添加链接</p>
        <p class="text-sm text-muted-foreground mt-1">支持拖放文本链接或 TXT 文件</p>
      </div>
    </div>

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

    <!-- 流选择器对话框 -->
    <StreamSelector
      v-model:open="showStreamSelector"
      :stream-info="currentStreamInfo"
      :loading="isParsing"
      @confirm="handleStreamConfirm"
      @cancel="handleStreamCancel"
    />
  </div>
</template>
