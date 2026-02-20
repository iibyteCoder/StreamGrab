/**
 * 更新检查组合式函数
 * 支持检查、下载和安装更新
 */

import { ref, onMounted, onUnmounted } from "vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
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
  assets: ReleaseAsset[];
}

// Release 资产接口
interface ReleaseAsset {
  name: string;
  browser_download_url: string;
  content_type: string;
  size: number;
}

// 更新错误类型
interface UpdateError {
  type: "network" | "api" | "parse" | "unknown";
  message: string;
}

// 下载状态
type DownloadStatus = "idle" | "downloading" | "downloaded" | "error";

// 平台类型
type Platform = "windows" | "macos" | "linux";

// 下载进度事件
interface DownloadProgressEvent {
  status: string;
  downloaded: number;
  total: number;
  percent: number;
}

/**
 * 获取当前平台
 */
function getPlatform(): Platform {
  if (navigator.platform.toLowerCase().includes("win")) return "windows";
  if (navigator.platform.toLowerCase().includes("mac")) return "macos";
  return "linux";
}

/**
 * 获取适合当前平台的安装包资源
 */
function getPlatformAsset(assets: ReleaseAsset[]): ReleaseAsset | null {
  const platform = getPlatform();

  // Windows: 优先 .exe (NSIS 安装器), 其次 .msi
  if (platform === "windows") {
    const exe = assets.find(
      (a) =>
        a.name.endsWith(".exe") &&
        !a.name.includes("Portable") &&
        !a.name.toLowerCase().includes("setup_x64"),
    );
    if (exe) return exe;

    const msi = assets.find((a) => a.name.endsWith(".msi"));
    if (msi) return msi;
  }

  // macOS: .dmg
  if (platform === "macos") {
    const dmg = assets.find((a) => a.name.endsWith(".dmg"));
    if (dmg) return dmg;

    // Apple Silicon
    const armApp = assets.find(
      (a) => a.name.includes("aarch64") || a.name.includes("arm64"),
    );
    if (armApp) return armApp;
  }

  // Linux: .AppImage, .deb, .rpm
  if (platform === "linux") {
    const appImage = assets.find((a) => a.name.endsWith(".AppImage"));
    if (appImage) return appImage;

    const deb = assets.find((a) => a.name.endsWith(".deb"));
    if (deb) return deb;

    const rpm = assets.find((a) => a.name.endsWith(".rpm"));
    if (rpm) return rpm;
  }

  return null;
}

/**
 * 格式化文件大小
 */
function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024)
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
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
  const releaseNotes = ref<string | null>(null);

  // 下载相关状态
  const downloadStatus = ref<DownloadStatus>("idle");
  const downloadProgress = ref(0);
  const downloadedSize = ref(0);
  const totalSize = ref(0);
  const selectedAsset = ref<ReleaseAsset | null>(null);
  const downloadedFilePath = ref<string | null>(null);

  // 检查间隔（24小时）
  const CHECK_INTERVAL = 24 * 60 * 60 * 1000;

  // 事件监听器清理函数
  let unlistenProgress: UnlistenFn | null = null;
  let unlistenComplete: UnlistenFn | null = null;

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
          assets_count: data.assets?.length || 0,
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
          assets: data.assets || [],
        };

        console.log("[UpdateChecker] 解析成功，最新版本:", releaseInfo.version);
        console.log("[UpdateChecker] 发布页面:", releaseInfo.url);

        // 查找适合当前平台的安装包
        const asset = getPlatformAsset(releaseInfo.assets);
        if (asset) {
          selectedAsset.value = asset;
          totalSize.value = asset.size;
          console.log("[UpdateChecker] 找到适合的安装包:", asset.name);
        } else {
          console.warn("[UpdateChecker] 未找到适合当前平台的安装包");
        }

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
      releaseNotes.value = release.notes || null;
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
        if (selectedAsset.value) {
          toast.success(
            `发现新版本 ${release.version}，点击"下载更新"按钮开始下载`,
            { duration: 5000 },
          );
        } else {
          toast.success(`发现新版本 ${release.version}，请前往 GitHub 下载`, {
            duration: 5000,
          });
        }
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
   * 设置事件监听器
   */
  async function setupEventListeners() {
    // 监听下载进度事件
    unlistenProgress = await listen<DownloadProgressEvent>(
      "app:update:progress",
      (event) => {
        const progress = event.payload;
        downloadStatus.value = progress.status as DownloadStatus;
        downloadProgress.value = progress.percent;
        downloadedSize.value = progress.downloaded;
        totalSize.value = progress.total;
      },
    );

    // 监听下载完成事件
    unlistenComplete = await listen<{ path: string }>(
      "app:update:complete",
      (event) => {
        downloadedFilePath.value = event.payload.path;
        console.log("[UpdateChecker] 下载完成，文件路径:", event.payload.path);
      },
    );
  }

  /**
   * 下载更新
   */
  async function downloadUpdate(): Promise<boolean> {
    if (!selectedAsset.value) {
      toast.error("未找到适合当前平台的安装包");
      return false;
    }

    if (downloadStatus.value === "downloading") {
      console.log("[UpdateChecker] 已有下载任务进行中");
      return false;
    }

    downloadStatus.value = "downloading";
    downloadProgress.value = 0;
    downloadedSize.value = 0;
    downloadedFilePath.value = null;

    console.log("[UpdateChecker] 开始下载更新:", selectedAsset.value.name);
    console.log(
      "[UpdateChecker] 下载地址:",
      selectedAsset.value.browser_download_url,
    );

    try {
      // 使用临时目录存放更新文件
      const { tempDir } = await import("@tauri-apps/api/path");

      // 保存到临时目录
      const savePath = `${await tempDir()}StreamGrab-${latestVersion.value}-${selectedAsset.value.name}`;

      console.log("[UpdateChecker] 保存路径:", savePath);

      // 调用后端命令下载
      const result = await invoke<string>("download_app_update", {
        downloadUrl: selectedAsset.value.browser_download_url,
        savePath,
      });

      downloadStatus.value = "downloaded";
      downloadedFilePath.value = result;
      console.log("[UpdateChecker] 下载完成:", result);

      toast.success("下载完成！正在打开安装程序...");

      // 运行安装程序
      await invoke("run_installer", { installerPath: result });

      return true;
    } catch (e) {
      console.error("[UpdateChecker] 下载失败:", e);

      downloadStatus.value = "error";
      lastError.value = {
        type: "network",
        message: e instanceof Error ? e.message : String(e),
      };
      toast.error(`下载失败: ${e instanceof Error ? e.message : String(e)}`);
      return false;
    }
  }

  /**
   * 打开下载文件所在目录
   */
  async function openDownloadLocation() {
    if (!downloadedFilePath.value) return;

    try {
      await invoke("open_file_in_explorer", {
        filePath: downloadedFilePath.value,
      });
    } catch (e) {
      console.error("[UpdateChecker] 打开目录失败:", e);
      toast.error("打开目录失败");
    }
  }

  /**
   * 重新运行安装程序
   */
  async function runInstallerAgain() {
    if (!downloadedFilePath.value) return;

    try {
      await invoke("run_installer", {
        installerPath: downloadedFilePath.value,
      });
      toast.success("已启动安装程序");
    } catch (e) {
      console.error("[UpdateChecker] 运行安装程序失败:", e);
      toast.error("运行安装程序失败");
    }
  }

  /**
   * 取消下载（实际上后端下载不支持取消，这里只是重置状态）
   */
  function cancelDownload() {
    downloadStatus.value = "idle";
    downloadProgress.value = 0;
    downloadedSize.value = 0;
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

  // 组件挂载时自动检查和设置事件监听
  onMounted(async () => {
    await setupEventListeners();
    autoCheckIfNeeded();
  });

  // 组件卸载时清理
  onUnmounted(() => {
    if (unlistenProgress) {
      unlistenProgress();
    }
    if (unlistenComplete) {
      unlistenComplete();
    }
  });

  return {
    // 基础状态
    isChecking,
    lastCheckTime,
    latestVersion,
    currentVersion: CURRENT_VERSION,
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
    formatFileSize,
    getPlatform,
  };
}
