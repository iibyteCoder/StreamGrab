/**
 * 工具管理服务
 *
 * 外部工具（N_m3u8DL-RE、FFmpeg）的检测、版本查询与下载安装。
 * 统一走 services/tauri.ts 封装（不直接使用 Tauri 原始 API）。
 */

import { invokeTauri, subscribeToEvent, type UnlistenFn } from "./tauri";

/** 工具信息（检测结果） */
export interface ToolInfo {
  name: string;
  installed: boolean;
  version: string | null;
  exePath: string | null;
  dirPath: string | null;
  error: string | null;
}

/** 工具发布信息（GitHub 最新 release / macOS evermeet 源） */
export interface ToolReleaseInfo {
  version: string;
  downloadUrl: string;
  filename: string;
  publishedAt: string;
  /** 随附的额外压缩包（如 macOS 的 ffprobe），与主包解压到同一目录 */
  extraAssets?: string[];
}

/** 工具下载进度事件 */
export interface ToolDownloadProgress {
  tool: string;
  status: "downloaded" | "extracting" | string;
  downloaded: number;
  total: number;
  percent: number;
}

class ToolsService {
  // ===== 检测 =====

  /** 检测 N_m3u8DL-RE（path 可为目录或完整路径，空 = 自动查找） */
  getNm3u8dlInfo(path?: string | null): Promise<ToolInfo> {
    return invokeTauri<ToolInfo>("get_nm3u8dl_info", { path: path ?? null });
  }

  getFfmpegInfo(path?: string | null): Promise<ToolInfo> {
    return invokeTauri<ToolInfo>("get_ffmpeg_info", { path: path ?? null });
  }

  getFfprobeInfo(ffmpegPath?: string | null): Promise<ToolInfo> {
    return invokeTauri<ToolInfo>("get_ffprobe_info", {
      ffmpegPath: ffmpegPath ?? null,
    });
  }

  // ===== 版本与下载 =====

  getNm3u8dlLatestRelease(): Promise<ToolReleaseInfo> {
    return invokeTauri<ToolReleaseInfo>("get_nm3u8dl_latest_release");
  }

  getFfmpegLatestRelease(): Promise<ToolReleaseInfo> {
    return invokeTauri<ToolReleaseInfo>("get_ffmpeg_latest_release");
  }

  /**
   * 下载并解压工具（支持 .zip / .tar.gz / .tar.xz，按当前平台选择资产）
   * @param extraUrls 随附的额外压缩包（如 macOS 的 ffprobe），解压到同一目录
   * @returns 解压后的可执行文件目录
   */
  downloadTool(
    tool: string,
    downloadUrl: string,
    targetDir: string,
    extraUrls: string[] = [],
  ): Promise<string> {
    return invokeTauri<string>("download_tool", {
      tool,
      downloadUrl,
      targetDir,
      extraUrls,
    });
  }

  /** 订阅工具下载进度事件 */
  subscribeToDownloadProgress(
    tool: string,
    handler: (progress: ToolDownloadProgress) => void,
  ): Promise<UnlistenFn> {
    return subscribeToEvent<ToolDownloadProgress>(
      `tool:download:progress:${tool}`,
      handler,
    );
  }
}

export const toolsService = new ToolsService();
