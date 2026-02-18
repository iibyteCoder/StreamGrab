/**
 * Toast 提示组合式函数
 * 封装 Shadcn-Vue Toast 通知的便捷方法
 */

import {
  toast as shadcnToast,
  useToast as useShadcnToast,
} from "@/components/ui/toast";

/**
 * Toast 选项
 */
export interface ToastOptions {
  /** 显示时长 (ms)，默认 3000 */
  duration?: number;
  /** 标题 */
  title?: string;
}

/**
 * Toast 组合式函数
 */
export function useToast() {
  const { toasts, dismiss } = useShadcnToast();

  /**
   * 显示成功提示
   */
  const success = (message: string, options?: ToastOptions): string => {
    const result = shadcnToast({
      title: options?.title || "成功",
      description: message,
      variant: "default",
    });
    return result.id;
  };

  /**
   * 显示错误提示
   */
  const error = (message: string, options?: ToastOptions): string => {
    const result = shadcnToast({
      title: options?.title || "错误",
      description: message,
      variant: "destructive",
    });
    return result.id;
  };

  /**
   * 显示警告提示
   */
  const warning = (message: string, options?: ToastOptions): string => {
    const result = shadcnToast({
      title: options?.title || "警告",
      description: message,
    });
    return result.id;
  };

  /**
   * 显示信息提示
   */
  const info = (message: string, options?: ToastOptions): string => {
    const result = shadcnToast({
      title: options?.title || "提示",
      description: message,
    });
    return result.id;
  };

  /**
   * 移除指定 Toast
   */
  const remove = (id: string): void => {
    dismiss(id);
  };

  /**
   * 清除所有 Toast
   */
  const clear = (): void => {
    dismiss();
  };

  return {
    success,
    error,
    warning,
    info,
    remove,
    clear,
    toasts,
  };
}
