<script setup lang="ts">
/**
 * TaskList 任务列表组件
 * 显示所有下载任务
 */

import { computed } from 'vue';
import TaskCard from './TaskCard.vue';
import { useTasks } from '@/composables';
import type { DownloadTask, TaskStatus } from '@/types';

type SortOrder = 'newest' | 'oldest' | 'status';

interface Props {
  filter?: TaskStatus | 'all';
  sort?: SortOrder;
  search?: string;
  emptyText?: string;
}

const props = withDefaults(defineProps<Props>(), {
  filter: 'all',
  sort: 'newest',
  emptyText: '暂无下载任务',
});

const emit = defineEmits<{
  (e: 'taskClick', task: DownloadTask): void;
  (e: 'taskContextmenu', event: MouseEvent, task: DownloadTask): void;
}>();

const { tasks } = useTasks();

// 过滤和排序后的任务列表
const filteredTasks = computed(() => {
  let result = [...tasks.value];

  // 搜索过滤
  if (props.search) {
    const query = props.search.toLowerCase();
    result = result.filter(
      (task) =>
        task.url.toLowerCase().includes(query) ||
        task.fileName?.toLowerCase().includes(query)
    );
  }

  // 状态过滤
  if (props.filter !== 'all') {
    result = result.filter((task) => task.status === props.filter);
  }

  // 排序
  switch (props.sort) {
    case 'newest':
      result.sort((a, b) => b.createdAt.getTime() - a.createdAt.getTime());
      break;
    case 'oldest':
      result.sort((a, b) => a.createdAt.getTime() - b.createdAt.getTime());
      break;
    case 'status':
      // 按状态优先级排序：下载中 > 解析中 > 合并中 > 混流中 > 暂停 > 等待中 > 失败 > 已取消 > 已完成
      const statusOrder: Record<TaskStatus, number> = {
        downloading: 1,
        analyzing: 2,
        merging: 3,
        muxing: 4,
        paused: 5,
        pending: 6,
        failed: 7,
        cancelled: 8,
        completed: 9,
      };
      result.sort((a, b) => statusOrder[a.status] - statusOrder[b.status]);
      break;
  }

  return result;
});

// 是否为空
const isEmpty = computed(() => filteredTasks.value.length === 0);

// 处理任务点击
const handleTaskClick = (task: DownloadTask) => {
  emit('taskClick', task);
};

// 处理任务右键菜单
const handleTaskContextmenu = (event: MouseEvent, task: DownloadTask) => {
  emit('taskContextmenu', event, task);
};
</script>

<template>
  <div class="task-list">
    <!-- 任务列表 -->
    <TransitionGroup
      v-if="!isEmpty"
      name="task-list"
      tag="div"
      class="grid gap-3"
    >
      <TaskCard
        v-for="task in filteredTasks"
        :key="task.id"
        :task="task"
        @click="handleTaskClick"
        @contextmenu="handleTaskContextmenu"
      />
    </TransitionGroup>

    <!-- 空状态 -->
    <div
      v-else
      class="flex flex-col items-center justify-center py-16 text-center"
    >
      <svg
        class="w-16 h-16 text-text-muted opacity-50 mb-4"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="1.5"
          d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"
        />
      </svg>
      <p class="text-text-muted">{{ emptyText }}</p>
      <p class="text-text-muted text-sm mt-1">输入链接开始下载</p>
    </div>
  </div>
</template>

<style scoped>
.task-list-enter-active,
.task-list-leave-active {
  transition: all 0.3s ease;
}

.task-list-enter-from {
  opacity: 0;
  transform: translateY(-10px);
}

.task-list-leave-to {
  opacity: 0;
  transform: translateX(10px);
}

.task-list-move {
  transition: transform 0.3s ease;
}
</style>
