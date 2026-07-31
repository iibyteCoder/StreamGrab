import type { StreamInfo, TaskOverrides, UrlType } from "@/domain";

/** 聚焦面板中按类型动态显示的配置项 */
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

/** 暂存链接状态：pending 未查看 / parsed 已解析 / ready 已确认 / invalid 无效 */
export type LinkStatus = "pending" | "parsed" | "ready" | "invalid";

/** 单条暂存链接（仅前端暂存层使用，不进领域/后端） */
export interface StagedLink {
  id: string;
  url: string;
  detectedType: UrlType | null;
  fileName: string;
  saveDir: string;
  overrides: TaskOverrides;
  status: LinkStatus;
  streamInfo?: StreamInfo;
}

/** 批次默认（AddTaskDialog 持有，不随任务持久化） */
export interface BatchDefaults {
  saveDir: string;
  autoStart: boolean;
}
