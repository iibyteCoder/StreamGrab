/**
 * 剪贴板服务
 *
 * 封装 @tauri-apps/plugin-clipboard-manager 的读取 + 焦点事件订阅。
 * composable 层不直接 import @tauri-apps 原始 API。
 */

import { readText } from "@tauri-apps/plugin-clipboard-manager";
import { subscribeToEvent, type UnlistenFn } from "./tauri";

class ClipboardService {
  /** 读取剪贴板文本 */
  async readText(): Promise<string> {
    return readText();
  }

  /** 订阅窗口焦点事件 */
  async onFocus(handler: () => void): Promise<UnlistenFn> {
    return subscribeToEvent<null>("tauri://focus", () => handler());
  }
}

export const clipboardService = new ClipboardService();
