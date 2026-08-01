import { computed } from "vue";
import { useStorage } from "@vueuse/core";
import { useSettingsStore } from "@/stores";
import { STORAGE_KEY, rememberDir, resolveDefaultDir } from "./recentDirs";

/** 最近保存目录记忆（localStorage，MRU-first，上限 5） */
export function useRecentDirs() {
  const settingsStore = useSettingsStore();
  const dirs = useStorage<string[]>(STORAGE_KEY, []);
  const defaultDir = computed(() =>
    resolveDefaultDir(dirs.value, settingsStore.defaultSaveDir),
  );
  function remember(dir: string): void {
    dirs.value = rememberDir(dirs.value, dir);
  }
  return { dirs, defaultDir, remember };
}
