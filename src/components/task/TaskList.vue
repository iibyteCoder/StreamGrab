<script setup lang="ts">
/**
 * TaskList - 任务列表组件
 * 纯展示组件，负责渲染任务卡片列表
 */

import TaskCard from "./TaskCard.vue";
import TaskEmptyState from "./TaskEmptyState.vue";
import type { DownloadTask } from "@/types";

interface Props {
  /** 任务列表 */
  tasks: DownloadTask[];
  /** 空状态类型 */
  emptyType?: "active" | "completed" | "all";
  /** 自定义空状态文本 */
  emptyText?: string;
}

withDefaults(defineProps<Props>(), {
  emptyType: "all",
});

defineEmits<{
  (e: "taskClick", task: DownloadTask): void;
}>();
</script>

<template>
  <div class="task-list h-full">
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
        @click="$emit('taskClick', $event)"
      />
    </TransitionGroup>

    <!-- 空状态 -->
    <TaskEmptyState v-else :type="emptyType" :title="emptyText" />
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
