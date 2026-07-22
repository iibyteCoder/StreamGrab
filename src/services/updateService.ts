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
import { compareVersions } from "@/utils/version";
import { formatFileSize as _formatFileSize } from "@/utils/format";

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

// ========================================
// 条件请求缓存（ETag / Last-Modified / 结果）
// 作用：未变更时 GitHub 返回 304 不消耗未鉴权配额；限流/离线时降级用缓存
// ========================================

interface CachedRelease {
  etag: string | null;
  lastModified: string | null;
  release: ReleaseInfo;
  asset: ReleaseAsset | null;
}

const RELEASE_CACHE_KEY = "streamgrab:updateReleaseCache";

function loadReleaseCache(): CachedRelease | null {
  try {
    const raw = localStorage.getItem(RELEASE_CACHE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as CachedRelease;
    if (!parsed?.release?.version) return null;
    return parsed;
  } catch {
    return null;
  }
}

function saveReleaseCache(cache: CachedRelease): void {
  try {
    localStorage.setItem(RELEASE_CACHE_KEY, JSON.stringify(cache));
  } catch {
    // 存储不可用时忽略
  }
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

  /**
   * 获取最新版本（带条件请求缓存与重试）
   *
   * 未鉴权的 GitHub REST API 限额为 60 次/小时/IP。为避免后台检查反复触发 403：
   * - 携带存储的 ETag / If-Modified-Since 发起条件请求；release 未变更时返回
   *   304 Not Modified，**不消耗配额**；
   * - 命中 304 / 限流(403) / 离线 时，回退到上次缓存的结果（降级而非报错）；
   * - 仅 200 时刷新缓存。配合 composable 的 24h 节流，常态下不再产生 403。
   */
  async fetchLatestVersion(
    retries = 2,
  ): Promise<{ release: ReleaseInfo; asset: ReleaseAsset | null } | null> {
    const url = `https://api.github.com/repos/${this.githubRepo}/releases/latest`;
    const cache = loadReleaseCache();
    const cached = cache
      ? { release: cache.release, asset: cache.asset }
      : null;

    for (let attempt = 0; attempt <= retries; attempt++) {
      try {
        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), 10000);

        const headers: Record<string, string> = {
          Accept: "application/vnd.github+json",
          "X-GitHub-Api-Version": "2022-11-28",
          "User-Agent": "StreamGrab-Update-Checker",
        };
        // 条件请求头：未变更 → 304（零配额）
        if (cache?.etag) headers["If-None-Match"] = cache.etag;
        if (cache?.lastModified) {
          headers["If-Modified-Since"] = cache.lastModified;
        }

        const response = await fetch(url, {
          headers,
          signal: controller.signal,
        });

        clearTimeout(timeoutId);

        // 304 未变更：使用缓存，不消耗配额
        if (response.status === 304) return cached;

        // 403 限流：有缓存则降级，否则放弃（不重试，避免放大限流）
        if (response.status === 403) return cached;

        // 404 无 release：缓存作废
        if (response.status === 404) return null;

        if (!response.ok) {
          throw new Error(`HTTP error: ${response.status}`);
        }

        const data = await response.json();

        if (!data.tag_name && !data.name) return cached;

        const release: ReleaseInfo = {
          version: data.tag_name || data.name,
          url: data.html_url,
          notes: data.body,
          assets: data.assets || [],
        };

        const asset = getPlatformAsset(release.assets);

        // 缓存 ETag / Last-Modified / 解析结果，供后续 304 与降级使用
        saveReleaseCache({
          etag: response.headers.get("etag"),
          lastModified: response.headers.get("last-modified"),
          release,
          asset,
        });

        return { release, asset };
      } catch (e) {
        if (e instanceof Error && e.name === "AbortError") {
          // 离线/超时：有缓存则展示上次结果
          return cached;
        }
        if (attempt < retries) {
          await new Promise((resolve) => setTimeout(resolve, 1000));
          continue;
        }
        return cached;
      }
    }

    return cached;
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
    return _formatFileSize(bytes);
  }

  getPlatform(): Platform {
    return getPlatform();
  }
}

export const updateService = new UpdateService();
