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

const detectedUrls = new Set<string>();
let lastClipboardContent = "";

function extractStreamUrls(text: string): string[] {
  if (!text) return [];

  const urls: string[] = [];
  for (const pattern of URL_PATTERNS) {
    const matches = text.match(new RegExp(pattern.source, "gi"));
    if (matches) {
      urls.push(...matches.map((url) => url.trim()));
    }
  }

  return [...new Set(urls)];
}

export function useClipboardWatcher() {
  const settingsStore = useSettingsStore();
  const toast = useToast();

  const isWatching = ref(false);
  let unlisten: (() => void) | null = null;
  let pollInterval: ReturnType<typeof setInterval> | null = null;

  async function handleClipboardChange() {
    if (!settingsStore.appSettings.clipboard_watch) return;

    try {
      const content = await readText();
      if (!content || content === lastClipboardContent) return;

      lastClipboardContent = content;
      const urls = extractStreamUrls(content);
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
            ? `已添加下载链接`
            : `已添加 ${newUrls.length} 个下载链接`;

        toast.success(message);
      }
    } catch (e) {
      console.debug("Failed to read clipboard:", e);
    }
  }

  async function startWatching() {
    if (isWatching.value) return;

    isWatching.value = true;

    unlisten = await listen("tauri://focus", () => {
      handleClipboardChange();
    });

    pollInterval = setInterval(handleClipboardChange, 2000);

    console.log("Clipboard watcher started");
  }

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

  function clearDetectedUrls() {
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
