<script setup lang="ts">
/**
 * HomeView - 下载任务页面
 * 子路由页面，只负责内容渲染
 */

import { ref, onMounted, nextTick, watch } from 'vue';
import { Button } from '@/components/ui/button';
import { AppIcon } from '@/components/common';
import { TaskList, TaskFilterBar, AddTaskDialog, TaskDetailPanel } from '@/components/task';
import { useTasks, useDownloader, useSettings, useClipboardWatcher, useTaskFilter } from '@/composables';
import type { DownloadTask } from '@/types';

const taskStore = useTasks();
const { tasks, clearCompleted } = taskStore;
const { startPendingTasks, checkDownloaderAvailable } = useDownloader();
const { loadSettings } = useSettings();

// 剪贴板监控
useClipboardWatcher();

// 添加任务弹窗
const showAddDialog = ref(false);

// 详情面板
const showDetailPanel = ref(false);
const selectedTaskId = ref<string | null>(null);

// Tab 状态
const activeTab = ref<'active' | 'completed'>('active');

// Tab 滑块动画
const tabRefs = ref<Record<string, HTMLElement | null>>({});
const sliderStyle = ref({ width: '0px', transform: 'translateX(0px)' });

const updateSlider = async () => {
  await nextTick();
  const activeEl = tabRefs.value[activeTab.value];
  if (activeEl) {
    const activeRect = activeEl.getBoundingClientRect();
    sliderStyle.value = {
      width: `${activeRect.width}px`,
      transform: `translateX(${activeEl.offsetLeft - 4}px)`,
    };
  }
};

watch(activeTab, () => {
  updateSlider();
  showDetailPanel.value = false;
});
onMounted(() => {
  updateSlider();
});

// 过滤器
const { search, sort, activeTasks, completedTasks, activeCount, completedCount } = useTaskFilter(tasks);

// 初始化
onMounted(async () => {
  await loadSettings();
  await checkDownloaderAvailable();
});

const handleStartAll = async () => await startPendingTasks();

// 点击任务卡片
const handleTaskClick = (task: DownloadTask) => {
  selectedTaskId.value = task.id;
  showDetailPanel.value = true;
};
</script>

<template>
  <div class="h-full flex flex-col">
    <!-- 工具栏 -->
    <div class="border-b px-6 py-3 shrink-0 bg-card/50 flex items-center justify-between">
      <!-- Tab 切换 -->
      <div class="relative flex items-center p-1 bg-muted/50 rounded-lg">
        <!-- 滑动背景块 -->
        <div
          class="absolute top-1 bottom-1 bg-background rounded-md shadow-sm transition-all duration-300 ease-out"
          :style="sliderStyle"
        />

        <!-- Tab 按钮 -->
        <button
          :ref="(el) => tabRefs.active = el as HTMLElement"
          :class="[
            'relative z-10 px-4 py-1.5 text-sm rounded-md transition-colors',
            activeTab === 'active' ? 'text-foreground' : 'text-muted-foreground hover:text-foreground'
          ]"
          @click="activeTab = 'active'"
        >
          进行中
          <span
            v-if="activeCount > 0"
            class="ml-1.5 px-1.5 py-0.5 rounded-full bg-primary/15 text-primary text-xs font-medium"
          >
            {{ activeCount }}
          </span>
        </button>
        <button
          :ref="(el) => tabRefs.completed = el as HTMLElement"
          :class="[
            'relative z-10 px-4 py-1.5 text-sm rounded-md transition-colors',
            activeTab === 'completed' ? 'text-foreground' : 'text-muted-foreground hover:text-foreground'
          ]"
          @click="activeTab = 'completed'"
        >
          已完成
          <span
            v-if="completedCount > 0"
            class="ml-1.5 px-1.5 py-0.5 rounded-full bg-green-500/15 text-green-600 text-xs font-medium"
          >
            {{ completedCount }}
          </span>
        </button>
      </div>

      <!-- 过滤器 + 添加按钮 -->
      <div class="flex items-center gap-3">
        <TaskFilterBar v-model:search="search" v-model:sort="sort" class="w-64" />
        <Button @click="showAddDialog = true">
          <AppIcon name="Plus" :size="16" class="mr-2" />
          添加任务
        </Button>
      </div>
    </div>

    <!-- 任务列表区域（包含详情面板） -->
    <div class="flex-1 min-h-0 flex overflow-hidden">
      <!-- 任务列表 -->
      <div class="flex-1 overflow-y-auto px-6 py-4">
        <TaskList
          v-if="activeTab === 'active'"
          :tasks="activeTasks"
          empty-type="active"
          @task-click="handleTaskClick"
        />
        <TaskList
          v-else
          :tasks="completedTasks"
          empty-type="completed"
          @task-click="handleTaskClick"
        />
      </div>

      <!-- 任务详情面板（在列表区域内侧滑） -->
      <TaskDetailPanel v-model:open="showDetailPanel" :task-id="selectedTaskId" />
    </div>

    <!-- 底部操作 -->
    <div
      v-if="activeCount > 0 || completedCount > 0"
      class="border-t px-6 py-3 shrink-0 flex items-center gap-3 bg-card/50"
    >
      <Button
        v-if="activeCount > 0"
        variant="outline"
        size="sm"
        @click="handleStartAll"
      >
        <AppIcon name="Play" :size="14" class="mr-2" />
        开始全部
      </Button>
      <Button
        v-if="completedCount > 0"
        variant="outline"
        size="sm"
        @click="clearCompleted"
      >
        <AppIcon name="Trash2" :size="14" class="mr-2" />
        清除已完成
      </Button>
    </div>

    <!-- 添加任务弹窗 -->
    <AddTaskDialog v-model:open="showAddDialog" />
  </div>
</template>
