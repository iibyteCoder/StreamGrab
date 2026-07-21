/**
 * 应用更新服务
 *
 * GitHub API 版本检查（带重试/限流处理）、semver 比较、平台资产选择；
 * 下载/安装走 systemService。composable 层不再有任何直接 invoke 或 fetch。
 */

import {
  systemService,
  type AppDownloadProgress,
  type UnlistenFn,
} from "@/services";

// ========================================
// 类型
// ========================================

export interface ReleaseInfo {
  version: string;
  url: string;
  notes?: string;
  assets: ReleaseAsset[];
}

export interface ReleaseAsset {
  name: string;
  browser_download_url: string;
  content_type: string;
  size: number;
}

export interface UpdateError {
  type: "network" | "api" | "parse" | "unknown";
  message: string;
}

type Platform = "windows" | "macos" | "linux";

// ========================================
// 工具函数
// ========================================

function getPlatform(): Platform {
  if (navigator.platform.toLowerCase().includes("win")) return "windows";
  if (navigator.platform.toLowerCase().includes("mac")) return "macos";
  return "linux";
}

function getPlatformAsset(assets: ReleaseAsset[]): ReleaseAsset | null {
  const platform = getPlatform();

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

  if (platform === "macos") {
    const dmg = assets.find((a) => a.name.endsWith(".dmg"));
    if (dmg) return dmg;
    const armApp = assets.find(
      (a) => a.name.includes("aarch64") || a.name.includes("arm64"),
    );
    if (armApp) return armApp;
  }

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

/** 比较语义化版本号：1 = v1 > v2，-1 = v1 < v2，0 = 相等 */
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

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024)
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

// ========================================
// 服务类
// ========================================

class UpdateService {
  private currentVersion: string;
  private githubRepo = "iibyteCoder/StreamGrab";

  constructor() {
    this.currentVersion =
      typeof __APP_VERSION__ !== "undefined" ? __APP_VERSION__ : "0.0.0";
  }

  getCurrentVersion(): string {
    return this.currentVersion;
  }

  /** 从 GitHub API 获取最新版本（带重试） */
  async fetchLatestVersion(
    retries = 2,
  ): Promise<{ release: ReleaseInfo; asset: ReleaseAsset | null } | null> {
    const url = `https://api.github.com/repos/${this.githubRepo}/releases/latest`;

    for (let attempt = 0; attempt <= retries; attempt++) {
      try {
        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), 10000);

        const response = await fetch(url, {
          headers: {
            Accept: "application/vnd.github+json",
            "X-GitHub-Api-Version": "2022-11-28",
            "User-Agent": "StreamGrab-Update-Checker",
          },
          signal: controller.signal,
        });

        clearTimeout(timeoutId);

        if (response.status === 403) {
          return null;
        }

        if (response.status === 404) {
          return null;
        }

        if (!response.ok) {
          throw new Error(`HTTP error: ${response.status}`);
        }

        const data = await response.json();

        if (!data.tag_name && !data.name) {
          return null;
        }

        const release: ReleaseInfo = {
          version: data.tag_name || data.name,
          url: data.html_url,
          notes: data.body,
          assets: data.assets || [],
        };

        const asset = getPlatformAsset(release.assets);
        return { release, asset };
      } catch (e) {
        if (e instanceof Error && e.name === "AbortError") {
          return null;
        }
        if (attempt < retries) {
          await new Promise((resolve) => setTimeout(resolve, 1000));
          continue;
        }
        return null;
      }
    }

    return null;
  }

  /** 下载更新安装包 */
  async downloadUpdate(downloadUrl: string, savePath: string): Promise<string> {
    return systemService.downloadAppUpdate(downloadUrl, savePath);
  }

  /** 运行安装程序 */
  async runInstaller(installerPath: string): Promise<void> {
    return systemService.runInstaller(installerPath);
  }

  /** 订阅更新下载进度 */
  async subscribeToProgress(
    handler: (progress: AppDownloadProgress) => void,
  ): Promise<UnlistenFn> {
    return systemService.subscribeToUpdateProgress(handler);
  }

  /** 比较版本：返回 true 表示 remote 比 local 新 */
  isNewerThanCurrent(remoteVersion: string): boolean {
    return compareVersions(remoteVersion, this.currentVersion) > 0;
  }

  formatFileSize(bytes: number): string {
    return formatFileSize(bytes);
  }

  getPlatform(): Platform {
    return getPlatform();
  }
}

export const updateService = new UpdateService();
