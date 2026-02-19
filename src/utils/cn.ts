/**
 * Tailwind CSS 类名合并工具
 */
import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

/**
 * 合并 Tailwind CSS 类名，智能处理冲突
 * @example cn('px-2 py-1', 'p-4') // => 'p-4'
 */
export function cn(...inputs: ClassValue[]): string {
  return twMerge(clsx(inputs));
}
