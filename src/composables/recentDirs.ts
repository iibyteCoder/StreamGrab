export const STORAGE_KEY = "streamgrab:recentSaveDirs";
export const MAX_RECENT = 5;

/** 纯函数：把 dir 记到最前，去重、去空、截断到 MAX_RECENT */
export function rememberDir(list: string[], dir: string): string[] {
  const t = dir.trim();
  if (!t) return list;
  return [t, ...list.filter((d) => d !== t)].slice(0, MAX_RECENT);
}

/** 纯函数：有效默认目录 = 最近记忆 > 全局默认 > 空 */
export function resolveDefaultDir(recent: string[], global: string): string {
  return (recent[0] ?? "").trim() || global.trim() || "";
}
