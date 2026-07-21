/**
 * 任务领域类型
 *
 * 与后端 `domain/task/` 对应（camelCase JSON 契约）
 */

import type { TaskOverrides } from "./config";

/** 任务状态（与后端 TaskStatus 状态机一致） */
export type TaskStatus =
  | "pending"
  | "analyzing"
  | "downloading"
  | "merging"
  | "muxing"
  | "paused"
  | "completed"
  | "failed"
  | "cancelled";

/** 活跃状态集合 */
export const ACTIVE_STATUSES: ReadonlySet<TaskStatus> = new Set<TaskStatus>([
  "analyzing",
  "downloading",
  "merging",
  "muxing",
]);

/** 终态集合 */
export const FINISHED_STATUSES: ReadonlySet<TaskStatus> = new Set<TaskStatus>([
  "completed",
  "failed",
  "cancelled",
]);

export function isActiveStatus(status: TaskStatus): boolean {
  return ACTIVE_STATUSES.has(status);
}

export function isFinishedStatus(status: TaskStatus): boolean {
  return FINISHED_STATUSES.has(status);
}

/** 实时进度数据（tasks.progress_json 列 + 进度事件载荷） */
export interface ProgressData {
  /** 当前流进度百分比 */
  percent: number;
  /** 总体进度（视频+音频聚合） */
  overallPercent: number;
  /** 下载速度 (bytes/s) */
  speed: number;
  downloadedSize: number;
  totalSize: number;
  downloadedSegments: number;
  totalSegments: number;
  /** 预估剩余时间（秒） */
  eta: number;
  currentAction: string;
}

/** 媒体信息（tasks.media_info_json 列 + 文件分析结果） */
export interface MediaInfo {
  resolution?: string | null;
  width?: number | null;
  height?: number | null;
  frameRate?: number | null;
  videoCodec?: string | null;
  videoRange?: string | null;
  audioCodec?: string | null;
  audioChannels?: string | null;
  audioLanguage?: string | null;
  /** 总时长（秒） */
  duration?: number | null;
  segmentCount?: number | null;
  isLive: boolean;
  isEncrypted: boolean;
  fileFormat?: string | null;
  /** 文件大小（字节，文件分析时填充） */
  fileSize?: number | null;
  /** 比特率 (bps) */
  bitRate?: number | null;
}

/**
 * 任务聚合记录（tasks 表单行，后端单表聚合模型）
 *
 * 前端任务列表/详情直接消费此类型（即原 DownloadTask）
 */
export interface TaskRecord {
  id: string;
  url: string;
  fileName: string;
  saveDir: string;
  outputPath?: string | null;
  status: TaskStatus;
  error?: string | null;
  wasInterrupted: boolean;
  createdAt: string;
  updatedAt: string;
  startedAt?: string | null;
  completedAt?: string | null;
  progress: ProgressData;
  mediaInfo?: MediaInfo | null;
  overrides?: TaskOverrides | null;
}

/** 前端任务模型（与后端记录一致） */
export type DownloadTask = TaskRecord;

/** 速率曲线采样点（进度图表数据） */
export interface ProgressSample {
  percent: number;
  speed: number;
  downloadedSize: number;
  /** Unix 毫秒时间戳 */
  recordedAt: number;
}

/** 历史记录（任务终态快照，独立于任务表） */
export interface HistoryRecord {
  id: number;
  taskId?: string | null;
  url: string;
  fileName: string;
  saveDir: string;
  outputPath?: string | null;
  fileSize?: number | null;
  status: TaskStatus;
  error?: string | null;
  createdAt: string;
  completedAt: string;
  /** 任务级覆盖快照（重新下载时携带原参数） */
  overrides?: TaskOverrides | null;
}
