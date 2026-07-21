<script setup lang="ts">
/**
 * ToolManagerCard - 工具管理卡片
 *
 * 参数化的工具管理组件，Nm3u8dlTab 和 FfmpegTab 共用。
 * 负责：检测工具状态、路径配置、版本更新与下载。
 */

import { ref, computed, onMounted, watch } from "vue";
import { useI18n } from "vue-i18n";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { AppIcon } from "@/components/common";
import { toolsService, systemService } from "@/services";
import type {
  ToolInfo,
  ToolReleaseInfo,
  ToolDownloadProgress,
} from "@/services";
import { useToast } from "@/composables";
import { SettingsGroup, SettingInput } from ".";

const { t } = useI18n();

type ToolId = "nm3u8dl" | "ffmpeg";

interface Props {
  toolId: ToolId;
  /** 当前配置中的路径（目录或完整路径） */
  configPath: string;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  /** 路径变更（用户选择目录或下载完成后自动写入） */
  (e: "pathChange", path: string): void;
}>();

const toast = useToast();

// ========================================
// 工具显示名称
// ========================================

const toolDisplayName = computed(() =>
  props.toolId === "nm3u8dl" ? "N_m3u8DL-RE" : "FFmpeg",
);

// ========================================
// 检测状态
// ========================================

const toolInfo = ref<ToolInfo | null>(null);
const isDetecting = ref(false);

async function detectTool(pathOverride?: string) {
  isDetecting.value = true;
  try {
    const path = pathOverride ?? props.configPath;
    if (props.toolId === "nm3u8dl") {
      toolInfo.value = await toolsService.getNm3u8dlInfo(path || null);
    } else {
      toolInfo.value = await toolsService.getFfmpegInfo(path || null);
    }
  } catch (e) {
    console.error(`Failed to detect ${toolDisplayName.value}:`, e);
    toolInfo.value = null;
  } finally {
    isDetecting.value = false;
  }
}

// 监听 configPath 变化重新检测
watch(
  () => props.configPath,
  () => {
    detectTool();
  },
);

onMounted(() => {
  detectTool();
});

// ========================================
// 路径选择
// ========================================

const isSelectingPath = ref(false);

async function handleSelectDirectory() {
  if (isSelectingPath.value) return;
  isSelectingPath.value = true;
  try {
    const dir = await systemService.selectDirectory();
    if (dir) {
      emit("pathChange", dir);
      // detectTool will be triggered by the watch on configPath
    }
  } catch {
    toast.error(t("settings.tool.selectDirFailed", "选择目录失败"));
  } finally {
    isSelectingPath.value = false;
  }
}

// ========================================
// 版本检测与下载
// ========================================

const latestRelease = ref<ToolReleaseInfo | null>(null);
const isCheckingLatest = ref(false);
const isDownloading = ref(false);
const downloadProgress = ref<ToolDownloadProgress | null>(null);

const hasUpdate = computed(() => {
  if (!toolInfo.value?.version || !latestRelease.value?.version) return false;
  return (
    compareVersions(latestRelease.value.version, toolInfo.value.version) > 0
  );
});

const downloadPercent = computed(() => {
  if (!downloadProgress.value) return 0;
  return downloadProgress.value.percent;
});

const downloadStatusText = computed(() => {
  if (!downloadProgress.value) return "";
  if (downloadProgress.value.status === "extracting")
    return t("settings.tool.extracting", "解压中...");
  if (downloadProgress.value.status === "downloaded")
    return t("settings.tool.downloadComplete", "下载完成");
  return `${downloadProgress.value.percent.toFixed(1)}%`;
});

async function handleCheckLatest() {
  isCheckingLatest.value = true;
  try {
    if (props.toolId === "nm3u8dl") {
      latestRelease.value = await toolsService.getNm3u8dlLatestRelease();
    } else {
      latestRelease.value = await toolsService.getFfmpegLatestRelease();
    }
  } catch {
    toast.error(t("settings.tool.fetchLatestFailed", "获取最新版本信息失败"));
  } finally {
    isCheckingLatest.value = false;
  }
}

async function handleDownload() {
  if (!latestRelease.value) {
    await handleCheckLatest();
    if (!latestRelease.value) {
      toast.error(t("settings.tool.fetchLatestFailed", "获取最新版本信息失败"));
      return;
    }
  }

  isDownloading.value = true;
  downloadProgress.value = null;

  let unlisten: (() => void) | null = null;
  try {
    const toolName = props.toolId === "nm3u8dl" ? "N_m3u8DL-RE" : "FFmpeg";

    unlisten = await toolsService.subscribeToDownloadProgress(
      toolName,
      (progress) => {
        downloadProgress.value = progress;
      },
    );

    // Use the current exe directory as target, or fallback to configPath
    const targetDir = toolInfo.value?.dirPath || props.configPath || "";

    const extractedPath = await toolsService.downloadTool(
      toolName,
      latestRelease.value.downloadUrl,
      targetDir,
    );

    emit("pathChange", extractedPath);
    toast.success(
      `${toolDisplayName.value} ${t("settings.tool.downloadComplete", "下载完成")}`,
    );

    // Re-detect
    await detectTool(extractedPath);
  } catch (e) {
    toast.error(
      `${t("settings.tool.downloadFailed", "下载失败")}: ${e instanceof Error ? e.message : String(e)}`,
    );
  } finally {
    isDownloading.value = false;
    downloadProgress.value = null;
    if (unlisten) unlisten();
  }
}

// ========================================
// 工具函数
// ========================================

function compareVersions(v1: string, v2: string): number {
  const parts1 = v1.replace(/^v/, "").split(".").map(Number);
  const parts2 = v2.replace(/^v/, "").split(".").map(Number);
  for (let i = 0; i < Math.max(parts1.length, parts2.length); i++) {
    const p1 = parts1[i] || 0;
    const p2 = parts2[i] || 0;
    if (p1 > p2) return 1;
    if (p1 < p2) return -1;
  }
  return 0;
}

function formatPublishedDate(dateStr: string): string {
  try {
    return new Date(dateStr).toLocaleDateString();
  } catch {
    return dateStr;
  }
}
</script>

<template>
  <SettingsGroup
    :title="`${toolDisplayName} ${t('settings.tool.toolManagement', '工具管理')}`"
    :description="`${t('settings.tool.manageDescription', '管理')} ${toolDisplayName} ${t('settings.tool.manageDescriptionSuffix', '的路径、版本与更新')}`"
  >
    <!-- 路径配置 -->
    <div class="flex items-end gap-2">
      <div class="flex-1">
        <SettingInput
          :model-value="configPath"
          :label="`${toolDisplayName} ${t('settings.tool.path', '路径')}`"
          :placeholder="t('settings.tool.pathPlaceholder', '留空使用系统 PATH')"
          :help="`${t('settings.tool.pathHelp', '填写包含')} ${toolDisplayName} ${t('settings.tool.pathHelpSuffix', '可执行文件的目录路径')}`"
          @update:model-value="emit('pathChange', String($event))"
          @blur="detectTool()"
        />
      </div>
      <Button
        variant="outline"
        size="sm"
        :disabled="isSelectingPath"
        class="cursor-pointer"
        @click="handleSelectDirectory"
      >
        <AppIcon
          v-if="isSelectingPath"
          name="Loader2"
          :size="16"
          class="mr-2 animate-spin"
        />
        <AppIcon v-else name="FolderOpen" :size="16" class="mr-2" />
        {{ t("settings.tool.selectDirectory", "选择目录") }}
      </Button>
      <Button
        variant="outline"
        size="sm"
        :disabled="isDetecting"
        class="cursor-pointer"
        :title="t('settings.tool.refresh', '重新检测')"
        @click="detectTool()"
      >
        <AppIcon
          :name="isDetecting ? 'Loader2' : 'RefreshCw'"
          :size="16"
          :class="{ 'animate-spin': isDetecting }"
        />
      </Button>
    </div>

    <!-- 状态显示 -->
    <div v-if="toolInfo" class="flex flex-col gap-1.5 text-xs">
      <div class="flex items-center gap-2">
        <!-- 状态徽章 -->
        <span
          v-if="toolInfo.installed"
          class="inline-flex items-center gap-1.5 rounded-full px-2.5 py-0.5 text-xs font-medium"
          style="
            background: rgba(34, 197, 94, 0.12);
            color: var(--accent-success);
          "
        >
          <AppIcon name="CheckCircle" :size="12" />
          {{ t("settings.tool.installed", "已安装") }}
        </span>
        <span
          v-else
          class="inline-flex items-center gap-1.5 rounded-full px-2.5 py-0.5 text-xs font-medium"
          style="
            background: rgba(239, 68, 68, 0.12);
            color: var(--accent-error);
          "
        >
          <AppIcon name="XCircle" :size="12" />
          {{ t("settings.tool.notInstalled", "未安装") }}
        </span>

        <!-- 版本号 -->
        <span
          v-if="toolInfo.version"
          class="font-mono"
          style="color: var(--text-secondary)"
        >
          v{{ toolInfo.version }}
        </span>

        <!-- 更新提示 -->
        <span
          v-if="hasUpdate"
          class="text-xs"
          style="color: var(--accent-primary)"
        >
          ({{ t("settings.tool.hasNewVersion", "有新版本") }}
          {{ latestRelease?.version }})
        </span>
      </div>

      <!-- 可执行文件路径 -->
      <div
        v-if="toolInfo.exePath"
        class="font-mono truncate"
        style="color: var(--text-secondary); opacity: 0.7"
        :title="toolInfo.exePath"
      >
        {{ toolInfo.exePath }}
      </div>

      <!-- 错误信息 -->
      <div
        v-if="!toolInfo.installed && toolInfo.error"
        style="color: var(--accent-error)"
      >
        {{ toolInfo.error }}
      </div>
    </div>

    <!-- 版本更新区域 -->
    <div class="flex items-center gap-3">
      <Button
        variant="outline"
        size="sm"
        :disabled="isCheckingLatest || isDownloading"
        class="cursor-pointer"
        @click="handleCheckLatest"
      >
        <AppIcon
          v-if="isCheckingLatest"
          name="Loader2"
          :size="14"
          class="mr-1.5 animate-spin"
        />
        <AppIcon v-else name="Search" :size="14" class="mr-1.5" />
        {{ t("settings.tool.checkLatest", "检查最新版本") }}
      </Button>

      <Button
        v-if="latestRelease && !isDownloading"
        variant="default"
        size="sm"
        class="cursor-pointer"
        @click="handleDownload"
      >
        <AppIcon name="Download" :size="14" class="mr-1.5" />
        {{
          hasUpdate
            ? t("settings.tool.update", "更新")
            : t("settings.tool.download", "下载")
        }}
      </Button>
    </div>

    <!-- 最新版本信息 -->
    <div
      v-if="latestRelease && !isDownloading"
      class="rounded-md p-3 text-xs"
      style="
        background: var(--bg-surface);
        border: 1px solid rgba(255, 255, 255, 0.08);
        color: var(--text-secondary);
      "
    >
      <div class="flex items-center gap-2">
        <AppIcon name="Package" :size="14" />
        <span>
          {{ t("settings.tool.latestVersion", "最新版本") }}:
          <span class="font-mono">{{ latestRelease.version }}</span>
        </span>
        <span v-if="latestRelease.publishedAt" class="ml-1">
          ({{ formatPublishedDate(latestRelease.publishedAt) }})
        </span>
      </div>
    </div>

    <!-- 下载进度 -->
    <div v-if="isDownloading" class="space-y-2">
      <div class="flex items-center justify-between text-xs">
        <span style="color: var(--text-secondary)">{{
          downloadStatusText
        }}</span>
        <span v-if="downloadProgress" class="font-mono">
          {{ downloadPercent.toFixed(1) }}%
        </span>
      </div>
      <Progress :model-value="downloadPercent" class="h-1.5" />
    </div>
  </SettingsGroup>
</template>
