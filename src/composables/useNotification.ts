/**
 * 系统通知组合式函数
 *
 * 通过 @tauri-apps/plugin-notification 发送原生系统通知，并根据设置决定是否显示。
 *
 * 修复说明：早期版本用浏览器 Notification API，在 Tauri WebView2 中通知权限
 * 恒为 denied（WebView 无通知权限弹窗），桌面通知实际从不显示。改用 Tauri
 * notification 插件后走系统通知中心，该配置项才真正生效。
 */

import {
  isPermissionGranted,
  requestPermission,
  sendNotification as pluginSendNotification,
} from "@tauri-apps/plugin-notification";
import { useSettingsStore } from "@/stores";
import { i18n } from "@/locales";

/** 通知权限缓存：插件权限一次请求后永久生效，后续不再重复请求 */
let permissionCache: boolean | null = null;

/**
 * 通知组合式函数
 */
export function useNotification() {
  const settingsStore = useSettingsStore();

  /**
   * 检查是否允许显示通知（依据设置项 show_notification）
   */
  const canShowNotification = (): boolean => {
    return settingsStore.appSettings.show_notification;
  };

  /**
   * 获取通知权限（带缓存）：已授权返回 true，未授权尝试请求，失败返回 false
   */
  const ensurePermission = async (): Promise<boolean> => {
    if (permissionCache !== null) return permissionCache;
    try {
      let granted = await isPermissionGranted();
      if (!granted) {
        const permission = await requestPermission();
        granted = permission === "granted";
      }
      permissionCache = granted;
      return granted;
    } catch (e) {
      console.debug("Failed to check notification permission:", e);
      return false;
    }
  };

  /**
   * 发送系统通知
   * @param title 通知标题
   * @param body 通知内容
   * @param options 额外选项（icon 等）
   */
  const sendNotification = async (
    title: string,
    body: string,
    options?: { icon?: string },
  ): Promise<boolean> => {
    // 设置不允许则直接跳过
    if (!canShowNotification()) {
      return false;
    }
    if (!(await ensurePermission())) {
      return false;
    }

    try {
      pluginSendNotification({ title, body, ...options });
      return true;
    } catch (e) {
      console.debug("Failed to send notification:", e);
      return false;
    }
  };

  /**
   * 发送下载完成通知
   * @param fileName 文件名
   */
  const sendDownloadCompleteNotification = (
    fileName: string,
  ): Promise<boolean> => {
    return sendNotification(
      i18n.global.t("messages.downloadCompleted"),
      i18n.global.t("messages.notificationDownloaded", { fileName }),
    );
  };

  /**
   * 发送下载失败通知
   * @param fileName 文件名
   * @param error 错误信息
   */
  const sendDownloadErrorNotification = (
    fileName: string,
    error?: string,
  ): Promise<boolean> => {
    return sendNotification(
      i18n.global.t("messages.downloadFailed"),
      error
        ? i18n.global.t("messages.notificationDownloadFailedWithError", {
            fileName,
            error,
          })
        : i18n.global.t("messages.notificationDownloadFailed", { fileName }),
    );
  };

  return {
    canShowNotification,
    sendNotification,
    sendDownloadCompleteNotification,
    sendDownloadErrorNotification,
    ensurePermission,
  };
}
