<script setup lang="ts">
/**
 * AddTaskDialog - 添加任务弹窗（闭环重建）
 *
 * 渐进披露流程：URL → 解析反馈 → 常用选项 → 高级折叠
 */

import { ref, computed, watch, nextTick } from "vue";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { AppIcon, UrlDuplicateDialog } from "@/components/common";
import { useToast, useDownloader, usePresetManager } from "@/composables";
import { useSettingsStore, useTaskStore } from "@/stores";
import { systemService } from "@/services";
import { StreamSelector } from "@/components/stream";
import { detectUrlType, isStreamingType } from "@/domain/url";
import { extractFileName } from "@/utils/format";
import type {
  StreamInfo,
  StreamSelection,
  DownloadTask,
  TaskOverrides,
  UrlType,
  MuxFormat,
  SubtitleFormat,
} from "@/domain";

interface Props {
  open: boolean;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: "update:open", value: boolean): void;
}>();

const toast = useToast();
const settingsStore = useSettingsStore();
const taskStore = useTaskStore();
const { parseUrl, isParsing, addAndStartTask } = useDownloader();
const { presets, applyPreset } = usePresetManager();

// ========================================
// 状态
// ========================================

// URL
const urlInput = ref("");
const textareaRef = ref<HTMLTextAreaElement | null>(null);
const isSubmitting = ref(false);

// 类型徽章（防抖检测）
const detectedUrlType = ref<UrlType | null>(null);
const showTypeBadge = ref(false);
let urlTypeTimer: ReturnType<typeof setTimeout> | null = null;

// 流解析
const showStreamSelector = ref(false);
const currentStreamInfo = ref<StreamInfo | null>(null);
const pendingUrl = ref<string | null>(null);
const pendingSelection = ref<StreamSelection | undefined>();

// URL 重复弹窗
const showUrlDuplicateDialog = ref(false);
const duplicateTask = ref<DownloadTask | null>(null);

// 拖拽
const isDragging = ref(false);

// 任务级选项
const fileNameInput = ref("");
const saveDirInput = ref("");
const selectedPresetId = ref<string>("__none__");
const scheduleEnabled = ref(false);
const scheduleTime = ref("");

// 高级选项
const showAdvanced = ref(false);
const maxSpeedInput = ref("");
const customRangeInput = ref("");
const muxFormatInput = ref<string>("__global__");
const subtitleFormatInput = ref<string>("__global__");
const subtitlesOnlyInput = ref(false);

// ========================================
// 计算属性
// ========================================

const isOpen = computed({
  get: () => props.open,
  set: (value) => emit("update:open", value),
});

/** 当前 URL 列表（每行一个） */
const parsedUrls = computed(() =>
  urlInput.value
    .split("\n")
    .map((line) => line.trim())
    .filter(
      (line) =>
        line.length > 0 &&
        (line.startsWith("http://") || line.startsWith("https://")),
    ),
);

/** 是否为单链接模式 */
const isSingleUrl = computed(() => parsedUrls.value.length === 1);

/** 全局默认保存目录 */
const globalSaveDir = computed(() => settingsStore.defaultSaveDir);

/** 是否自动开始下载 */
const autoStart = computed(() => settingsStore.autoStartDownload);

// ========================================
// URL 类型检测（防抖 300ms）
// ========================================

watch(urlInput, (val) => {
  showTypeBadge.value = false;
  detectedUrlType.value = null;

  if (urlTypeTimer) clearTimeout(urlTypeTimer);

  const firstUrl = val
    .split("\n")
    .map((l) => l.trim())
    .find((l) => l.startsWith("http://") || l.startsWith("https://"));

  if (!firstUrl) return;

  urlTypeTimer = setTimeout(() => {
    detectedUrlType.value = detectUrlType(firstUrl);
    showTypeBadge.value = true;

    // 自动建议文件名
    if (!fileNameInput.value) {
      fileNameInput.value = extractFileName(firstUrl);
    }
  }, 300);
});

// ========================================
// 弹窗生命周期
// ========================================

watch(isOpen, async (open) => {
  if (open) {
    await nextTick();
    textareaRef.value?.focus();
  }
});

const reset = () => {
  urlInput.value = "";
  isSubmitting.value = false;
  showStreamSelector.value = false;
  currentStreamInfo.value = null;
  pendingUrl.value = null;
  pendingSelection.value = undefined;
  showUrlDuplicateDialog.value = false;
  duplicateTask.value = null;
  detectedUrlType.value = null;
  showTypeBadge.value = false;
  fileNameInput.value = "";
  saveDirInput.value = "";
  selectedPresetId.value = "__none__";
  scheduleEnabled.value = false;
  scheduleTime.value = "";
  showAdvanced.value = false;
  maxSpeedInput.value = "";
  customRangeInput.value = "";
  muxFormatInput.value = "__global__";
  subtitleFormatInput.value = "__global__";
  subtitlesOnlyInput.value = false;
  if (urlTypeTimer) clearTimeout(urlTypeTimer);
};

// ========================================
// 拖拽处理
// ========================================

const handleDragOver = (event: DragEvent) => {
  event.preventDefault();
  isDragging.value = true;
};

const handleDragLeave = () => {
  isDragging.value = false;
};

const handleDrop = async (event: DragEvent) => {
  event.preventDefault();
  isDragging.value = false;

  const urls: string[] = [];
  const text = event.dataTransfer?.getData("text/plain");
  if (text) {
    urls.push(
      ...text
        .split("\n")
        .map((l) => l.trim())
        .filter((l) => l.startsWith("http://") || l.startsWith("https://")),
    );
  }

  if (urls.length > 0) {
    const currentUrls = urlInput.value.trim();
    urlInput.value = currentUrls
      ? `${currentUrls}\n${urls.join("\n")}`
      : urls.join("\n");
  }
};

// ========================================
// 预设处理
// ========================================

const handlePresetChange = (presetId: string) => {
  selectedPresetId.value = presetId;

  if (presetId === "__none__") {
    // 清空预设填入的字段（但保留用户手动输入的）
    return;
  }

  const overrides = applyPreset(presetId);
  if (overrides) {
    // 将 overrides 填入表单
    if (overrides.maxSpeed) maxSpeedInput.value = overrides.maxSpeed;
    if (overrides.customRange) customRangeInput.value = overrides.customRange;
    if (overrides.muxFormat) muxFormatInput.value = overrides.muxFormat;
    if (overrides.subtitleFormat)
      subtitleFormatInput.value = overrides.subtitleFormat;
    if (overrides.subtitlesOnly != null)
      subtitlesOnlyInput.value = overrides.subtitlesOnly;
    if (overrides.saveDir) saveDirInput.value = overrides.saveDir;
    if (overrides.saveName) fileNameInput.value = overrides.saveName;
  }
};

// ========================================
// 保存目录浏览
// ========================================

const handleBrowseSaveDir = async () => {
  const dir = await systemService.selectDirectory();
  if (dir) {
    saveDirInput.value = dir;
  }
};

// ========================================
// 构建 TaskOverrides
// ========================================

const buildOverrides = (): TaskOverrides | undefined => {
  const o: TaskOverrides = {};

  // 保存位置覆盖
  if (saveDirInput.value.trim()) {
    o.saveDir = saveDirInput.value.trim();
  }

  // 文件名覆盖
  if (fileNameInput.value.trim()) {
    o.saveName = fileNameInput.value.trim();
  }

  // 定时开始
  if (scheduleEnabled.value && scheduleTime.value) {
    o.scheduledStartAt = new Date(scheduleTime.value).toISOString();
  }

  // 流选择
  if (pendingSelection.value) {
    o.selection = pendingSelection.value;
  }

  // 高级选项
  if (maxSpeedInput.value.trim()) {
    o.maxSpeed = maxSpeedInput.value.trim();
  }
  if (customRangeInput.value.trim()) {
    o.customRange = customRangeInput.value.trim();
  }
  if (muxFormatInput.value !== "__global__") {
    o.muxFormat = muxFormatInput.value as MuxFormat;
  }
  if (subtitleFormatInput.value !== "__global__") {
    o.subtitleFormat = subtitleFormatInput.value as SubtitleFormat;
  }
  if (subtitlesOnlyInput.value) {
    o.subtitlesOnly = true;
  }

  // 预设来源
  if (selectedPresetId.value !== "__none__") {
    o.presetId = selectedPresetId.value;
  }

  // 只有非空字段才返回 overrides
  const hasOverrides = Object.values(o).some(
    (v) => v !== undefined && v !== null,
  );
  return hasOverrides ? o : undefined;
};

// ========================================
// 提交处理
// ========================================

const handleSubmit = async () => {
  const urls = parsedUrls.value;
  if (urls.length === 0) {
    toast.warning("请输入有效的下载链接");
    return;
  }

  if (isSubmitting.value) return;
  isSubmitting.value = true;

  try {
    // 单链接模式 + 流媒体类型 → 先解析流
    if (
      isSingleUrl.value &&
      detectedUrlType.value &&
      isStreamingType(detectedUrlType.value)
    ) {
      const url = urls[0]!;
      pendingUrl.value = url;
      const info = await parseUrl(url);

      if (info) {
        currentStreamInfo.value = info;
        showStreamSelector.value = true;
        // 等流选择器确认后再创建任务
        return;
      }
      // 解析失败，直接添加
    }

    // 批量或直链：直接添加任务
    await addTasks(urls);
  } catch (error) {
    toast.error(
      `添加任务失败: ${error instanceof Error ? error.message : "未知错误"}`,
    );
  } finally {
    if (!showUrlDuplicateDialog.value && !showStreamSelector.value) {
      isSubmitting.value = false;
    }
  }
};

/** 批量添加任务 */
const addTasks = async (urls: string[]) => {
  const overrides = buildOverrides();
  const saveDir = saveDirInput.value.trim() || undefined;
  const fileName = isSingleUrl.value
    ? fileNameInput.value.trim() || undefined
    : undefined;

  let successCount = 0;
  const duplicateCount = 0;

  for (const url of urls) {
    // URL 重复检测
    const existing = taskStore.checkUrlExists(url);
    if (existing && !isSubmitting.value) {
      // 第一个重复的弹窗提示
      duplicateTask.value = existing;
      pendingUrl.value = url;
      showUrlDuplicateDialog.value = true;
      return;
    }

    try {
      const hasSchedule = scheduleEnabled.value && scheduleTime.value;

      if (autoStart.value && !hasSchedule) {
        await addAndStartTask(url, fileName, saveDir, overrides);
      } else {
        await taskStore.addTask({
          url,
          fileName,
          saveDir,
          overrides,
        });
      }
      successCount++;
    } catch {
      // ignore individual errors
    }
  }

  if (successCount > 0) {
    const hasSchedule = scheduleEnabled.value && scheduleTime.value;
    if (hasSchedule) {
      toast.success(`已添加 ${successCount} 个定时任务`);
    } else {
      toast.success(`已添加 ${successCount} 个任务`);
    }
  }
  if (duplicateCount > 0) {
    toast.warning(`${duplicateCount} 个链接已存在，已跳过`);
  }

  handleClose();
};

// ========================================
// 流选择器回调
// ========================================

const handleStreamConfirm = async (selection: StreamSelection) => {
  showStreamSelector.value = false;
  pendingSelection.value = selection;

  if (pendingUrl.value) {
    const overrides = buildOverrides();
    const saveDir = saveDirInput.value.trim() || undefined;
    const fileName = fileNameInput.value.trim() || undefined;

    // URL 重复检测
    const existing = taskStore.checkUrlExists(pendingUrl.value);
    if (existing) {
      duplicateTask.value = existing;
      showUrlDuplicateDialog.value = true;
      return;
    }

    try {
      const hasSchedule = scheduleEnabled.value && scheduleTime.value;

      if (autoStart.value && !hasSchedule) {
        await addAndStartTask(pendingUrl.value, fileName, saveDir, overrides);
      } else {
        await taskStore.addTask({
          url: pendingUrl.value,
          fileName,
          saveDir,
          overrides,
        });
      }

      toast.success("已添加任务");
      handleClose();
    } catch (e) {
      toast.error(`添加失败: ${e instanceof Error ? e.message : "未知错误"}`);
    }
  }

  isSubmitting.value = false;
};

const handleStreamCancel = () => {
  pendingUrl.value = null;
  pendingSelection.value = undefined;
  currentStreamInfo.value = null;
  showStreamSelector.value = false;
  isSubmitting.value = false;
};

// ========================================
// URL 重复确认
// ========================================

const handleUrlDuplicateConfirm = async () => {
  showUrlDuplicateDialog.value = false;

  if (pendingUrl.value) {
    const overrides = buildOverrides();
    const saveDir = saveDirInput.value.trim() || undefined;
    const fileName = fileNameInput.value.trim() || undefined;

    try {
      // 强制添加（跳过 URL 检查）
      const hasSchedule = scheduleEnabled.value && scheduleTime.value;

      if (autoStart.value && !hasSchedule) {
        await addAndStartTask(pendingUrl.value, fileName, saveDir, overrides);
      } else {
        await taskStore.addTask({
          url: pendingUrl.value,
          fileName,
          saveDir,
          overrides,
          skipUrlCheck: true,
        });
      }
      toast.success("已添加任务");
    } catch (e) {
      toast.error(`添加失败: ${e instanceof Error ? e.message : "未知错误"}`);
    }
  }

  handleClose();
};

const handleUrlDuplicateCancel = () => {
  showUrlDuplicateDialog.value = false;
  isSubmitting.value = false;
};

// ========================================
// 关闭
// ========================================

const handleClose = () => {
  reset();
  isOpen.value = false;
};

// ========================================
// datetime-local 最小时间（当前时间）
// ========================================

const minScheduleTime = computed(() => {
  const now = new Date();
  const pad = (n: number) => n.toString().padStart(2, "0");
  return `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}T${pad(now.getHours())}:${pad(now.getMinutes())}`;
});

// 类型徽章文本
const typeBadgeLabel = computed(() => {
  if (!detectedUrlType.value) return "";
  const labels: Record<UrlType, string> = {
    hls: "HLS",
    dash: "DASH",
    mss: "MSS",
    httpVideo: "直链视频",
    unknown: "未知",
  };
  return labels[detectedUrlType.value];
});

const typeBadgeColor = computed(() => {
  if (!detectedUrlType.value) return "";
  const colors: Record<UrlType, string> = {
    hls: "#3b82f6",
    dash: "#8b5cf6",
    mss: "#06b6d4",
    httpVideo: "#22c55e",
    unknown: "#6b7280",
  };
  return colors[detectedUrlType.value];
});
</script>

<template>
  <Dialog v-model:open="isOpen">
    <DialogContent
      class="sm:max-w-[600px] max-h-[85vh] flex flex-col"
      @close-auto-focus="reset"
    >
      <DialogHeader>
        <DialogTitle class="flex items-center gap-2">
          <AppIcon name="Plus" :size="20" />
          添加下载任务
        </DialogTitle>
      </DialogHeader>

      <div class="flex-1 overflow-y-auto space-y-4 pr-1">
        <!-- URL 输入区域 -->
        <div
          class="relative"
          @dragover="handleDragOver"
          @dragleave="handleDragLeave"
          @drop="handleDrop"
        >
          <!-- 拖拽遮罩 -->
          <div
            v-if="isDragging"
            class="absolute inset-0 z-10 bg-primary/10 rounded-lg border-2 border-dashed border-primary flex items-center justify-center"
          >
            <span class="text-sm font-medium text-primary">释放以添加链接</span>
          </div>

          <textarea
            ref="textareaRef"
            v-model="urlInput"
            placeholder="输入或粘贴下载链接（支持多个链接，每行一个）&#10;&#10;支持格式：&#10;• M3U8 / M3U&#10;• DASH / MPD&#10;• MSS / ISM&#10;• 直链视频（MP4/MKV 等）"
            class="w-full h-36 px-3 py-2 text-sm bg-muted/50 border rounded-lg resize-none focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary transition-colors"
          />
        </div>

        <!-- 类型徽章 + 快捷提示 -->
        <div class="flex items-center justify-between text-xs">
          <div class="flex items-center gap-2">
            <!-- 类型徽章 -->
            <Transition name="badge-fade">
              <span
                v-if="
                  showTypeBadge &&
                  detectedUrlType &&
                  detectedUrlType !== 'unknown'
                "
                class="px-2 py-0.5 rounded-full text-xs font-medium"
                :style="{
                  backgroundColor: `${typeBadgeColor}20`,
                  color: typeBadgeColor,
                }"
              >
                {{ typeBadgeLabel }}
              </span>
            </Transition>
            <span class="text-muted-foreground"> 支持拖放文本或 TXT 文件 </span>
          </div>
          <span class="text-muted-foreground"> Ctrl + V 粘贴剪贴板链接 </span>
        </div>

        <!-- 解析中提示 -->
        <div
          v-if="isParsing"
          class="flex items-center gap-2 text-sm text-primary"
        >
          <AppIcon name="Loader2" :size="16" class="animate-spin" />
          正在解析流信息...
        </div>

        <!-- 任务级选项（始终可见） -->
        <div class="space-y-3">
          <!-- 文件名 -->
          <div class="space-y-1.5">
            <Label class="text-xs text-muted-foreground">文件名</Label>
            <Input
              v-model="fileNameInput"
              placeholder="自动从 URL 提取"
              class="h-9 text-sm"
            />
          </div>

          <!-- 保存位置 -->
          <div class="space-y-1.5">
            <Label class="text-xs text-muted-foreground">保存位置</Label>
            <div class="flex gap-2">
              <Input
                v-model="saveDirInput"
                :placeholder="globalSaveDir || '使用全局默认'"
                class="h-9 text-sm flex-1"
              />
              <Button
                variant="outline"
                size="sm"
                class="h-9 px-3"
                @click="handleBrowseSaveDir"
              >
                <AppIcon name="FolderOpen" :size="14" />
              </Button>
            </div>
          </div>

          <!-- 预设选择器 -->
          <div class="space-y-1.5">
            <Label class="text-xs text-muted-foreground">预设</Label>
            <Select
              :model-value="selectedPresetId"
              @update:model-value="(value) => handlePresetChange(String(value))"
            >
              <SelectTrigger class="h-9 text-sm">
                <SelectValue placeholder="不使用预设" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="__none__">不使用预设</SelectItem>
                <SelectItem
                  v-for="preset in presets"
                  :key="preset.id"
                  :value="preset.id"
                >
                  {{ preset.name }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          <!-- 定时开始 -->
          <div class="space-y-1.5">
            <div class="flex items-center gap-2">
              <input
                type="checkbox"
                :checked="scheduleEnabled"
                class="w-4 h-4 rounded border accent-primary shrink-0"
                @change="scheduleEnabled = !scheduleEnabled"
              />
              <Label class="text-xs text-muted-foreground cursor-pointer"
                >定时开始</Label
              >
            </div>
            <Input
              v-if="scheduleEnabled"
              v-model="scheduleTime"
              type="datetime-local"
              :min="minScheduleTime"
              class="h-9 text-sm datetime-dark"
            />
            <p v-if="scheduleEnabled" class="text-xs text-muted-foreground/70">
              到达时间且应用运行时自动开始
            </p>
          </div>
        </div>

        <!-- 高级选项折叠 -->
        <div>
          <button
            class="flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground transition-colors cursor-pointer"
            @click="showAdvanced = !showAdvanced"
          >
            <AppIcon
              name="ChevronRight"
              :size="14"
              class="transition-transform duration-150"
              :class="{ 'rotate-90': showAdvanced }"
            />
            高级选项
          </button>

          <Transition name="fold">
            <div v-if="showAdvanced" class="mt-3 space-y-3 pl-1">
              <!-- 限速 -->
              <div class="space-y-1.5">
                <Label class="text-xs text-muted-foreground">限速</Label>
                <Input
                  v-model="maxSpeedInput"
                  placeholder="如 10M，留空跟随全局"
                  class="h-9 text-sm"
                />
              </div>

              <!-- 下载范围 -->
              <div class="space-y-1.5">
                <Label class="text-xs text-muted-foreground">下载范围</Label>
                <Input
                  v-model="customRangeInput"
                  placeholder="如 00:00:00-00:10:00"
                  class="h-9 text-sm"
                />
              </div>

              <!-- 容器格式 -->
              <div class="space-y-1.5">
                <Label class="text-xs text-muted-foreground">容器格式</Label>
                <Select v-model="muxFormatInput">
                  <SelectTrigger class="h-9 text-sm">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="__global__">跟随全局</SelectItem>
                    <SelectItem value="mp4">MP4</SelectItem>
                    <SelectItem value="mkv">MKV</SelectItem>
                  </SelectContent>
                </Select>
              </div>

              <!-- 字幕格式 -->
              <div class="space-y-1.5">
                <Label class="text-xs text-muted-foreground">字幕格式</Label>
                <Select v-model="subtitleFormatInput">
                  <SelectTrigger class="h-9 text-sm">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="__global__">跟随全局</SelectItem>
                    <SelectItem value="SRT">SRT</SelectItem>
                    <SelectItem value="VTT">VTT</SelectItem>
                  </SelectContent>
                </Select>
              </div>

              <!-- 仅下载字幕 -->
              <div class="flex items-center gap-2">
                <input
                  type="checkbox"
                  :checked="subtitlesOnlyInput"
                  class="w-4 h-4 rounded border accent-primary shrink-0"
                  @change="subtitlesOnlyInput = !subtitlesOnlyInput"
                />
                <Label
                  class="text-xs text-muted-foreground cursor-pointer"
                  @click="subtitlesOnlyInput = !subtitlesOnlyInput"
                  >仅下载字幕</Label
                >
              </div>
            </div>
          </Transition>
        </div>
      </div>

      <!-- 操作按钮 -->
      <div class="flex justify-end gap-2 pt-3 border-t shrink-0">
        <Button variant="outline" @click="handleClose">取消</Button>
        <Button
          :disabled="isSubmitting || !urlInput.trim()"
          @click="handleSubmit"
        >
          <AppIcon
            v-if="isSubmitting"
            name="Loader2"
            :size="16"
            class="mr-2 animate-spin"
          />
          <AppIcon v-else name="Download" :size="16" class="mr-2" />
          {{ isSubmitting ? "处理中..." : "添加任务" }}
        </Button>
      </div>

      <!-- 流选择器 -->
      <StreamSelector
        v-model:open="showStreamSelector"
        :stream-info="currentStreamInfo"
        :loading="isParsing"
        @confirm="handleStreamConfirm"
        @cancel="handleStreamCancel"
      />

      <!-- URL 重复确认弹窗 -->
      <UrlDuplicateDialog
        v-model:open="showUrlDuplicateDialog"
        :existing-task="duplicateTask"
        @confirm="handleUrlDuplicateConfirm"
        @cancel="handleUrlDuplicateCancel"
      />
    </DialogContent>
  </Dialog>
</template>

<style scoped>
/* 类型徽章淡入 */
.badge-fade-enter-active {
  transition: opacity 150ms ease-out;
}
.badge-fade-enter-from {
  opacity: 0;
}

/* 高级折叠 */
.fold-enter-active,
.fold-leave-active {
  transition: all 150ms ease-out;
  overflow: hidden;
}
.fold-enter-from,
.fold-leave-to {
  opacity: 0;
  max-height: 0;
}
.fold-enter-to,
.fold-leave-from {
  opacity: 1;
  max-height: 400px;
}

/* datetime-local 暗色主题适配 */
.datetime-dark {
  color-scheme: dark;
}
</style>
