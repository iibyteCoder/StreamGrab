<script setup lang="ts">
/**
 * ProgressChart - 下载速率图表组件
 * 横坐标：进度百分比
 * 纵坐标：下载速率
 *
 * 支持两种数据源：
 * 1. 实时进度数据（下载中，从 taskStore 获取）
 * 2. 数据库历史数据（下载完成后）
 */

import { ref, watch, onMounted, computed } from "vue";
import {
  Chart as ChartJS,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Tooltip,
  Legend,
  Filler,
  type ChartData,
  type ChartOptions,
} from "chart.js";
import { Line } from "vue-chartjs";
import {
  taskService,
  type ProgressHistoryRecord,
} from "@/services/taskService";
import { formatSpeed } from "@/utils/format";
import { useTaskStore } from "@/stores";

// 注册 Chart.js 组件
ChartJS.register(
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Tooltip,
  Legend,
  Filler,
);

interface Props {
  taskId: string;
  /** 图表高度 */
  height?: number;
  /** 是否显示速率信息（已废弃，保留兼容） */
  showSpeed?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  height: 180,
  showSpeed: false,
});

const taskStore = useTaskStore();

// 数据库历史数据
const dbHistoryData = ref<ProgressHistoryRecord[]>([]);
const isLoading = ref(false);
const error = ref<string | null>(null);

// 实时数据点（下载中时从 store 收集）
const liveDataPoints = ref<Array<{ percent: number; speed: number }>>([]);

// 获取当前任务
const task = computed(() => taskStore.getTask(props.taskId));

// 是否正在下载
const isDownloading = computed(() => task.value?.status === "downloading");

// 加载历史数据
const loadHistory = async () => {
  if (!props.taskId) return;

  isLoading.value = true;
  error.value = null;

  try {
    dbHistoryData.value = await taskService.getProgressHistory(
      props.taskId,
      200,
    );
  } catch (e) {
    error.value = e instanceof Error ? e.message : "加载失败";
    console.error("Failed to load progress history:", e);
  } finally {
    isLoading.value = false;
  }
};

// 合并的图表数据（实时 + 历史）
const allDataPoints = computed(() => {
  const points: Array<{ percent: number; speed: number }> = [];

  // 1. 首先添加数据库历史数据
  for (const record of dbHistoryData.value) {
    if (record.speed > 0) {
      points.push({ percent: record.percent, speed: record.speed });
    }
  }

  // 2. 然后添加/更新实时数据点（会覆盖相同进度的历史数据）
  for (const live of liveDataPoints.value) {
    if (live.speed > 0) {
      // 检查是否已有相同或相近进度的点
      const existingIndex = points.findIndex(
        (p) => Math.abs(p.percent - live.percent) < 1,
      );
      if (existingIndex >= 0) {
        points[existingIndex] = live;
      } else {
        points.push(live);
      }
    }
  }

  // 3. 如果还在下载中，添加当前进度点
  if (isDownloading.value && task.value) {
    const currentPercent = task.value.progressPercent;
    const currentSpeed = task.value.progressSpeed;
    if (currentSpeed > 0 && currentPercent > 0) {
      // 检查是否已有相近的点
      const existingIndex = points.findIndex(
        (p) => Math.abs(p.percent - currentPercent) < 1,
      );
      if (existingIndex >= 0) {
        points[existingIndex] = {
          percent: currentPercent,
          speed: currentSpeed,
        };
      } else {
        points.push({ percent: currentPercent, speed: currentSpeed });
      }
    }
  }

  // 按进度排序
  points.sort((a, b) => a.percent - b.percent);

  return points;
});

// 计算统计数据
const stats = computed(() => {
  const points = allDataPoints.value;
  if (points.length === 0) return null;

  const speeds = points.map((d) => d.speed).filter((s) => s > 0);
  if (speeds.length === 0) return null;

  const maxSpeed = Math.max(...speeds);
  const minSpeed = Math.min(...speeds);
  const avgSpeed = speeds.reduce((a, b) => a + b, 0) / speeds.length;

  // 当前速率：从实时任务获取或使用最后一个数据点
  const currentSpeed =
    isDownloading.value && task.value
      ? task.value.progressSpeed
      : (speeds[speeds.length - 1] ?? 0);

  return {
    maxSpeed,
    minSpeed,
    avgSpeed,
    currentSpeed,
  };
});

// 图表数据：横坐标是进度百分比，纵坐标是速率
const chartData = computed<ChartData<"line">>(() => {
  const points = allDataPoints.value;

  return {
    datasets: [
      {
        label: "下载速率",
        data: points.map((d) => ({
          x: d.percent,
          y: d.speed,
        })),
        borderColor: "rgb(59, 130, 246)",
        backgroundColor: "rgba(59, 130, 246, 0.15)",
        fill: true,
        tension: 0.4,
        pointRadius: 0,
        pointHoverRadius: 4,
        borderWidth: 2,
      },
      // 平均速率线
      ...(stats.value && stats.value.avgSpeed > 0
        ? [
            {
              label: "平均速率",
              data: [
                { x: 0, y: stats.value.avgSpeed },
                { x: 100, y: stats.value.avgSpeed },
              ],
              borderColor: "rgb(251, 191, 36)",
              backgroundColor: "transparent",
              borderDash: [5, 5],
              fill: false,
              pointRadius: 0,
              borderWidth: 1.5,
            },
          ]
        : []),
    ],
  };
});

// 图表选项
const chartOptions = computed<ChartOptions<"line">>(() => ({
  responsive: true,
  maintainAspectRatio: false,
  interaction: {
    mode: "nearest" as const,
    axis: "x" as const,
    intersect: false,
  },
  plugins: {
    legend: {
      display: true,
      position: "top" as const,
      labels: {
        color: "#888",
        usePointStyle: true,
        pointStyle: "circle",
        font: { size: 11 },
      },
    },
    tooltip: {
      callbacks: {
        title: (items) => {
          const first = items[0];
          if (
            first &&
            first.parsed.x !== undefined &&
            first.parsed.x !== null
          ) {
            return `进度: ${Math.round(first.parsed.x)}%`;
          }
          return "";
        },
        label: (context) => {
          const y = context.parsed.y ?? 0;
          return `${context.dataset.label}: ${formatSpeed(y)}`;
        },
      },
    },
  },
  scales: {
    x: {
      type: "linear" as const,
      display: true,
      min: 0,
      max: 100,
      title: {
        display: true,
        text: "进度 %",
        color: "#666",
        font: { size: 11 },
      },
      ticks: {
        color: "#666",
        stepSize: 20,
        callback: (value) => `${value}%`,
      },
      grid: {
        color: "rgba(255, 255, 255, 0.05)",
      },
    },
    y: {
      type: "linear" as const,
      display: true,
      min: 0,
      title: {
        display: true,
        text: "速率",
        color: "#666",
        font: { size: 11 },
      },
      ticks: {
        color: "#666",
        callback: (value) => formatSpeed(value as number),
      },
      grid: {
        color: "rgba(255, 255, 255, 0.05)",
      },
    },
  },
  // 动画配置：下载中时禁用动画以提高性能
  animation: isDownloading.value ? false : undefined,
}));

// 实时更新：监听任务进度变化
let lastRecordedPercent = 0;
const RECORD_INTERVAL = 2; // 每 2% 进度记录一次

watch(
  () => task.value?.progressPercent,
  (newPercent) => {
    if (!isDownloading.value || !task.value) return;

    const speed = task.value.progressSpeed;
    if (speed <= 0 || newPercent === undefined) return;

    // 每隔一定进度记录一次，避免数据点过多
    if (
      newPercent - lastRecordedPercent >= RECORD_INTERVAL ||
      liveDataPoints.value.length === 0
    ) {
      liveDataPoints.value.push({ percent: newPercent, speed });
      lastRecordedPercent = newPercent;
    } else if (liveDataPoints.value.length > 0) {
      // 更新最后一个点
      liveDataPoints.value[liveDataPoints.value.length - 1] = {
        percent: newPercent,
        speed,
      };
    }
  },
);

// 监听 taskId 变化
watch(
  () => props.taskId,
  () => {
    // 重置实时数据
    liveDataPoints.value = [];
    lastRecordedPercent = 0;
    // 加载历史数据
    loadHistory();
  },
  { immediate: true },
);

// 监听下载完成，刷新历史数据
watch(
  () => task.value?.status,
  (newStatus, oldStatus) => {
    if (oldStatus === "downloading" && newStatus !== "downloading") {
      // 下载结束，重新加载历史数据
      loadHistory();
    }
  },
);

onMounted(loadHistory);
</script>

<template>
  <div class="progress-chart space-y-3">
    <!-- 加载中 -->
    <div
      v-if="isLoading && allDataPoints.length === 0"
      class="flex items-center justify-center"
      :style="{ height: `${height}px` }"
    >
      <div
        class="animate-spin w-5 h-5 border-2 border-primary border-t-transparent rounded-full"
      />
    </div>

    <!-- 无数据 -->
    <div
      v-else-if="allDataPoints.length === 0"
      class="flex items-center justify-center text-muted-foreground text-sm"
      :style="{ height: `${height}px` }"
    >
      {{ isDownloading ? "等待下载数据..." : "暂无下载数据" }}
    </div>

    <!-- 图表和统计 -->
    <template v-else>
      <!-- 统计信息 -->
      <div v-if="stats" class="grid grid-cols-3 gap-2 text-center">
        <div class="bg-muted/30 rounded-lg p-2">
          <div class="text-xs text-muted-foreground">峰值速率</div>
          <div class="text-sm font-medium text-green-500">
            {{ formatSpeed(stats.maxSpeed) }}
          </div>
        </div>
        <div class="bg-muted/30 rounded-lg p-2">
          <div class="text-xs text-muted-foreground">平均速率</div>
          <div class="text-sm font-medium text-amber-500">
            {{ formatSpeed(stats.avgSpeed) }}
          </div>
        </div>
        <div class="bg-muted/30 rounded-lg p-2">
          <div class="text-xs text-muted-foreground">当前速率</div>
          <div class="text-sm font-medium text-primary">
            {{ formatSpeed(stats.currentSpeed) }}
          </div>
        </div>
      </div>

      <!-- 图表 -->
      <div :style="{ height: `${height}px` }">
        <Line :data="chartData" :options="chartOptions" />
      </div>
    </template>
  </div>
</template>

<style scoped>
.progress-chart {
  width: 100%;
}
</style>
