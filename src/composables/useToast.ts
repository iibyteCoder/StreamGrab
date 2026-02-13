/**
 * Toast 提示组合式函数
 * 封装 Toast 通知的便捷方法
 */

import { useUiStore } from '@/stores';

/**
 * Toast 选项
 */
export interface ToastOptions {
  /** 显示时长 (ms)，默认 3000 */
  duration?: number;
}

/**
 * Toast 组合式函数
 */
export function useToast() {
  const uiStore = useUiStore();

  /**
   * 显示成功提示
   */
  const success = (message: string, options?: ToastOptions): string => {
    return uiStore.showSuccess(message, options?.duration);
  };

  /**
   * 显示错误提示
   */
  const error = (message: string, options?: ToastOptions): string => {
    return uiStore.showError(message, options?.duration);
  };

  /**
   * 显示警告提示
   */
  const warning = (message: string, options?: ToastOptions): string => {
    return uiStore.showWarning(message, options?.duration);
  };

  /**
   * 显示信息提示
   */
  const info = (message: string, options?: ToastOptions): string => {
    return uiStore.showInfo(message, options?.duration);
  };

  /**
   * 移除指定 Toast
   */
  const remove = (id: string): void => {
    uiStore.removeToast(id);
  };

  /**
   * 清除所有 Toast
   */
  const clear = (): void => {
    uiStore.clearToasts();
  };

  return {
    success,
    error,
    warning,
    info,
    remove,
    clear,
    toasts: uiStore.toasts,
  };
}
