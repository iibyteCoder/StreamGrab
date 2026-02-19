/**
 * 流信息类型
 */

// URL 类型
export type UrlType = "hls" | "dash" | "mss" | "httpVideo" | "unknown";

// URL 类型检测函数
export function detectUrlType(url: string): UrlType {
  const urlLower = url.toLowerCase();

  // 检查流媒体格式
  if (urlLower.endsWith(".m3u8") || urlLower.includes(".m3u8?")) {
    return "hls";
  }
  if (urlLower.endsWith(".mpd") || urlLower.includes(".mpd?")) {
    return "dash";
  }
  if (
    urlLower.endsWith(".ism/manifest") ||
    urlLower.includes(".ism/manifest?") ||
    urlLower.endsWith(".isml/manifest")
  ) {
    return "mss";
  }

  // 检查常见视频扩展名
  const videoExtensions = [
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

  for (const ext of videoExtensions) {
    if (urlLower.endsWith(ext) || urlLower.includes(`${ext}?`)) {
      return "httpVideo";
    }
  }

  return "unknown";
}

// 判断是否需要使用 ffmpeg 下载
export function needsFfmpeg(urlType: UrlType): boolean {
  return urlType === "httpVideo";
}

// 判断是否是流媒体格式（N_m3u8DL-RE 支持）
export function isStreamingType(urlType: UrlType): boolean {
  return urlType === "hls" || urlType === "dash" || urlType === "mss";
}

// 流信息
export interface StreamInfo {
  videos: VideoStream[];
  audios: AudioStream[];
  subtitles: SubtitleStream[];
  duration: number;
  segmentCount: number;
  isLive: boolean;
  isEncrypted: boolean;
}

// 基础流
export interface BaseStream {
  id: string;
  bandwidth: number;
  codecs: string;
  language: string;
  name: string;
  groupId?: string;
  selected?: boolean;
}

// 视频流
export interface VideoStream extends BaseStream {
  resolution: string;
  width: number;
  height: number;
  frameRate: number;
  videoRange: "SDR" | "HDR10" | "HDR10+" | "DV" | "HLG";
}

// 音频流
export interface AudioStream extends BaseStream {
  channels: string;
  sampleRate: number;
  isDefault: boolean;
}

// 字幕流
export interface SubtitleStream extends BaseStream {
  format: "srt" | "vtt" | "ttml";
  isDefault: boolean;
  isForced: boolean;
}

// 流选择
export interface StreamSelection {
  videoIds: string[];
  audioIds: string[];
  subtitleIds: string[];
}
