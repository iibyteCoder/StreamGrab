/**
 * 版本比较工具
 */

/** 版本解析结果 */
interface ParsedVersion {
  /** 按出现顺序提取的数字段 */
  nums: number[];
  /** 是否含预发布标识（如 -beta、-rc1） */
  prerelease: boolean;
}

/**
 * 解析版本字符串：提取所有数字段，并标记预发布标识。
 *
 * - 兼容 `v` 前缀：`v0.6.0-beta`
 * - 兼容日期版本：`2026-08-09`、`latest-2026-08-09`（FFmpeg 滚动构建）
 * - 兼容构建元数据：`0.6.0+df70f0b3`（`+` 之后按 semver 惯例忽略）
 */
function parseVersion(v: string): ParsedVersion {
  const cleaned = v.trim().replace(/^[vV]/, "").replace(/\+.*$/, "");
  const nums = Array.from(cleaned.matchAll(/\d+/g), (m) => Number(m[0]));
  const prerelease = /-(alpha|beta|rc|pre|dev|nightly|canary|snapshot)/i.test(
    cleaned,
  );
  return { nums, prerelease };
}

/**
 * 比较两个版本
 *
 * 数字段按出现顺序逐位比较（缺失段视为 0）；数字相等时，
 * 带预发布标识的版本视为更旧（`v0.6.0-beta` ≤ `0.6.0`）。
 *
 * @returns 1 (v1 > v2) | -1 (v1 < v2) | 0 (相等)
 */
export function compareVersions(v1: string, v2: string): number {
  const a = parseVersion(v1);
  const b = parseVersion(v2);

  const len = Math.max(a.nums.length, b.nums.length);
  for (let i = 0; i < len; i++) {
    const x = a.nums[i] ?? 0;
    const y = b.nums[i] ?? 0;
    if (x !== y) return x > y ? 1 : -1;
  }

  if (a.prerelease !== b.prerelease) {
    return a.prerelease ? -1 : 1;
  }
  return 0;
}
