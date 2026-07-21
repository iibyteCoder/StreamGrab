/**
 * 剪贴板监控组合式函数
 *
 * 监控剪贴板变化，自动检测 URL 并分发 CustomEvent。
 * URL 识别统一使用 @/domain/url（消灭 composable 内私有正则）。
 */

import { ref, onMounted, onUnmounted, watch } from "vue";
import { useSettingsStore } from "@/stores";
import { useToast } from "./useToast";
import { clipboardService } from "@/services";
import { detectUrlType, isHttpUrl } from "@/domain";
import { i18n } from "@/locales";

/** 已检测 URL 去重集合（模块级，跨组件实例共享） */
const detectedUrls = new Set<string>();
let lastClipboardContent = "";

/** 从文本中提取 URL（以空格/换行分隔） */
function extractUrls(text: string): string[] {
  if (!text) return [];
  return text
    .split(/[\s\n\r]+/)
    .map((s) => s.trim())
    .filter((url) => isHttpUrl(url) && detectUrlType(url) !== "unknown");
}

export function useClipboardWatcher() {
  const settingsStore = useSettingsStore();
  const toast = useToast();

  const isWatching = ref(false);
  let unlistenFocus: (() => void) | null = null;
  let pollInterval: ReturnType<typeof setInterval> | null = null;

  async function handleClipboardChange(): Promise<void> {
    if (!settingsStore.appSettings.clipboard_watch) return;

    try {
      const content = await clipboardService.readText();
      if (!content || content === lastClipboardContent) return;

      lastClipboardContent = content;
      const urls = extractUrls(content);
      const newUrls = urls.filter((url) => !detectedUrls.has(url));

      if (newUrls.length > 0) {
        newUrls.forEach((url) => detectedUrls.add(url));

        window.dispatchEvent(
          new CustomEvent("clipboard-urls-detected", {
            detail: { urls: newUrls },
          }),
        );

        const message =
          newUrls.length === 1
            ? i18n.global.t("messages.clipboardUrlDetected")
            : i18n.global.t("messages.clipboardUrlsDetected", {
                count: newUrls.length,
              });

        toast.success(message);
      }
    } catch (e) {
      console.debug("Failed to read clipboard:", e);
    }
  }

  async function startWatching(): Promise<void> {
    if (isWatching.value) return;

    isWatching.value = true;

    unlistenFocus = await clipboardService.onFocus(() => {
      handleClipboardChange();
    });

    pollInterval = setInterval(handleClipboardChange, 2000);

    console.log("Clipboard watcher started");
  }

  function stopWatching(): void {
    if (unlistenFocus) {
      unlistenFocus();
      unlistenFocus = null;
    }

    if (pollInterval) {
      clearInterval(pollInterval);
      pollInterval = null;
    }

    isWatching.value = false;
    console.log("Clipboard watcher stopped");
  }

  function clearDetectedUrls(): void {
    detectedUrls.clear();
  }

  watch(
    () => settingsStore.appSettings.clipboard_watch,
    (enabled) => {
      if (enabled) {
        startWatching();
      } else {
        stopWatching();
      }
    },
    { immediate: true },
  );

  onMounted(() => {
    if (settingsStore.appSettings.clipboard_watch) {
      startWatching();
    }
  });

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
