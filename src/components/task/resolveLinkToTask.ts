import type { TaskOverrides } from "@/domain";
import type { StagedLink } from "./addTaskTypes";

/** 解析后的任务规格（喂给 addAndStartTask / taskStore.addTask） */
export interface ResolvedTask {
  url: string;
  fileName?: string;
  saveDir?: string;
  overrides?: TaskOverrides;
  hasSchedule: boolean;
}

function firstNonEmpty(...vals: string[]): string | undefined {
  const found = vals.map((v) => v.trim()).find((v) => v.length > 0);
  return found || undefined;
}

/** 剔除空字段，返回干净 TaskOverrides（无字段则 undefined） */
export function cleanOverrides(overrides: TaskOverrides): TaskOverrides | undefined {
  const o: TaskOverrides = {};
  if (overrides.saveDir) o.saveDir = overrides.saveDir;
  if (overrides.saveName) o.saveName = overrides.saveName;
  if (overrides.muxFormat) o.muxFormat = overrides.muxFormat;
  if (overrides.maxSpeed) o.maxSpeed = overrides.maxSpeed;
  if (overrides.customRange) o.customRange = overrides.customRange;
  if (overrides.subtitleFormat) o.subtitleFormat = overrides.subtitleFormat;
  if (overrides.subtitlesOnly != null) o.subtitlesOnly = overrides.subtitlesOnly;
  if (overrides.scheduledStartAt) o.scheduledStartAt = overrides.scheduledStartAt;
  if (overrides.selection) o.selection = overrides.selection;
  if (overrides.presetId) o.presetId = overrides.presetId;
  if (overrides.key) o.key = overrides.key;
  const hasAny = Object.values(o).some((v) => v !== undefined && v !== null);
  return hasAny ? o : undefined;
}

/**
 * 两层合并「逐条配置 > 有效默认目录」，产出可直接建任务的规格。
 * fallbackSaveDir 传 useRecentDirs.defaultDir（最近记忆 > 全局默认）。
 */
export function resolveLinkToTask(link: StagedLink, fallbackSaveDir: string): ResolvedTask {
  const saveDir = firstNonEmpty(link.saveDir, fallbackSaveDir);
  const fileName = link.fileName.trim() || undefined;
  const overrides = cleanOverrides(link.overrides);
  const hasSchedule = !!overrides?.scheduledStartAt;
  return { url: link.url, fileName, saveDir, overrides, hasSchedule };
}
