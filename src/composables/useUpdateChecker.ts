/**
 * 更新检查组合式函数
 *
 * 纯状态与 UI 交互逻辑。版本检查、下载/安装全部委托 updateService。
 * composable 不再有任何直接 invoke 或 fetch。
 */

import { ref, onMounted, onUnmounted } from "vue";
import { useSettingsStore } from "@/stores";
import { useToast } from "./useToast";
import { updateService, type ReleaseAsset, type UpdateError } from "@/services";
import { i18n } from "@/locales";

type DownloadStatus = "idle" | "downloading" | "downloaded" | "error";

/** 检查间隔（24 小时） */
const CHECK_INTERVAL = 24 * 60 * 60 * 1000;

/** 持久化上次检查时间的 key（跨挂载/会话节流，避免重复触发未鉴权请求） */
const LAST_CHECK_KEY = "streamgrab:lastUpdateCheck";

function loadLastCheck(): Date | null {
  try {
    const raw = localStorage.getItem(LAST_CHECK_KEY);
    if (!raw) return null;
    const d = new Date(raw);
    return Number.isNaN(d.getTime()) ? null : d;
  } catch {
    return null;
  }
}

function saveLastCheck(d: Date): void {
  try {
    localStorage.setItem(LAST_CHECK_KEY, d.toISOString());
  } catch {
    // 存储不可用时忽略（隐私模式等）
  }
}

/**
 * 启动时自动检查更新（App.vue 挂载时调用，无组件生命周期依赖）
 *
 * 受设置项 check_update 与 24h 节流双重控制；与 useUpdateChecker 共用
 * localStorage 节流键（streamgrab:lastUpdateCheck），重复触发安全。
 * 静默检查：不弹 toast，发现更新由 useUpdateChecker 在设置页展示。
 */
export async function autoCheckUpdateAtStartup(): Promise<void> {
  try {
    const settingsStore = useSettingsStore();
    if (!settingsStore.appSettings.check_update) return;

    const lastCheck = loadLastCheck();
    if (lastCheck && Date.now() - lastCheck.getTime() < CHECK_INTERVAL) return;

    // 先记录节流时间（含失败），避免网络异常时反复重试
    saveLastCheck(new Date());
    await updateService.fetchLatestVersion();
  } catch (e) {
    console.debug("[UpdateChecker] 启动自动检查失败:", e);
  }
}

export function useUpdateChecker() {
  const settingsStore = useSettingsStore();
  const toast = useToast();

  // ==========================================
  // 状态
  // ==========================================

  const isChecking = ref(false);
  const lastCheckTime = ref<Date | null>(loadLastCheck());
  const latestVersion = ref<string | null>(null);
  const updateAvailable = ref(false);
  const releaseUrl = ref<string | null>(null);
  const lastError = ref<UpdateError | null>(null);
  const releaseNotes = ref<string | null>(null);

  // 下载相关
  const downloadStatus = ref<DownloadStatus>("idle");
  const downloadProgress = ref(0);
  const downloadedSize = ref(0);
  const totalSize = ref(0);
  const selectedAsset = ref<ReleaseAsset | null>(null);
  const downloadedFilePath = ref<string | null>(null);

  const currentVersion = updateService.getCurrentVersion();

  // 事件监听器清理
  let unlistenProgress: (() => void) | null = null;

  // ==========================================
  // 检查更新
  // ==========================================

  async function checkForUpdate(showNoUpdateToast = false): Promise<boolean> {
    if (isChecking.value) return false;

    isChecking.value = true;

    // 记录本次检查时间（含失败），使自动检查跨挂载/会话也按间隔节流
    const now = new Date();
    lastCheckTime.value = now;
    saveLastCheck(now);

    try {
      const result = await updateService.fetchLatestVersion();

      if (!result) {
        if (showNoUpdateToast) {
          toast.warning(
            i18n.global.t("settings.general.update.checkFailedRetry"),
          );
        }
        return false;
      }

      const { release, asset } = result;

      latestVersion.value = release.version;
      releaseUrl.value = release.url;
      releaseNotes.value = release.notes || null;
      lastCheckTime.value = new Date();

      if (asset) {
        selectedAsset.value = asset;
        totalSize.value = asset.size;
      }

      updateAvailable.value = updateService.isNewerThanCurrent(release.version);

      if (updateAvailable.value) {
        if (asset) {
          toast.success(
            i18n.global.t("settings.general.update.availableWithDownload", {
              version: release.version,
            }),
            { duration: 5000 },
          );
        } else {
          toast.success(
            i18n.global.t("messages.updateAvailable", {
              version: release.version,
            }),
            { duration: 5000 },
          );
        }
      } else if (showNoUpdateToast) {
        toast.success(i18n.global.t("messages.noUpdate"));
      }

      return updateAvailable.value;
    } catch (e) {
      console.error("[UpdateChecker] 检查更新异常:", e);
      if (showNoUpdateToast) {
        toast.error(i18n.global.t("messages.updateCheckFailed"));
      }
      return false;
    } finally {
      isChecking.value = false;
    }
  }

  // ==========================================
  // 下载与安装
  // ==========================================

  async function downloadUpdate(): Promise<boolean> {
    if (!selectedAsset.value) {
      toast.error(i18n.global.t("settings.general.update.noAssetFound"));
      return false;
    }

    if (downloadStatus.value === "downloading") return false;

    downloadStatus.value = "downloading";
    downloadProgress.value = 0;
    downloadedSize.value = 0;
    downloadedFilePath.value = null;

    try {
      // 使用临时目录
      const savePath = `StreamGrab-${latestVersion.value}-${selectedAsset.value.name}`;

      const result = await updateService.downloadUpdate(
        selectedAsset.value.browser_download_url,
        savePath,
      );

      downloadStatus.value = "downloaded";
      downloadedFilePath.value = result;

      toast.success(
        `${i18n.global.t("settings.general.update.downloadComplete")}！${i18n.global.t("settings.general.update.installing")}`,
      );

      await updateService.runInstaller(result);
      return true;
    } catch (e) {
      console.error("[UpdateChecker] 下载失败:", e);
      downloadStatus.value = "error";
      lastError.value = {
        type: "network",
        message: e instanceof Error ? e.message : String(e),
      };
      toast.error(
        i18n.global.t("settings.general.update.downloadFailedError", {
          error: e instanceof Error ? e.message : String(e),
        }),
      );
      return false;
    }
  }

  function cancelDownload(): void {
    downloadStatus.value = "idle";
    downloadProgress.value = 0;
    downloadedSize.value = 0;
  }

  async function openDownloadPage(): Promise<void> {
    const url =
      releaseUrl.value || `https://github.com/iibyteCoder/StreamGrab/releases`;
    window.open(url, "_blank");
  }

  async function openDownloadLocation(): Promise<void> {
    if (!downloadedFilePath.value) return;
    try {
      const { systemService } = await import("@/services");
      await systemService.openFileInExplorer(downloadedFilePath.value);
    } catch (e) {
      console.error("Failed to open download location:", e);
      toast.error(i18n.global.t("settings.general.update.openDirFailed"));
    }
  }

  async function runInstallerAgain(): Promise<void> {
    if (!downloadedFilePath.value) return;
    try {
      await updateService.runInstaller(downloadedFilePath.value);
      toast.success(i18n.global.t("settings.general.update.installerStarted"));
    } catch (e) {
      console.error("Failed to run installer:", e);
      toast.error(i18n.global.t("settings.general.update.runInstallerFailed"));
    }
  }

  // ==========================================
  // 自动检查
  // ==========================================

  async function autoCheckIfNeeded(): Promise<void> {
    if (!settingsStore.appSettings.check_update) return;

    const lastCheck = lastCheckTime.value;
    if (lastCheck && Date.now() - lastCheck.getTime() < CHECK_INTERVAL) return;

    await checkForUpdate(false);
  }

  // ==========================================
  // 生命周期
  // ==========================================

  onMounted(async () => {
    unlistenProgress = await updateService.subscribeToProgress((progress) => {
      downloadStatus.value = progress.status as DownloadStatus;
      downloadProgress.value = progress.percent;
      downloadedSize.value = progress.downloaded;
      totalSize.value = progress.total;
    });

    autoCheckIfNeeded();
  });

  onUnmounted(() => {
    if (unlistenProgress) {
      unlistenProgress();
      unlistenProgress = null;
    }
  });

  return {
    // 基础状态
    isChecking,
    lastCheckTime,
    latestVersion,
    currentVersion,
    updateAvailable,
    releaseUrl,
    releaseNotes,
    lastError,

    // 下载相关状态
    downloadStatus,
    downloadProgress,
    downloadedSize,
    totalSize,
    selectedAsset,
    downloadedFilePath,

    // 方法
    checkForUpdate,
    downloadUpdate,
    cancelDownload,
    openDownloadPage,
    openDownloadLocation,
    runInstallerAgain,

    // 工具函数
    formatFileSize: updateService.formatFileSize.bind(updateService),
    getPlatform: updateService.getPlatform.bind(updateService),
  };
}
