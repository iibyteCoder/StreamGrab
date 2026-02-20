/**
 * 更新检查组合式函数
 * 检查 GitHub Releases 获取最新版本
 */

import { ref, onMounted } from "vue";
import { openUrl } from "@tauri-apps/plugin-opener";
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

// 更新错误类型
interface UpdateError {
  type: "network" | "api" | "parse" | "unknown";
  message: string;
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
  const lastError = ref<UpdateError | null>(null);

  // 检查间隔（24小时）
  const CHECK_INTERVAL = 24 * 60 * 60 * 1000;

  /**
   * 从 GitHub API 获取最新版本（带重试机制）
   */
  async function fetchLatestVersion(retries = 2): Promise<ReleaseInfo | null> {
    const url = `https://api.github.com/repos/${GITHUB_REPO}/releases/latest`;

    console.log("[UpdateChecker] 开始检查更新");
    console.log("[UpdateChecker] API URL:", url);
    console.log("[UpdateChecker] 当前版本:", CURRENT_VERSION);

    for (let attempt = 0; attempt <= retries; attempt++) {
      try {
        console.log(`[UpdateChecker] 第 ${attempt + 1} 次尝试请求...`);

        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), 10000); // 10秒超时

        const response = await fetch(url, {
          headers: {
            Accept: "application/vnd.github+json",
            "X-GitHub-Api-Version": "2022-11-28",
            "User-Agent": "StreamGrab-Update-Checker",
          },
          signal: controller.signal,
        });

        clearTimeout(timeoutId);

        console.log(
          "[UpdateChecker] 响应状态:",
          response.status,
          response.statusText,
        );

        if (response.status === 403) {
          // GitHub API 限流
          console.error("[UpdateChecker] GitHub API 限流 (403)");
          lastError.value = {
            type: "api",
            message: "GitHub API 请求频率受限，请稍后再试",
          };
          return null;
        }

        if (response.status === 404) {
          // 仓库没有已发布的正式版本（非草稿、非预发布）
          console.error("[UpdateChecker] 未找到发布版本 (404)");
          lastError.value = {
            type: "api",
            message: "暂无发布版本，请前往 GitHub 查看",
          };
          return null;
        }

        if (!response.ok) {
          throw new Error(`HTTP error: ${response.status}`);
        }

        const data = await response.json();
        console.log("[UpdateChecker] API 响应数据:", {
          tag_name: data.tag_name,
          name: data.name,
          html_url: data.html_url,
          published_at: data.published_at,
          draft: data.draft,
          prerelease: data.prerelease,
        });

        // 验证返回数据
        if (!data.tag_name && !data.name) {
          console.error("[UpdateChecker] 无法解析版本信息，响应数据:", data);
          lastError.value = {
            type: "parse",
            message: "无法解析版本信息",
          };
          return null;
        }

        const releaseInfo: ReleaseInfo = {
          version: data.tag_name || data.name,
          url: data.html_url,
          notes: data.body,
        };

        console.log("[UpdateChecker] 解析成功，最新版本:", releaseInfo.version);
        console.log("[UpdateChecker] 发布页面:", releaseInfo.url);

        lastError.value = null;
        return releaseInfo;
      } catch (e) {
        if (e instanceof Error && e.name === "AbortError") {
          console.error("[UpdateChecker] 请求超时");
          lastError.value = {
            type: "network",
            message: "请求超时，请检查网络连接",
          };
        } else if (e instanceof TypeError && e.message.includes("fetch")) {
          console.error("[UpdateChecker] 网络连接失败:", e.message);
          lastError.value = {
            type: "network",
            message: "网络连接失败，请检查网络设置",
          };
        } else if (attempt < retries) {
          console.warn(
            `[UpdateChecker] 第 ${attempt + 1} 次尝试失败，1秒后重试...`,
          );
          // 重试前等待
          await new Promise((resolve) => setTimeout(resolve, 1000));
          continue;
        } else {
          console.error(`[UpdateChecker] 所有重试失败:`, e);
          lastError.value = {
            type: "unknown",
            message: e instanceof Error ? e.message : "未知错误",
          };
        }
      }
    }

    return null;
  }

  /**
   * 检查更新
   */
  async function checkForUpdate(showNoUpdateToast = false): Promise<boolean> {
    if (isChecking.value) {
      console.log("[UpdateChecker] 已有检查任务进行中，跳过");
      return false;
    }

    isChecking.value = true;
    console.log("[UpdateChecker] ========== 开始检查更新 ==========");

    try {
      const release = await fetchLatestVersion();

      if (!release) {
        console.error("[UpdateChecker] 获取版本信息失败");
        if (showNoUpdateToast) {
          const currentError = lastError.value;
          const errorMsg =
            currentError && "message" in currentError
              ? currentError.message
              : "检查更新失败，请稍后重试";
          toast.warning(errorMsg);
        }
        return false;
      }

      latestVersion.value = release.version;
      releaseUrl.value = release.url;
      lastCheckTime.value = new Date();

      // 比较版本
      const comparison = compareVersions(release.version, CURRENT_VERSION);
      updateAvailable.value = comparison > 0;

      console.log("[UpdateChecker] 版本比较结果:");
      console.log("  - 当前版本:", CURRENT_VERSION);
      console.log("  - 最新版本:", release.version);
      console.log(
        "  - 比较结果:",
        comparison > 0
          ? "有更新"
          : comparison < 0
            ? "当前版本更高"
            : "版本相同",
      );
      console.log("[UpdateChecker] ========== 检查完成 ==========");

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
      console.error("[UpdateChecker] 检查更新异常:", e);
      if (showNoUpdateToast) {
        toast.error("检查更新失败");
      }
      return false;
    } finally {
      isChecking.value = false;
    }
  }

  /**
   * 打开下载页面（使用 Tauri shell）
   */
  async function openDownloadPage() {
    const url =
      releaseUrl.value || `https://github.com/${GITHUB_REPO}/releases`;
    try {
      await openUrl(url);
    } catch (e) {
      console.error("Failed to open URL:", e);
      // 降级到 window.open
      window.open(url, "_blank");
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
    lastError,
    checkForUpdate,
    openDownloadPage,
  };
}
