/**
 * Tauri API 封装层
 *
 * 全部 service 经此调用 Tauri 命令 / 订阅事件（组件与 Store 禁止直接使用原始 API）
 */

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

/**
 * 通用 Tauri 命令调用封装
 */
export async function invokeTauri<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    console.error(`Tauri command '${command}' failed:`, error);
    throw error;
  }
}

/**
 * 事件订阅封装（自动解包 payload）
 */
export async function subscribeToEvent<T>(
  event: string,
  handler: (payload: T) => void,
): Promise<UnlistenFn> {
  return listen<T>(event, (event) => {
    handler(event.payload);
  });
}

export type { UnlistenFn };
