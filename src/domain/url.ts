/**
 * URL 类型检测（前端本地快速检测）
 *
 * 与后端 `UrlType::detect` 逻辑一致的本地实现，用于输入时的即时徽章反馈；
 * 最终分派以后端 `detect_url_type` 命令为准。
 * 剪贴板监控、URL 验证等所有场景统一使用本模块（消灭多处正则重复）。
 */

import type { UrlType } from "./stream";

const VIDEO_EXTENSIONS = [
  ".mp4",
  ".mkv",
  ".avi",
  ".mov",
  ".wmv",
  ".flv",
  ".webm",
  ".m4v",
  ".ts",
  ".m2ts",
  ".mp3",
  ".m4a",
  ".aac",
  ".ogg",
  ".flac",
  ".wav",
];

/** 检测 URL 类型（与后端 UrlType::detect 一致） */
export function detectUrlType(url: string): UrlType {
  const urlLower = url.trim().toLowerCase();

  if (urlLower.endsWith(".m3u8") || urlLower.includes(".m3u8?")) {
    return "hls";
  }
  if (urlLower.endsWith(".mpd") || urlLower.includes(".mpd?")) {
    return "dash";
  }
  if (
    urlLower.endsWith(".ism/manifest") ||
    urlLower.includes(".ism/manifest?") ||
    urlLower.endsWith(".isml/manifest") ||
    urlLower.includes(".isml/manifest?")
  ) {
    return "mss";
  }

  for (const ext of VIDEO_EXTENSIONS) {
    if (urlLower.endsWith(ext) || urlLower.includes(`${ext}?`)) {
      return "httpVideo";
    }
  }

  return "unknown";
}

/** 是否直链视频（FFmpeg 引擎处理） */
export function needsFfmpeg(urlType: UrlType): boolean {
  return urlType === "httpVideo";
}

/** 是否流媒体格式（N_m3u8DL-RE 引擎处理） */
export function isStreamingType(urlType: UrlType): boolean {
  return urlType === "hls" || urlType === "dash" || urlType === "mss";
}

/** URL 是否大致合法（http/https 协议） */
export function isHttpUrl(url: string): boolean {
  try {
    const u = new URL(url.trim());
    return u.protocol === "http:" || u.protocol === "https:";
  } catch {
    return false;
  }
}
