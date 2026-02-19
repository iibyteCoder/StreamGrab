/**
 * 任务相关类型
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

// 下载任务
export interface DownloadTask {
  id: string;
  url: string;
  fileName: string;
  saveDir: string;
  status: TaskStatus;
  progress: TaskProgressData;
  error?: string;
  config?: Partial<TaskConfig>;
  outputPath?: string;
  createdAt: Date;
  updatedAt: Date;
  startedAt?: Date;
  completedAt?: Date;
}

// 任务进度数据
export interface TaskProgressData {
  percent: number;
  speed: number; // bytes/s
  downloadedSize: number;
  totalSize: number;
  downloadedSegments: number;
  totalSegments: number;
  eta: number; // seconds
  currentAction: string;
}

// 任务日志条目
export interface TaskLogEntry {
  timestamp: Date;
  level: "info" | "warn" | "error" | "debug";
  message: string;
}

// 任务配置
export interface TaskConfig {
  saveDir: string;
  saveName: string;
  threadCount: number;
  retryCount: number;
  timeout: number;
  maxSpeed: string;

  // 流选择
  autoSelect: boolean;
  selectVideo?: string;
  selectAudio?: string;
  selectSubtitle?: string;

  // 流排除
  dropVideo?: string;
  dropAudio?: string;
  dropSubtitle?: string;

  // 命名模板
  savePattern?: SavePatternSettings;

  // 混流
  muxFormat?: "mp4" | "mkv";
  muxAfterDone: boolean;

  // 其他选项
  skipMerge: boolean;
  delAfterDone: boolean;
  checkSegmentsCount: boolean;

  // 范围下载
  customRange?: string;

  // 定时开始
  startAt?: Date | null;

  // 高级
  headers?: HeaderConfig[];
  proxy?: string;
  key?: string;
}

// ============================================
// 相关类型引用
// ============================================

import type { HeaderConfig, SavePatternSettings } from "./common";
