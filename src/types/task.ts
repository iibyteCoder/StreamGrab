/**
 * 任务相关类型
 * 与后端 FullTaskRecord 结构对齐
 */

// 任务状态
export type TaskStatus =
  | "pending" // 等待中
  | "analyzing" // 解析中
  | "downloading" // 下载中
  | "paused" // 已暂停
  | "merging" // 合并中
  | "muxing" // 混流中
  | "completed" // 已完成
  | "failed" // 失败
  | "cancelled"; // 已取消

// 媒体信息（用于前端显示）
export interface MediaInfo {
  resolution?: string;
  width?: number;
  height?: number;
  frameRate?: number;
  videoCodec?: string;
  videoRange?: string;
  audioCodec?: string;
  audioChannels?: string;
  audioLanguage?: string;
  duration?: number;
  segmentCount?: number;
  isLive?: boolean;
  isEncrypted?: boolean;
  fileFormat?: string;
}

// 任务进度数据
export interface TaskProgressData {
  percent: number;
  /** 总体进度百分比（视频+音频合并计算） */
  overallPercent?: number;
  speed: number; // bytes/s
  downloadedSize: number;
  totalSize: number;
  downloadedSegments: number;
  totalSegments: number;
  /** 总分片数（视频+音频） */
  totalDownloadedSegments?: number;
  eta: number; // seconds
  currentAction: string;
}

// 下载任务（完整信息，与后端 FullTaskRecord 对齐）
export interface DownloadTask {
  // 基本信息
  id: string;
  url: string;
  fileName: string;
  saveDir: string;
  outputPath?: string;
  status: TaskStatus;
  error?: string;
  wasInterrupted: boolean;

  // 时间戳
  createdAt: Date;
  updatedAt: Date;
  startedAt?: Date;
  completedAt?: Date;

  // 进度（扁平化）
  progressPercent: number;
  progressSpeed: number;
  progressDownloadedSize: number;
  progressTotalSize: number;
  progressDownloadedSegments: number;
  progressTotalSegments: number;
  progressEta: number;
  progressCurrentAction: string;

  // 媒体信息（扁平化）
  mediaResolution?: string;
  mediaWidth?: number;
  mediaHeight?: number;
  mediaFrameRate?: number;
  mediaVideoCodec?: string;
  mediaVideoRange?: string;
  mediaAudioCodec?: string;
  mediaAudioChannels?: string;
  mediaAudioLanguage?: string;
  mediaDuration?: number;
  mediaSegmentCount?: number;
  mediaIsLive: boolean;
  mediaIsEncrypted: boolean;
  mediaFileFormat?: string;

  // 运行时配置（不持久化）
  config?: Record<string, unknown>;
}

// 任务创建参数（用于创建新任务）
export interface TaskCreateParams {
  id: string;
  url: string;
  fileName: string;
  saveDir: string;
  outputPath?: string;
  status: TaskStatus;
  error?: string;
  wasInterrupted?: boolean;
  createdAt?: string;
  updatedAt?: string;
  startedAt?: string;
  completedAt?: string;
}

// 任务配置
export interface TaskConfig {
  saveDir?: string;
  saveName?: string;
  savePattern?: { enabled: boolean; template: string; presetId?: string };
  startAt?: Date;
  threadCount?: number;
  retryCount?: number;
  timeout?: number;
  maxSpeed?: string;
  autoSelect?: boolean;
  selectVideo?: string;
  selectAudio?: string;
  selectSubtitle?: string;
  dropVideo?: string;
  dropAudio?: string;
  dropSubtitle?: string;
  muxFormat?: string;
  muxAfterDone?: boolean;
  skipMerge?: boolean;
  delAfterDone?: boolean;
  checkSegmentsCount?: boolean;
  customRange?: string;
  key?: string;
  proxy?: string;
  headers?: { key: string; value: string; enabled: boolean }[];
}

// 任务日志条目
export interface TaskLogEntry {
  timestamp: Date;
  level: "info" | "warn" | "error" | "debug";
  message: string;
}

// ============================================
// 辅助函数
// ============================================

// 用于前端显示的进度对象（从扁平数据构建）
export function getProgressFromTask(task: DownloadTask): TaskProgressData {
  return {
    percent: task.progressPercent,
    speed: task.progressSpeed,
    downloadedSize: task.progressDownloadedSize,
    totalSize: task.progressTotalSize,
    downloadedSegments: task.progressDownloadedSegments,
    totalSegments: task.progressTotalSegments,
    eta: task.progressEta,
    currentAction: task.progressCurrentAction,
  };
}

// 用于前端显示的媒体信息对象（从扁平数据构建）
export function getMediaInfoFromTask(
  task: DownloadTask,
): MediaInfo | undefined {
  const hasMediaInfo =
    task.mediaResolution ||
    task.mediaWidth ||
    task.mediaHeight ||
    task.mediaVideoCodec ||
    task.mediaAudioCodec;

  if (!hasMediaInfo) return undefined;

  return {
    resolution: task.mediaResolution,
    width: task.mediaWidth,
    height: task.mediaHeight,
    frameRate: task.mediaFrameRate,
    videoCodec: task.mediaVideoCodec,
    videoRange: task.mediaVideoRange,
    audioCodec: task.mediaAudioCodec,
    audioChannels: task.mediaAudioChannels,
    audioLanguage: task.mediaAudioLanguage,
    duration: task.mediaDuration,
    segmentCount: task.mediaSegmentCount,
    isLive: task.mediaIsLive,
    isEncrypted: task.mediaIsEncrypted,
    fileFormat: task.mediaFileFormat,
  };
}
