/**
 * 流信息类型
 */

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
