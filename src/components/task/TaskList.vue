<script setup lang="ts">
/**
 * TaskList - 任务列表组件
 * 纯展示组件，负责渲染任务卡片列表
 */

import TaskCard from "./TaskCard.vue";
import TaskEmptyState from "./TaskEmptyState.vue";
import type { DownloadTask } from "@/domain";

interface Props {
  /** 任务列表 */
  tasks: DownloadTask[];
  /** 空状态类型 */
  emptyType?: "active" | "completed" | "all";
  /** 自定义空状态文本 */
  emptyText?: string;
  /** 当前选中的任务 ID（用于高亮显示） */
  activeTaskId?: string | null;
}

withDefaults(defineProps<Props>(), {
  emptyType: "all",
  activeTaskId: null,
});

defineEmits<{
  (e: "taskClick", task: DownloadTask): void;
  (e: "taskRedownload", task: DownloadTask): void;
}>();
</script>

<template>
  <div class="task-list h-full flex flex-col">
    <!-- 任务列表 -->
    <TransitionGroup
      v-if="tasks.length > 0"
      name="task-list"
      tag="div"
      class="grid gap-2"
    >
      <TaskCard
        v-for="task in tasks"
        :key="task.id"
        :task="task"
        :active="activeTaskId === task.id"
        @click="$emit('taskClick', $event)"
        @redownload="$emit('taskRedownload', $event)"
      />
    </TransitionGroup>

    <!-- 空状态 -->
    <div v-else class="flex-1 flex items-center justify-center">
      <TaskEmptyState :type="emptyType" :title="emptyText" />
    </div>
  </div>
</template>

<style scoped>
.task-list-enter-active,
.task-list-leave-active {
  transition: all 0.25s ease-out;
}

.task-list-enter-from {
  opacity: 0;
  transform: translateY(-8px);
}

.task-list-leave-to {
  opacity: 0;
  transform: translateX(8px);
}

.task-list-move {
  transition: transform 0.25s ease-out;
}
</style>
