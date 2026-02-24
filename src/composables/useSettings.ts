/**
 * 设置管理组合式函数
 * 封装设置的加载和更新
 */

import { computed, onMounted } from "vue";
import { useSettingsStore } from "@/stores";
import { configService } from "@/services";
import type {
  AppSettings,
  M3U8DLSettings,
  FFmpegSettings,
  NetworkSettings,
  DecryptionSettings,
  Theme,
  Language,
} from "@/domain/config";

/**
 * 设置组合式函数
 */
export function useSettings() {
  const store = useSettingsStore();

  // Computed
  const appSettings = computed(() => store.appSettings);
  const m3u8dlSettings = computed(() => store.m3u8dlSettings);
  const ffmpegSettings = computed(() => store.ffmpegSettings);
  const networkSettings = computed(() => store.networkSettings);
  const decryptionSettings = computed(() => store.decryptionSettings);
  const networkHeaders = computed(() => store.networkHeaders);
  const decryptionKeys = computed(() => store.decryptionKeys);

  const theme = computed(() => store.theme);
  const isLoaded = computed(() => store.isLoaded);

  // Actions
  const loadSettings = (): Promise<void> => store.loadSettings();
  const resetSettings = (): Promise<void> => store.resetSettings();
  const setTheme = (newTheme: Theme) => store.setTheme(newTheme);
  const setLanguage = (lang: Language) => store.setLanguage(lang);

  // 字段更新方法
  const updateAppField = <K extends keyof AppSettings>(
    field: K,
    value: AppSettings[K],
  ) => store.updateAppField(field, value);

  const updateM3U8DLField = <K extends keyof M3U8DLSettings>(
    field: K,
    value: M3U8DLSettings[K],
  ) => store.updateM3U8DLField(field, value);

  const updateFFmpegField = <K extends keyof FFmpegSettings>(
    field: K,
    value: FFmpegSettings[K],
  ) => store.updateFFmpegField(field, value);

  const updateNetworkField = <K extends keyof NetworkSettings>(
    field: K,
    value: NetworkSettings[K],
  ) => store.updateNetworkField(field, value);

  const updateDecryptionField = <K extends keyof DecryptionSettings>(
    field: K,
    value: DecryptionSettings[K],
  ) => store.updateDecryptionField(field, value);

  // 导入导出
  const exportConfig = (filePath: string): Promise<void> =>
    configService.exportConfig(filePath);

  const importConfig = async (filePath: string): Promise<void> => {
    await store.importConfig(filePath);
    store.initTheme();
  };

  // 组件挂载时加载设置
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
    m3u8dlSettings,
    ffmpegSettings,
    networkSettings,
    decryptionSettings,
    networkHeaders,
    decryptionKeys,

    // Computed
    theme,
    isLoaded,

    // Actions
    loadSettings,
    resetSettings,
    setTheme,
    setLanguage,

    // 字段更新
    updateAppField,
    updateM3U8DLField,
    updateFFmpegField,
    updateNetworkField,
    updateDecryptionField,

    // 导入导出
    exportConfig,
    importConfig,
  };
}
