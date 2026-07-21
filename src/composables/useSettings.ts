/**
 * 设置管理组合式函数
 *
 * settingsStore 的薄封装，保持旧 API 名称以降低迁移成本。
 */

import { computed, onMounted } from "vue";
import { useSettingsStore } from "@/stores";
import { type DeepPartial } from "@/services";
import { systemService } from "@/services";
import type { AppSettings, Nm3u8dlConfig, FfmpegConfig } from "@/domain";

export function useSettings() {
  const store = useSettingsStore();

  // ==========================================
  // Computed
  // ==========================================

  const appSettings = computed(() => store.appSettings);
  const nm3u8dlConfig = computed(() => store.nm3u8dlConfig);
  const ffmpegConfig = computed(() => store.ffmpegConfig);
  const theme = computed(() => store.theme);
  const isLoaded = computed(() => store.loaded);

  // ==========================================
  // Actions
  // ==========================================

  const loadSettings = (): Promise<void> => store.loadSettings();
  const resetSettings = (): Promise<void> => store.resetSettings();

  const updateAppSettings = (partial: DeepPartial<AppSettings>) =>
    store.updateAppSettings(partial);

  const updateNm3u8dlConfig = (partial: DeepPartial<Nm3u8dlConfig>) =>
    store.updateNm3u8dlConfig(partial);

  const updateFfmpegConfig = (partial: DeepPartial<FfmpegConfig>) =>
    store.updateFfmpegConfig(partial);

  // ==========================================
  // 导入导出
  // ==========================================

  const exportConfig = async (): Promise<void> => {
    await store.exportConfig();
  };

  const importConfig = async (): Promise<void> => {
    const filePath = await systemService.selectFile([
      { name: "JSON", extensions: ["json"] },
    ]);
    if (!filePath) return;
    await store.importConfig(filePath);
    store.initTheme();
  };

  // ==========================================
  // 自动加载
  // ==========================================

  onMounted(async () => {
    if (!isLoaded.value) {
      try {
        await loadSettings();
        store.initTheme();
      } catch (e) {
        console.error("Failed to initialize settings:", e);
      }
    }
  });

  return {
    // State
    appSettings,
    nm3u8dlConfig,
    ffmpegConfig,

    // Computed
    theme,
    isLoaded,

    // Actions
    loadSettings,
    resetSettings,
    updateAppSettings,
    updateNm3u8dlConfig,
    updateFfmpegConfig,

    // 导入导出
    exportConfig,
    importConfig,
  };
}
