/**
 * 更新检查组合式函数
 * 检查 GitHub Releases 获取最新版本
 */

import { ref, onMounted } from "vue";
import { useSettingsStore } from "@/stores";
import { useToast } from "./useToast";

// 当前应用版本（从 package.json 读取）
const CURRENT_VERSION = __APP_VERSION__;

// GitHub 仓库信息
const GITHUB_REPO = "iibyteCoder/StreamGrab";

// 更新信息接口
interface ReleaseInfo {
  version: string;
  url: string;
  notes?: string;
}

/**
 * 比较语义化版本号
 * @returns 1 if v1 > v2, -1 if v1 < v2, 0 if equal
 */
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

/**
 * 更新检查组合式函数
 */
export function useUpdateChecker() {
  const settingsStore = useSettingsStore();
  const toast = useToast();

  // 状态
  const isChecking = ref(false);
  const lastCheckTime = ref<Date | null>(null);
  const latestVersion = ref<string | null>(null);
  const updateAvailable = ref(false);
  const releaseUrl = ref<string | null>(null);

  // 检查间隔（24小时）
  const CHECK_INTERVAL = 24 * 60 * 60 * 1000;

  /**
   * 从 GitHub API 获取最新版本
   */
  async function fetchLatestVersion(): Promise<ReleaseInfo | null> {
    try {
      const response = await fetch(
        `https://api.github.com/repos/${GITHUB_REPO}/releases/latest`,
        {
          headers: {
            Accept: "application/vnd.github.v3+json",
          },
        },
      );

      if (!response.ok) {
        throw new Error(`GitHub API error: ${response.status}`);
      }

      const data = await response.json();

      return {
        version: data.tag_name || data.name,
        url: data.html_url,
        notes: data.body,
      };
    } catch (e) {
      console.error("Failed to fetch latest version:", e);
      return null;
    }
  }

  /**
   * 检查更新
   */
  async function checkForUpdate(showNoUpdateToast = false): Promise<boolean> {
    if (isChecking.value) return false;

    isChecking.value = true;

    try {
      const release = await fetchLatestVersion();

      if (!release) {
        if (showNoUpdateToast) {
          toast.warning("检查更新失败，请稍后重试");
        }
        return false;
      }

      latestVersion.value = release.version;
      releaseUrl.value = release.url;
      lastCheckTime.value = new Date();

      // 比较版本
      const comparison = compareVersions(release.version, CURRENT_VERSION);
      updateAvailable.value = comparison > 0;

      if (updateAvailable.value) {
        // 有新版本
        toast.success(`发现新版本 ${release.version}，请前往 GitHub 下载`, {
          duration: 5000,
        });
      } else if (showNoUpdateToast) {
        toast.success("当前已是最新版本");
      }

      return updateAvailable.value;
    } catch (e) {
      console.error("Update check failed:", e);
      if (showNoUpdateToast) {
        toast.error("检查更新失败");
      }
      return false;
    } finally {
      isChecking.value = false;
    }
  }

  /**
   * 打开下载页面
   */
  function openDownloadPage() {
    if (releaseUrl.value) {
      window.open(releaseUrl.value, "_blank");
    } else {
      window.open(`https://github.com/${GITHUB_REPO}/releases`, "_blank");
    }
  }

  /**
   * 自动检查更新（如果设置启用且距离上次检查超过间隔）
   */
  async function autoCheckIfNeeded() {
    if (!settingsStore.settings.general.checkUpdate) return;

    // 检查是否需要检查（距离上次检查超过24小时）
    const lastCheck = lastCheckTime.value;
    if (lastCheck && Date.now() - lastCheck.getTime() < CHECK_INTERVAL) {
      return;
    }

    await checkForUpdate(false);
  }

  // 组件挂载时自动检查
  onMounted(() => {
    autoCheckIfNeeded();
  });

  return {
    isChecking,
    lastCheckTime,
    latestVersion,
    currentVersion: CURRENT_VERSION,
    updateAvailable,
    releaseUrl,
    checkForUpdate,
    openDownloadPage,
  };
}
