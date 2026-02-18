/**
 * 剪贴板监控组合式函数
 * 监控剪贴板变化，自动检测 M3U8/MPD/MSS 链接
 */

import { ref, onMounted, onUnmounted, watch } from "vue";
import { listen } from "@tauri-apps/api/event";
import { readText } from "@tauri-apps/plugin-clipboard-manager";
import { useSettingsStore } from "@/stores";
import { useToast } from "./useToast";

// URL 匹配正则
const URL_PATTERNS = [
  /https?:\/\/[^\s]+\.(?:m3u8|mpd|mss)/i,
  /https?:\/\/[^\s]*\?(?:.*&)?(?:m3u8|mpd|manifest)/i,
];

// 记录已检测过的 URL，避免重复提示
const detectedUrls = new Set<string>();

// 上一次剪贴板内容
let lastClipboardContent = "";

/**
 * 检查文本是否包含有效的流媒体 URL
 */
function extractStreamUrls(text: string): string[] {
  if (!text) return [];

  const urls: string[] = [];
  for (const pattern of URL_PATTERNS) {
    const matches = text.match(new RegExp(pattern.source, "gi"));
    if (matches) {
      urls.push(...matches.map((url) => url.trim()));
    }
  }

  // 去重
  return [...new Set(urls)];
}

/**
 * 剪贴板监控组合式函数
 */
export function useClipboardWatcher() {
  const settingsStore = useSettingsStore();
  const toast = useToast();

  // 是否正在监控
  const isWatching = ref(false);

  // 监听器取消函数
  let unlisten: (() => void) | null = null;

  // 轮询定时器（备用方案）
  let pollInterval: ReturnType<typeof setInterval> | null = null;

  /**
   * 处理剪贴板变化
   */
  async function handleClipboardChange() {
    if (!settingsStore.settings.ui.clipboardWatch) return;

    try {
      const content = await readText();
      if (!content || content === lastClipboardContent) return;

      lastClipboardContent = content;
      const urls = extractStreamUrls(content);

      // 过滤已检测过的 URL
      const newUrls = urls.filter((url) => !detectedUrls.has(url));

      if (newUrls.length > 0) {
        // 记录已检测
        newUrls.forEach((url) => detectedUrls.add(url));

        // 直接发送事件通知主界面添加 URL
        window.dispatchEvent(
          new CustomEvent("clipboard-urls-detected", {
            detail: { urls: newUrls },
          }),
        );

        // 发送提示
        const message =
          newUrls.length === 1
            ? `已添加下载链接`
            : `已添加 ${newUrls.length} 个下载链接`;

        toast.success(message);
      }
    } catch (e) {
      // 忽略读取错误
      console.debug("Failed to read clipboard:", e);
    }
  }

  /**
   * 开始监控
   */
  async function startWatching() {
    if (isWatching.value) return;

    isWatching.value = true;

    // 监听窗口焦点变化时检查剪贴板
    unlisten = await listen("tauri://focus", () => {
      handleClipboardChange();
    });

    // 同时使用轮询作为备用方案（每 2 秒检查一次）
    pollInterval = setInterval(handleClipboardChange, 2000);

    console.log("Clipboard watcher started");
  }

  /**
   * 停止监控
   */
  function stopWatching() {
    if (unlisten) {
      unlisten();
      unlisten = null;
    }

    if (pollInterval) {
      clearInterval(pollInterval);
      pollInterval = null;
    }

    isWatching.value = false;
    console.log("Clipboard watcher stopped");
  }

  /**
   * 清除已检测 URL 记录
   */
  function clearDetectedUrls() {
    detectedUrls.clear();
  }

  // 监听设置变化
  watch(
    () => settingsStore.settings.ui.clipboardWatch,
    (enabled) => {
      if (enabled) {
        startWatching();
      } else {
        stopWatching();
      }
    },
    { immediate: true },
  );

  // 组件挂载时，如果设置已启用则开始监控
  onMounted(() => {
    if (settingsStore.settings.ui.clipboardWatch) {
      startWatching();
    }
  });

  // 组件卸载时停止监控
  onUnmounted(() => {
    stopWatching();
  });

  return {
    isWatching,
    startWatching,
    stopWatching,
    clearDetectedUrls,
  };
}
