<script setup lang="ts">
/**
 * GeneralSettings - 常规设置组件
 */

import { computed } from "vue";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { SettingSwitch, SettingSelect, SettingPath, SettingsGroup } from "..";
import { useUpdateChecker } from "@/composables";
import { AppIcon } from "@/components/common";

interface Settings {
  general: {
    saveDir: string;
    tmpDir: string;
    language: string;
    autoStartDownload: boolean;
    minimizeToTray: boolean;
    checkUpdate: boolean;
  };
}

interface Props {
  settings: Settings;
}

defineProps<Props>();

const emit = defineEmits<{
  (e: "update:settings", value: any): void;
}>();

// 更新检查器
const {
  isChecking,
  updateAvailable,
  currentVersion,
  latestVersion,
  downloadStatus,
  downloadProgress,
  downloadedSize,
  totalSize,
  selectedAsset,
  downloadedFilePath,
  checkForUpdate,
  downloadUpdate,
  cancelDownload,
  openDownloadPage,
  openDownloadLocation,
  runInstallerAgain,
  formatFileSize,
} = useUpdateChecker();

// 语言选项
const languageOptions = [
  { value: "zh-CN", label: "简体中文" },
  { value: "zh-TW", label: "繁体中文" },
  { value: "en-US", label: "English" },
];

// 更新设置
const updateGeneral = (value: any) => {
  emit("update:settings", value);
};

// 手动检查更新
const handleCheckUpdate = async () => {
  await checkForUpdate(true);
};

// 下载更新
const handleDownloadUpdate = async () => {
  await downloadUpdate();
};

// 取消下载
const handleCancelDownload = () => {
  cancelDownload();
};

// 是否正在下载
const isDownloading = computed(() => downloadStatus.value === "downloading");

// 是否下载完成
const isDownloaded = computed(() => downloadStatus.value === "downloaded");

// 进度条显示文本
const progressText = computed(() => {
  if (totalSize.value > 0) {
    return `${formatFileSize(downloadedSize.value)} / ${formatFileSize(totalSize.value)}`;
  }
  return formatFileSize(downloadedSize.value);
});

// 从完整路径中提取文件名
const displayFileName = computed(() => {
  if (!downloadedFilePath.value) return "";
  const parts = downloadedFilePath.value.replace(/\\/g, "/").split("/");
  return parts[parts.length - 1] || downloadedFilePath.value;
});
</script>

<template>
  <div class="space-y-2">
    <SettingsGroup title="存储位置" description="设置下载和临时文件的保存位置">
      <SettingPath
        :model-value="settings.general.saveDir"
        label="下载目录"
        placeholder="./downloads"
        @update:model-value="updateGeneral({ saveDir: $event })"
        @select="updateGeneral({ saveDir: $event })"
      />
      <SettingPath
        :model-value="settings.general.tmpDir"
        label="临时目录"
        placeholder="./temp"
        @update:model-value="updateGeneral({ tmpDir: $event })"
        @select="updateGeneral({ tmpDir: $event })"
      />
    </SettingsGroup>

    <SettingsGroup title="应用行为" description="配置应用程序的默认行为">
      <SettingSelect
        :model-value="settings.general.language"
        label="语言"
        :options="languageOptions"
        placeholder="选择语言"
        @update:model-value="updateGeneral({ language: $event })"
      />

      <SettingSwitch
        :model-value="settings.general.autoStartDownload"
        label="自动开始下载"
        description="添加任务后自动开始下载"
        @update:model-value="updateGeneral({ autoStartDownload: $event })"
      />

      <SettingSwitch
        :model-value="settings.general.minimizeToTray"
        label="最小化到托盘"
        description="关闭窗口时最小化到系统托盘"
        @update:model-value="updateGeneral({ minimizeToTray: $event })"
      />

      <SettingSwitch
        :model-value="settings.general.checkUpdate"
        label="检查更新"
        description="启动时自动检查新版本"
        @update:model-value="updateGeneral({ checkUpdate: $event })"
      />
    </SettingsGroup>

    <SettingsGroup title="版本信息">
      <!-- 下载进度/完成区域 -->
      <div
        v-if="isDownloading || isDownloaded"
        class="mb-4 rounded-lg bg-muted/50 p-4"
      >
        <div class="mb-2 flex items-center justify-between">
          <span class="text-sm font-medium">
            {{ isDownloaded ? "下载完成" : "正在下载更新..." }}
          </span>
          <span class="text-sm text-muted-foreground">{{ latestVersion }}</span>
        </div>

        <!-- 进度条 -->
        <Progress
          v-if="isDownloading"
          :model-value="downloadProgress"
          class="mb-2 h-2"
        />

        <div
          v-if="isDownloading"
          class="flex items-center justify-between text-xs text-muted-foreground"
        >
          <span>{{ progressText }}</span>
        </div>

        <!-- 下载完成提示 -->
        <div v-if="isDownloaded" class="space-y-3">
          <div class="flex items-center gap-2 text-sm text-primary">
            <AppIcon name="CheckCircle" :size="16" />
            <span>安装程序已下载并启动</span>
          </div>

          <!-- 文件路径和操作按钮 -->
          <div
            class="flex items-center justify-between rounded bg-background/50 p-2"
          >
            <div class="flex items-center gap-2 text-xs text-muted-foreground">
              <AppIcon name="FileDown" :size="14" />
              <span class="truncate" :title="downloadedFilePath || undefined">
                {{ displayFileName }}
              </span>
            </div>
            <div class="flex items-center gap-1">
              <Button
                variant="ghost"
                size="sm"
                class="h-6 px-2 text-xs"
                @click="openDownloadLocation"
              >
                <AppIcon name="FolderOpen" :size="12" class="mr-1" />
                打开位置
              </Button>
              <Button
                variant="ghost"
                size="sm"
                class="h-6 px-2 text-xs"
                @click="runInstallerAgain"
              >
                <AppIcon name="Play" :size="12" class="mr-1" />
                运行
              </Button>
            </div>
          </div>
        </div>
      </div>

      <div class="flex items-center justify-between">
        <div class="flex items-center gap-4">
          <div class="flex items-center gap-2">
            <span class="text-sm text-muted-foreground">当前版本</span>
            <span class="font-mono text-sm">{{ currentVersion }}</span>
          </div>
          <div
            v-if="updateAvailable && latestVersion"
            class="flex items-center gap-1.5"
          >
            <span
              class="inline-flex h-2 w-2 rounded-full bg-primary animate-pulse"
            />
            <span class="text-sm text-primary"
              >有新版本 {{ latestVersion }}</span
            >
          </div>
        </div>
        <div class="flex items-center gap-2">
          <!-- 下载更新按钮 -->
          <template v-if="updateAvailable">
            <Button
              v-if="isDownloading"
              variant="destructive"
              size="sm"
              @click="handleCancelDownload"
            >
              <AppIcon name="X" :size="16" class="mr-2" />
              取消
            </Button>
            <Button
              v-else-if="selectedAsset"
              variant="default"
              size="sm"
              @click="handleDownloadUpdate"
            >
              <AppIcon name="Download" :size="16" class="mr-2" />
              下载更新
            </Button>
            <Button
              v-else
              variant="default"
              size="sm"
              @click="openDownloadPage"
            >
              <AppIcon name="ExternalLink" :size="16" class="mr-2" />
              前往下载
            </Button>
          </template>

          <!-- 检查更新按钮 -->
          <Button
            variant="outline"
            size="sm"
            :disabled="isChecking || isDownloading"
            @click="handleCheckUpdate"
          >
            <AppIcon
              v-if="isChecking"
              name="Loader2"
              :size="16"
              class="mr-2 animate-spin"
            />
            <AppIcon v-else name="RefreshCw" :size="16" class="mr-2" />
            {{ isChecking ? "检查中..." : "检查更新" }}
          </Button>
        </div>
      </div>

      <!-- 资产信息 -->
      <div
        v-if="
          updateAvailable && selectedAsset && !isDownloading && !isDownloaded
        "
        class="mt-3 rounded-md bg-muted/30 p-3 text-xs text-muted-foreground"
      >
        <div class="flex items-center gap-2">
          <AppIcon name="Package" :size="14" />
          <span>
            检测到适合的安装包: {{ selectedAsset.name }}
            <span class="ml-1">({{ formatFileSize(selectedAsset.size) }})</span>
          </span>
        </div>
      </div>
    </SettingsGroup>
  </div>
</template>
