import { isStreamingType } from "@/domain/url";
import type { TaskOverrides, UrlType } from "@/domain";
import type { BatchDefaults, StagedLink } from "./staging-types";

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
export function cleanOverrides(
  overrides: TaskOverrides,
): TaskOverrides | undefined {
  const o: TaskOverrides = {};
  if (overrides.saveDir) o.saveDir = overrides.saveDir;
  if (overrides.saveName) o.saveName = overrides.saveName;
  if (overrides.muxFormat) o.muxFormat = overrides.muxFormat;
  if (overrides.maxSpeed) o.maxSpeed = overrides.maxSpeed;
  if (overrides.customRange) o.customRange = overrides.customRange;
  if (overrides.subtitleFormat) o.subtitleFormat = overrides.subtitleFormat;
  if (overrides.subtitlesOnly != null)
    o.subtitlesOnly = overrides.subtitlesOnly;
  if (overrides.scheduledStartAt)
    o.scheduledStartAt = overrides.scheduledStartAt;
  if (overrides.selection) o.selection = overrides.selection;
  if (overrides.presetId) o.presetId = overrides.presetId;
  if (overrides.key) o.key = overrides.key;
  const hasAny = Object.values(o).some((v) => v !== undefined && v !== null);
  return hasAny ? o : undefined;
}

/**
 * 合并「逐条 > 批次默认 > 全局默认」三层，产出可直接建任务的规格。
 * 唯一的合并规则持有者（设计 4.4）。
 */
export function resolveLinkToTask(
  link: StagedLink,
  batch: BatchDefaults,
  globalSaveDir: string,
): ResolvedTask {
  const saveDir = firstNonEmpty(link.saveDir, batch.saveDir, globalSaveDir);
  const fileName = link.fileName.trim() || undefined;
  const overrides = cleanOverrides(link.overrides);
  const hasSchedule = !!overrides?.scheduledStartAt;
  return { url: link.url, fileName, saveDir, overrides, hasSchedule };
}

/**
 * 预设作为「初值提供者」（设计 4.3）：
 * 仅流媒体行接受预设 overrides 作初值；直链行返回空对象。
 * selection 做浅拷贝避免多行共享同一引用。
 */
export function seedPresetOverrides(
  preset: TaskOverrides | null,
  urlType: UrlType | null,
): TaskOverrides {
  if (urlType === null || !isStreamingType(urlType)) return {};
  if (!preset) return {};
  return {
    ...preset,
    selection: preset.selection ? { ...preset.selection } : undefined,
  };
}
