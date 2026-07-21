/**
 * ID 生成工具（统一实现，消灭各 Store 的复制粘贴）
 */

/**
 * 生成唯一 ID
 * @param prefix 可选前缀（如预设用 "preset-"）
 */
export function generateId(prefix = ""): string {
  return `${prefix}${crypto.randomUUID()}`;
}
