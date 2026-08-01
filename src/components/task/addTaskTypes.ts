import type { StreamInfo, TaskOverrides, UrlType } from "@/domain";

/** 向导步骤 */
export type WizardStep = "paste" | "parsing" | "config" | "done";

/** 高级选项键（供 linkOptionVisibility 使用） */
export type LinkOption =
  | "fileName"
  | "saveDir"
  | "schedule"
  | "maxSpeed"
  | "customRange"
  | "muxFormat"
  | "subtitleFormat"
  | "subtitlesOnly"
  | "streamSelection"
  | "key";

/** 纯解析产出（无 id、无 overrides —— 由向导装配为 StagedLink） */
export interface ParsedLink {
  url: string;
  detectedType: UrlType;
  fileName: string;
  streaming: boolean;
}

/** 向导内单条待配置链接（仅前端暂存，不进领域/后端） */
export interface StagedLink {
  id: string;
  url: string;
  detectedType: UrlType;
  fileName: string;
  saveDir: string;
  overrides: TaskOverrides;
  streamInfo?: StreamInfo;
  /** 流媒体解析失败（失败 ≠ 无效；无效在解析阶段已剔除） */
  parseFailed: boolean;
}

/** URL 类型 → 徽章文案（集中一处，消除分散） */
export const URL_TYPE_BADGE: Record<UrlType, string> = {
  hls: "HLS",
  dash: "DASH",
  mss: "MSS",
  httpVideo: "直链视频",
  unknown: "未知",
};

export function typeBadgeLabel(t: UrlType): string {
  return URL_TYPE_BADGE[t];
}
