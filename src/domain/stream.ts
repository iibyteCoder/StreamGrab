/**
 * 流信息类型（parse_url 解析结果）
 *
 * 与后端 `domain/download/stream_info.rs` 对应（camelCase，flatten 展开）
 */

/** URL 类型（后端 detect_url_type 命令的返回值） */
export type UrlType = "hls" | "dash" | "mss" | "httpVideo" | "unknown";

/** 视频流 */
export interface VideoStream {
  id: string;
  bandwidth: number;
  codecs: string;
  language: string;
  name: string;
  groupId?: string | null;
  selected?: boolean | null;
  resolution: string;
  width: number;
  height: number;
  frameRate: number;
  videoRange: string;
}

/** 音频流 */
export interface AudioStream {
  id: string;
  bandwidth: number;
  codecs: string;
  language: string;
  name: string;
  groupId?: string | null;
  selected?: boolean | null;
  channels: string;
  sampleRate: number;
  isDefault: boolean;
}

/** 字幕流 */
export interface SubtitleStream {
  id: string;
  bandwidth: number;
  codecs: string;
  language: string;
  name: string;
  groupId?: string | null;
  selected?: boolean | null;
  format: string;
  isDefault: boolean;
  isForced: boolean;
}

/** 流信息 */
export interface StreamInfo {
  videos: VideoStream[];
  audios: AudioStream[];
  subtitles: SubtitleStream[];
  /** 总时长（秒） */
  duration: number;
  segmentCount: number;
  isLive: boolean;
  isEncrypted: boolean;
}
