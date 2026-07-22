/**
 * 版本比较工具
 */

/**
 * 比较两个语义化版本号
 *
 * 支持 `v` 前缀（自动剥离）。按 `.` 分段逐位数值比较，
 * 缺失段视为 0。
 *
 * @returns 1 (v1 > v2) | -1 (v1 < v2) | 0 (相等)
 */
export function compareVersions(v1: string, v2: string): number {
  const parts1 = v1.replace(/^v/, "").split(".").map(Number);
  const parts2 = v2.replace(/^v/, "").split(".").map(Number);

  for (let i = 0; i < Math.max(parts1.length, parts2.length); i++) {
    const p1 = parts1[i] || 0;
    const p2 = parts2[i] || 0;
    if (p1 > p2) return 1;
    if (p1 < p2) return -1;
  }
  return 0;
}
