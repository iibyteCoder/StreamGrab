<script setup lang="ts">
/**
 * HomeView - 主页面
 * 包含 URL 输入和任务列表
 */

import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { UrlInput } from '@/components/input';
import { TaskList } from '@/components/task';
import { AppButton } from '@/components/common';
import { useTasks, useDownloader, useToast, useSettings } from '@/composables';
import type { DownloadTask } from '@/types';

const router = useRouter();
const { addTask, hasTasks, stats } = useTasks();
const { startDownload, startPendingTasks, checkDownloaderAvailable, getDownloaderVersion } = useDownloader();
const toast = useToast();
const { settings, loadSettings, initTheme } = useSettings();

const isSubmitting = ref(false);
const downloaderVersion = ref('');

// 初始化
onMounted(async () => {
  // 加载设置
  await loadSettings();
  initTheme();

  // 检查下载器是否可用
  const available = await checkDownloaderAvailable();
  if (available) {
    downloaderVersion.value = await getDownloaderVersion();
    console.log('Downloader version:', downloaderVersion.value);
  } else {
    toast.warning('N_m3u8DL-RE 未找到，请确保已安装并添加到 PATH');
  }
});

// 处理 URL 提交
const handleSubmit = async (url: string) => {
  if (isSubmitting.value) return;
  isSubmitting.value = true;

  try {
    // 添加任务
    const task = addTask(url, undefined, settings.value.general.saveDir);

    // 如果设置了自动开始下载
    if (settings.value.general.autoStartDownload) {
      await startDownload(task);
    }

    toast.success(`已添加任务: ${task.fileName}`);
  } catch (error) {
    console.error('Failed to add task:', error);
    toast.error(`添加任务失败: ${error instanceof Error ? error.message : '未知错误'}`);
  } finally {
    isSubmitting.value = false;
  }
};

// 处理 URL 解析（可选，用于流选择器）
const handleParse = async (url: string) => {
  // TODO: 实现流选择器
  console.log('Parse URL:', url);
};

// 处理任务点击
const handleTaskClick = (task: DownloadTask) => {
  // TODO: 显示任务详情
  console.log('Task clicked:', task);
};

// 打开设置页面
const openSettings = () => {
  router.push('/settings');
};

// 开始所有等待中的任务
const startAllPending = async () => {
  await startPendingTasks();
};
</script>

<template>
  <div class="flex h-full flex-col">
    <!-- 头部区域 -->
    <div class="border-b border-border p-6">
      <!-- 标题栏 -->
      <div class="flex items-center justify-between mb-4">
        <h1 class="text-2xl font-semibold text-text-primary">StreamGrab</h1>
        <div class="flex items-center gap-2">
          <!-- 设置按钮 -->
          <button
            class="p-2 rounded-lg hover:bg-bg-elevated text-text-secondary hover:text-text-primary transition-colors"
            title="设置"
            @click="openSettings"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
          </button>
        </div>
      </div>

      <!-- URL 输入区 -->
      <UrlInput
        :loading="isSubmitting"
        placeholder="输入 M3U8 / MPD / MSS 链接..."
        @submit="handleSubmit"
        @parse="handleParse"
      />

      <!-- 统计信息 -->
      <div v-if="hasTasks" class="flex items-center gap-4 mt-4 text-sm text-text-secondary">
        <span>总任务: {{ stats.total }}</span>
        <span>已完成: {{ stats.completed }}</span>
        <span>进度: {{ stats.percent }}%</span>
      </div>
    </div>

    <!-- 任务列表区域 -->
    <div class="flex-1 overflow-auto p-6">
      <TaskList
        @task-click="handleTaskClick"
      />
    </div>

    <!-- 底部工具栏 -->
    <div v-if="hasTasks" class="border-t border-border p-4 flex items-center justify-between">
      <div class="flex items-center gap-2">
        <AppButton
          variant="secondary"
          size="sm"
          @click="startAllPending"
        >
          开始全部
        </AppButton>
      </div>
      <div class="text-sm text-text-muted">
        {{ downloaderVersion || 'N_m3u8DL-RE' }}
      </div>
    </div>
  </div>
</template>
