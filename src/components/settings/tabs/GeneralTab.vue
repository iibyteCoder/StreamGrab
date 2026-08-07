<script setup lang="ts">
/**
 * GeneralTab - 常规·界面 标签页
 *
 * 合并旧 GeneralSettings + UISettings + AdvancedSettings（非工具部分）+ 应用更新。
 * 全部经 settingsStore.updateAppSettings(partial)。
 */

import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { useSettingsStore } from "@/stores";
import { useToast } from "@/composables";
import { useUpdateChecker } from "@/composables";
import {
  SettingSwitch,
  SettingSelect,
  SettingInput,
  SettingPath,
  SettingSlider,
  SettingsGroup,
} from "..";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { AppIcon } from "@/components/common";
import { systemService } from "@/services";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog";
import type { Theme, Language, LogLevel } from "@/domain";

const { t } = useI18n();
const settingsStore = useSettingsStore();
const toast = useToast();

// ========================================
// 应用更新
// ========================================

const {
  isChecking,
  updateAvailable,
  currentVersion,
  latestVersion,
  releaseNotes,
  selectedAsset,
  downloadStatus,
  downloadProgress,
  downloadedSize,
  totalSize,
  downloadedFilePath,
  checkForUpdate,
  downloadUpdate,
  cancelDownload,
  openDownloadPage,
  openDownloadLocation,
  runInstallerAgain,
  formatFileSize,
} = useUpdateChecker();

const isDownloading = computed(() => downloadStatus.value === "downloading");
const isDownloaded = computed(() => downloadStatus.value === "downloaded");

const progressText = computed(() => {
  if (totalSize.value > 0) {
    return `${formatFileSize(downloadedSize.value)} / ${formatFileSize(totalSize.value)}`;
  }
  return formatFileSize(downloadedSize.value);
});

const displayFileName = computed(() => {
  if (!downloadedFilePath.value) return "";
  const parts = downloadedFilePath.value.replace(/\\/g, "/").split("/");
  return parts[parts.length - 1] || downloadedFilePath.value;
});

// ========================================
// 选项
// ========================================

const languageOptions = [
  { value: "zh-CN", label: "简体中文" },
  { value: "zh-TW", label: "繁体中文" },
  { value: "en-US", label: "English" },
];

const themeOptions = computed(() => [
  { value: "light", label: t("settings.ui.themeLight") },
  { value: "dark", label: t("settings.ui.themeDark") },
  { value: "system", label: t("settings.ui.themeSystem") },
]);

const logLevelOptions = [
  { value: "DEBUG", label: "DEBUG" },
  { value: "INFO", label: "INFO" },
  { value: "WARN", label: "WARN" },
  { value: "ERROR", label: "ERROR" },
  { value: "OFF", label: "OFF" },
];

// ========================================
// 导入/导出
// ========================================

async function handleExport() {
  try {
    await settingsStore.exportConfig();
    toast.success(t("common.success"));
  } catch {
    toast.error(t("common.error"));
  }
}

async function handleImport() {
  try {
    const filePath = await systemService.selectFile([
      { name: "JSON", extensions: ["json"] },
    ]);
    if (filePath) {
      await settingsStore.importConfig(filePath);
      toast.success(t("common.success"));
    }
  } catch {
    toast.error(t("common.error"));
  }
}

// ========================================
// 重置
// ========================================

async function handleReset() {
  try {
    await settingsStore.resetSettings();
    toast.success(t("messages.settingsReset"));
  } catch {
    toast.error(t("common.error"));
  }
}

// ========================================
// 更新操作
// ========================================

async function handleCheckUpdate() {
  await checkForUpdate(true);
}

async function handleDownloadUpdate() {
  await downloadUpdate();
}

function handleCancelDownload() {
  cancelDownload();
}
</script>

<template>
  <div class="space-y-6">
    <!-- 语言与外观 -->
    <SettingsGroup :title="t('settings.general.langAppearance', '语言与外观')">
      <SettingSelect
        :model-value="settingsStore.appSettings.language"
        :label="t('settings.general.language')"
        :options="languageOptions"
        :placeholder="t('settings.general.language')"
        @update:model-value="
          settingsStore.updateAppSettings({
            language: $event as Language,
          })
        "
      />

      <SettingSelect
        :model-value="settingsStore.appSettings.theme"
        :label="t('settings.ui.theme')"
        :options="themeOptions"
        :placeholder="t('settings.ui.theme')"
        @update:model-value="
          settingsStore.updateAppSettings({ theme: $event as Theme })
        "
      />
    </SettingsGroup>

    <!-- 存储位置 -->
    <SettingsGroup
      :title="t('settings.general.storage')"
      :description="t('settings.general.storageDesc')"
    >
      <SettingPath
        :model-value="settingsStore.appSettings.default_save_dir"
        :label="t('settings.general.saveDir')"
        placeholder="./downloads"
        @update:model-value="
          settingsStore.updateAppSettings({
            default_save_dir: String($event),
          })
        "
        @select="settingsStore.updateAppSettings({ default_save_dir: $event })"
      />

      <SettingPath
        :model-value="settingsStore.appSettings.default_tmp_dir"
        :label="t('settings.general.tmpDir')"
        placeholder="./temp"
        @update:model-value="
          settingsStore.updateAppSettings({
            default_tmp_dir: String($event),
          })
        "
        @select="settingsStore.updateAppSettings({ default_tmp_dir: $event })"
      />
    </SettingsGroup>

    <!-- 应用行为 -->
    <SettingsGroup
      :title="t('settings.general.behavior')"
      :description="t('settings.general.behaviorDesc')"
    >
      <SettingSwitch
        :model-value="settingsStore.appSettings.show_notification"
        :label="t('settings.ui.notification')"
        :description="t('settings.ui.notificationDesc')"
        @update:model-value="
          settingsStore.updateAppSettings({ show_notification: $event })
        "
      />

      <SettingSwitch
        :model-value="settingsStore.appSettings.clipboard_watch"
        :label="t('settings.ui.clipboardWatch')"
        :description="t('settings.ui.clipboardWatchDesc')"
        @update:model-value="
          settingsStore.updateAppSettings({ clipboard_watch: $event })
        "
      />

      <SettingSwitch
        :model-value="settingsStore.appSettings.minimize_to_tray"
        :label="t('settings.general.minimizeToTray')"
        :description="t('settings.general.minimizeToTrayDesc')"
        @update:model-value="
          settingsStore.updateAppSettings({ minimize_to_tray: $event })
        "
      />

      <SettingSwitch
        :model-value="settingsStore.appSettings.check_update"
        :label="t('settings.general.checkUpdate')"
        :description="t('settings.general.checkUpdateDesc')"
        @update:model-value="
          settingsStore.updateAppSettings({ check_update: $event })
        "
      />

      <SettingSwitch
        :model-value="settingsStore.appSettings.auto_start_download"
        :label="t('settings.general.autoStartDownload')"
        :description="t('settings.general.autoStartDownloadDesc')"
        @update:model-value="
          settingsStore.updateAppSettings({ auto_start_download: $event })
        "
      />

      <SettingSlider
        :model-value="settingsStore.appSettings.max_concurrent_tasks"
        :label="t('settings.general.maxConcurrentTasks', '最大并发任务数')"
        :description="
          t('settings.general.maxConcurrentTasksDesc', '同时进行的下载任务上限')
        "
        :min="1"
        :max="20"
        :step="1"
        @update:model-value="
          settingsStore.updateAppSettings({ max_concurrent_tasks: $event })
        "
      />
    </SettingsGroup>

    <!-- 日志设置 -->
    <SettingsGroup
      :title="t('settings.general.logSettings', '日志设置')"
      :description="t('settings.general.logSettingsDesc', '配置日志级别和输出')"
    >
      <SettingSelect
        :model-value="settingsStore.appSettings.log_level"
        :label="t('settings.general.logLevel', '日志级别')"
        :options="logLevelOptions"
        :placeholder="t('settings.general.logLevel', '日志级别')"
        @update:model-value="
          settingsStore.updateAppSettings({ log_level: $event as LogLevel })
        "
      />

      <SettingInput
        :model-value="settingsStore.appSettings.log_file_path"
        :label="t('settings.general.logFilePath', '日志文件路径')"
        :placeholder="
          t('settings.general.logFilePathPlaceholder', '留空则不写入文件')
        "
        @update:model-value="
          settingsStore.updateAppSettings({
            log_file_path: String($event),
          })
        "
      />

      <SettingSwitch
        :model-value="settingsStore.appSettings.no_log"
        :label="t('settings.general.disableLog', '禁用日志')"
        @update:model-value="
          settingsStore.updateAppSettings({ no_log: $event })
        "
      />
    </SettingsGroup>

    <!-- 应用更新 -->
    <SettingsGroup
      :title="t('settings.general.update.title', '应用更新')"
      :description="t('settings.general.update.desc', '检查并安装应用程序更新')"
    >
      <!-- 下载进度/完成区域 -->
      <div v-if="isDownloading || isDownloaded" class="px-5 py-4">
        <div class="rounded-lg bg-muted/40 p-4">
          <div class="mb-2 flex items-center justify-between">
            <span class="text-sm font-medium text-foreground">
              {{
                isDownloaded
                  ? t("settings.general.update.downloadComplete", "下载完成")
                  : t("settings.general.update.downloading", "正在下载更新...")
              }}
            </span>
            <span class="font-mono text-sm text-muted-foreground">
              {{ latestVersion }}
            </span>
          </div>

          <Progress
            v-if="isDownloading"
            :model-value="downloadProgress"
            class="mb-2 h-2"
          />

          <div
            v-if="isDownloading"
            class="flex items-center justify-between text-xs text-muted-foreground"
          >
            <span class="font-mono">{{ progressText }}</span>
          </div>

          <!-- 下载完成 -->
          <div v-if="isDownloaded" class="space-y-3">
            <div
              class="flex items-center gap-2 text-sm text-[var(--accent-primary)]"
            >
              <AppIcon name="CheckCircle" :size="16" />
              <span>{{
                t("settings.general.update.installerReady", "安装程序已下载")
              }}</span>
            </div>

            <div
              class="flex items-center justify-between rounded-md bg-muted/40 p-2"
            >
              <div
                class="flex min-w-0 items-center gap-2 text-xs text-muted-foreground"
              >
                <AppIcon name="FileDown" :size="14" class="shrink-0" />
                <span
                  class="truncate font-mono"
                  :title="downloadedFilePath || undefined"
                >
                  {{ displayFileName }}
                </span>
              </div>
              <div class="flex shrink-0 items-center gap-1">
                <Button
                  variant="ghost"
                  size="sm"
                  class="h-6 cursor-pointer px-2 text-xs"
                  @click="openDownloadLocation"
                >
                  <AppIcon name="FolderOpen" :size="12" class="mr-1" />
                  {{ t("settings.general.update.openLocation", "打开位置") }}
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  class="h-6 cursor-pointer px-2 text-xs"
                  @click="runInstallerAgain"
                >
                  <AppIcon name="Play" :size="12" class="mr-1" />
                  {{ t("settings.general.update.runInstaller", "运行") }}
                </Button>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 当前版本 + 检查更新 -->
      <div class="flex items-center justify-between gap-4 px-5 py-4">
        <div class="flex min-w-0 items-center gap-4">
          <div class="flex shrink-0 items-center gap-2">
            <span class="text-sm text-muted-foreground">
              {{ t("settings.general.currentVersion") }}
            </span>
            <span class="font-mono text-sm text-foreground">
              {{ currentVersion }}
            </span>
          </div>
          <div
            v-if="updateAvailable && latestVersion"
            class="flex min-w-0 items-center gap-1.5"
          >
            <span
              class="inline-flex h-2 w-2 shrink-0 animate-pulse rounded-full bg-[var(--accent-primary)]"
            />
            <span class="truncate text-sm text-[var(--accent-primary)]">
              {{
                t("messages.updateAvailable", "发现新版本").replace(
                  "{version}",
                  latestVersion,
                )
              }}
            </span>
          </div>
        </div>
        <div class="flex shrink-0 items-center gap-2">
          <template v-if="updateAvailable">
            <Button
              v-if="isDownloading"
              variant="destructive"
              size="sm"
              class="cursor-pointer"
              @click="handleCancelDownload"
            >
              <AppIcon name="X" :size="16" class="mr-2" />
              {{ t("common.cancel", "取消") }}
            </Button>
            <Button
              v-else-if="selectedAsset"
              variant="default"
              size="sm"
              class="cursor-pointer"
              @click="handleDownloadUpdate"
            >
              <AppIcon name="Download" :size="16" class="mr-2" />
              {{ t("settings.general.downloadUpdate") }}
            </Button>
            <Button
              v-else
              variant="default"
              size="sm"
              class="cursor-pointer"
              @click="openDownloadPage"
            >
              <AppIcon name="ExternalLink" :size="16" class="mr-2" />
              {{ t("settings.general.update.goToDownload", "前往下载") }}
            </Button>
          </template>

          <Button
            variant="outline"
            size="sm"
            class="cursor-pointer"
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
            {{
              isChecking
                ? t("settings.general.checking")
                : t("settings.general.checkNow")
            }}
          </Button>
        </div>
      </div>

      <!-- 更新说明 -->
      <div
        v-if="
          updateAvailable && releaseNotes && !isDownloading && !isDownloaded
        "
        class="px-5 py-4"
      >
        <p
          class="line-clamp-3 rounded-lg border border-border/60 bg-muted/30 p-3 text-xs leading-relaxed text-muted-foreground"
        >
          {{ releaseNotes }}
        </p>
      </div>

      <!-- 资产信息 -->
      <div
        v-if="
          updateAvailable && selectedAsset && !isDownloading && !isDownloaded
        "
        class="px-5 py-4"
      >
        <div
          class="flex items-center gap-2 rounded-lg border border-border/60 bg-muted/30 p-3 text-xs text-muted-foreground"
        >
          <AppIcon name="Package" :size="14" class="shrink-0" />
          <span>
            {{ t("settings.general.update.detectedAsset", "检测到安装包") }}:
            {{ selectedAsset.name }}
            <span class="ml-1 font-mono">
              ({{ formatFileSize(selectedAsset.size) }})
            </span>
          </span>
        </div>
      </div>
    </SettingsGroup>

    <!-- 导入/导出/重置 -->
    <div
      class="flex items-center justify-between rounded-xl border border-border/60 bg-card/60 px-5 py-4"
    >
      <div class="flex items-center gap-2">
        <Button
          variant="outline"
          size="sm"
          class="cursor-pointer"
          @click="handleExport"
        >
          <AppIcon name="Upload" :size="14" class="mr-1.5" />
          {{ t("settings.general.exportConfig", "导出配置") }}
        </Button>
        <Button
          variant="outline"
          size="sm"
          class="cursor-pointer"
          @click="handleImport"
        >
          <AppIcon name="Download" :size="14" class="mr-1.5" />
          {{ t("settings.general.importConfig", "导入配置") }}
        </Button>
      </div>

      <!-- 重置确认弹窗 -->
      <AlertDialog>
        <AlertDialogTrigger as-child>
          <Button
            variant="outline"
            size="sm"
            class="cursor-pointer border-destructive/40 text-destructive hover:bg-destructive/10 hover:text-destructive"
          >
            <AppIcon name="RotateCcw" :size="14" class="mr-1.5" />
            {{ t("settings.advanced.reset") }}
          </Button>
        </AlertDialogTrigger>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>
              {{
                t("settings.general.confirmResetTitle", "确认恢复默认配置？")
              }}
            </AlertDialogTitle>
            <AlertDialogDescription>
              {{
                t(
                  "settings.general.confirmResetDesc",
                  "此操作将把所有设置恢复为默认值，当前配置将被覆盖。此操作无法撤销。",
                )
              }}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{{ t("common.cancel") }}</AlertDialogCancel>
            <AlertDialogAction @click="handleReset">
              {{ t("common.confirm") }}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  </div>
</template>
