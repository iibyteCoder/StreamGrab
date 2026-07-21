/**
 * 任务过滤与排序组合式函数
 * 抽取任务列表的过滤、搜索、排序逻辑
 */

import { ref, computed, type MaybeRef } from "vue";
import { toValue } from "vue";
import type { DownloadTask, TaskStatus } from "@/domain";

/** 排序类型 */
export type SortOrder = "newest" | "oldest" | "status";

/** 任务视图类型 */
export type TaskViewType = "active" | "completed";

/** 过滤状态 */
interface FilterState {
  search: string;
  sort: SortOrder;
  status: TaskStatus | "all";
}

/** 状态排序优先级 */
const STATUS_PRIORITY: Record<TaskStatus, number> = {
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

/**
 * 任务过滤组合式函数
 * @param tasks 任务数据源（响应式）
 * @param initialFilter 初始过滤条件
 */
export function useTaskFilter(
  tasks: MaybeRef<DownloadTask[]>,
  initialFilter?: Partial<FilterState>,
) {
  // 过滤状态
  const search = ref(initialFilter?.search ?? "");
  const sort = ref<SortOrder>(initialFilter?.sort ?? "newest");
  const statusFilter = ref<TaskStatus | "all">(initialFilter?.status ?? "all");

  /**
   * 搜索过滤
   */
  const filterBySearch = (
    taskList: DownloadTask[],
    query: string,
  ): DownloadTask[] => {
    if (!query.trim()) return taskList;
    const lowerQuery = query.toLowerCase();
    return taskList.filter(
      (task) =>
        task.url.toLowerCase().includes(lowerQuery) ||
        task.fileName?.toLowerCase().includes(lowerQuery),
    );
  };

  /**
   * 状态过滤
   */
  const filterByStatus = (
    taskList: DownloadTask[],
    status: TaskStatus | "all",
  ): DownloadTask[] => {
    if (status === "all") return taskList;
    return taskList.filter((task) => task.status === status);
  };

  /**
   * 过滤活跃任务（非已完成）
   */
  const filterActive = (taskList: DownloadTask[]): DownloadTask[] => {
    return taskList.filter((task) => task.status !== "completed");
  };

  /**
   * 过滤已完成任务
   */
  const filterCompleted = (taskList: DownloadTask[]): DownloadTask[] => {
    return taskList.filter((task) => task.status === "completed");
  };

  /**
   * 排序任务
   */
  const sortTasks = (
    taskList: DownloadTask[],
    order: SortOrder,
  ): DownloadTask[] => {
    const result = [...taskList];
    switch (order) {
      case "newest":
        result.sort(
          (a, b) =>
            new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime(),
        );
        break;
      case "oldest":
        result.sort(
          (a, b) =>
            new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime(),
        );
        break;
      case "status":
        result.sort(
          (a, b) => STATUS_PRIORITY[a.status] - STATUS_PRIORITY[b.status],
        );
        break;
    }
    return result;
  };

  /**
   * 过滤后的任务列表（全部）
   */
  const filteredTasks = computed(() => {
    let result = toValue(tasks);

    // 搜索
    result = filterBySearch(result, search.value);
    // 状态过滤
    result = filterByStatus(result, statusFilter.value);
    // 排序
    result = sortTasks(result, sort.value);

    return result;
  });

  /**
   * 活跃任务列表（进行中）
   */
  const activeTasks = computed(() => {
    let result = toValue(tasks);
    result = filterActive(result);
    result = filterBySearch(result, search.value);
    result = sortTasks(result, sort.value);
    return result;
  });

  /**
   * 已完成任务列表（历史）
   */
  const completedTasks = computed(() => {
    let result = toValue(tasks);
    result = filterCompleted(result);
    result = filterBySearch(result, search.value);
    result = sortTasks(result, sort.value);
    return result;
  });

  /**
   * 活跃任务数量
   */
  const activeCount = computed(() => activeTasks.value.length);

  /**
   * 已完成任务数量
   */
  const completedCount = computed(() => completedTasks.value.length);

  /**
   * 重置过滤器
   */
  const resetFilter = (): void => {
    search.value = "";
    sort.value = "newest";
    statusFilter.value = "all";
  };

  return {
    // 过滤状态
    search,
    sort,
    statusFilter,

    // 过滤结果
    filteredTasks,
    activeTasks,
    completedTasks,
    activeCount,
    completedCount,

    // 过滤方法
    filterBySearch,
    filterByStatus,
    filterActive,
    filterCompleted,
    sortTasks,
    resetFilter,
  };
}
