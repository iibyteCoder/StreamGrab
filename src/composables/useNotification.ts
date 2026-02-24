/**
 * 系统通知组合式函数
 * 处理系统通知的发送，并根据设置决定是否显示
 */

import { useSettingsStore } from "@/stores";

/**
 * 通知组合式函数
 */
export function useNotification() {
  const settingsStore = useSettingsStore();

  /**
   * 检查是否允许显示通知
   */
  const canShowNotification = (): boolean => {
    return settingsStore.appSettings.show_notification;
  };

  /**
   * 发送系统通知
   * @param title 通知标题
   * @param body 通知内容
   * @param options 额外选项
   */
  const sendNotification = async (
    title: string,
    body: string,
    options?: NotificationOptions,
  ): Promise<boolean> => {
    // 检查设置是否允许通知
    if (!canShowNotification()) {
      console.log("Notification disabled by settings");
      return false;
    }

    // 检查浏览器是否支持通知
    if (!("Notification" in window)) {
      console.warn("This browser does not support notifications");
      return false;
    }

    // 检查通知权限
    let permission = Notification.permission;

    if (permission === "default") {
      // 请求权限
      permission = await Notification.requestPermission();
    }

    if (permission !== "granted") {
      console.warn("Notification permission denied");
      return false;
    }

    // 发送通知
    try {
      const notification = new Notification(title, {
        body,
        icon: "/favicon.ico",
        ...options,
      });

      // 点击通知时聚焦窗口
      notification.onclick = () => {
        window.focus();
        notification.close();
      };

      return true;
    } catch (e) {
      console.error("Failed to send notification:", e);
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
    return sendNotification("下载完成", `${fileName} 已成功下载`);
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
      "下载失败",
      `${fileName} 下载失败${error ? `: ${error}` : ""}`,
    );
  };

  /**
   * 请求通知权限
   */
  const requestPermission = async (): Promise<NotificationPermission> => {
    if (!("Notification" in window)) {
      return "denied";
    }
    return await Notification.requestPermission();
  };

  /**
   * 获取当前通知权限状态
   */
  const getPermissionStatus = (): NotificationPermission => {
    if (!("Notification" in window)) {
      return "denied";
    }
    return Notification.permission;
  };

  return {
    canShowNotification,
    sendNotification,
    sendDownloadCompleteNotification,
    sendDownloadErrorNotification,
    requestPermission,
    getPermissionStatus,
  };
}
