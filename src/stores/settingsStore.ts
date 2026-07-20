/**
 * 设置状态管理
 *
 * 使用新的配置结构，支持字段级别更新
 */

import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type {
  AppSettings,
  M3U8DLSettings,
  FFmpegSettings,
  NetworkSettings,
  DecryptionSettings,
  NetworkHeader,
  DecryptionKey,
  Theme,
  Language,
  AllConfig,
} from "@/domain/config";
import {
  DEFAULT_APP_SETTINGS,
  DEFAULT_M3U8DL_SETTINGS,
  DEFAULT_FFMPEG_SETTINGS,
  DEFAULT_NETWORK_SETTINGS,
  DEFAULT_DECRYPTION_SETTINGS,
} from "@/domain/config";
import { configService, type ConfigTable } from "@/services/configService";
import { setLocale } from "@/locales";

export const useSettingsStore = defineStore("settings", () => {
  // ========================================
  // State
  // ========================================

  const appSettings = ref<AppSettings>({ ...DEFAULT_APP_SETTINGS });
  const m3u8dlSettings = ref<M3U8DLSettings>({ ...DEFAULT_M3U8DL_SETTINGS });
  const ffmpegSettings = ref<FFmpegSettings>({ ...DEFAULT_FFMPEG_SETTINGS });
  const networkSettings = ref<NetworkSettings>({ ...DEFAULT_NETWORK_SETTINGS });
  const decryptionSettings = ref<DecryptionSettings>({
    ...DEFAULT_DECRYPTION_SETTINGS,
  });

  const networkHeaders = ref<NetworkHeader[]>([]);
  const decryptionKeys = ref<DecryptionKey[]>([]);

  const isLoading = ref(false);
  const isLoaded = ref(false);
  const error = ref<string | null>(null);

  // ========================================
  // Computed
  // ========================================

  const theme = computed<Theme>(() => appSettings.value.theme);
  const m3u8dlPath = computed(() => m3u8dlSettings.value.n_m3u8dl_path);
  const ffmpegPath = computed(() => ffmpegSettings.value.ffmpeg_path);
  const ffprobePath = computed(() => ffmpegSettings.value.ffprobe_path);

  // ========================================
  // Actions
  // ========================================

  async function loadSettings(): Promise<void> {
    isLoading.value = true;
    error.value = null;

    try {
      const config = await configService.loadAllConfig();

      appSettings.value = config.app;
      m3u8dlSettings.value = config.m3u8dl;
      ffmpegSettings.value = config.ffmpeg;
      networkSettings.value = config.network;
      decryptionSettings.value = config.decryption;
      networkHeaders.value = config.headers;
      decryptionKeys.value = config.keys;

      isLoaded.value = true;
      setLocale(appSettings.value.language);
      applyTheme(appSettings.value.theme);
    } catch (e) {
      error.value = e instanceof Error ? e.message : "加载配置失败";
      console.error("Failed to load settings:", e);

      // 使用默认值
      appSettings.value = { ...DEFAULT_APP_SETTINGS };
      m3u8dlSettings.value = { ...DEFAULT_M3U8DL_SETTINGS };
      ffmpegSettings.value = { ...DEFAULT_FFMPEG_SETTINGS };
      networkSettings.value = { ...DEFAULT_NETWORK_SETTINGS };
      decryptionSettings.value = { ...DEFAULT_DECRYPTION_SETTINGS };
      networkHeaders.value = [];
      decryptionKeys.value = [];

      isLoaded.value = true;
    } finally {
      isLoading.value = false;
    }
  }

  async function updateAppField<K extends keyof AppSettings>(
    field: K,
    value: AppSettings[K],
  ): Promise<void> {
    const oldValue = appSettings.value[field];
    appSettings.value[field] = value;

    try {
      await configService.updateAppSettingField(
        field,
        value as string | boolean,
      );
    } catch (e) {
      appSettings.value[field] = oldValue;
      throw e;
    }
  }

  async function updateM3U8DLField<K extends keyof M3U8DLSettings>(
    field: K,
    value: M3U8DLSettings[K],
  ): Promise<void> {
    const oldValue = m3u8dlSettings.value[field];
    m3u8dlSettings.value[field] = value;

    try {
      await configService.updateM3U8DLSettingField(field, value);
    } catch (e) {
      m3u8dlSettings.value[field] = oldValue;
      throw e;
    }
  }

  async function updateFFmpegField<K extends keyof FFmpegSettings>(
    field: K,
    value: FFmpegSettings[K],
  ): Promise<void> {
    const oldValue = ffmpegSettings.value[field];
    ffmpegSettings.value[field] = value;

    try {
      await configService.updateFFmpegSettingField(field, value);
    } catch (e) {
      ffmpegSettings.value[field] = oldValue;
      throw e;
    }
  }

  async function updateNetworkField<K extends keyof NetworkSettings>(
    field: K,
    value: NetworkSettings[K],
  ): Promise<void> {
    const oldValue = networkSettings.value[field];
    networkSettings.value[field] = value;

    try {
      await configService.updateNetworkSettingField(field, value);
    } catch (e) {
      networkSettings.value[field] = oldValue;
      throw e;
    }
  }

  async function updateDecryptionField<K extends keyof DecryptionSettings>(
    field: K,
    value: DecryptionSettings[K],
  ): Promise<void> {
    const oldValue = decryptionSettings.value[field];
    decryptionSettings.value[field] = value;

    try {
      await configService.updateDecryptionSettingField(field, value);
    } catch (e) {
      decryptionSettings.value[field] = oldValue;
      throw e;
    }
  }

  // ========================================
  // 便捷方法
  // ========================================

  const setSaveDir = (dir: string) => updateAppField("default_save_dir", dir);
  const setTmpDir = (dir: string) => updateAppField("default_tmp_dir", dir);

  const setLanguage = (lang: Language) => {
    updateAppField("language", lang);
    setLocale(lang);
  };

  const setTheme = (newTheme: Theme) => {
    updateAppField("theme", newTheme);
    applyTheme(newTheme);
  };

  function applyTheme(theme: Theme): void {
    const root = document.documentElement;
    if (theme === "system") {
      const prefersDark = window.matchMedia(
        "(prefers-color-scheme: dark)",
      ).matches;
      root.classList.toggle("dark", prefersDark);
    } else {
      root.classList.toggle("dark", theme === "dark");
    }
  }

  function initTheme(): void {
    applyTheme(appSettings.value.theme);
    window
      .matchMedia("(prefers-color-scheme: dark)")
      .addEventListener("change", (e) => {
        if (appSettings.value.theme === "system") {
          document.documentElement.classList.toggle("dark", e.matches);
        }
      });
  }

  // ========================================
  // 网络请求头操作
  // ========================================

  async function addHeader(name: string, value: string): Promise<void> {
    const id = await configService.addNetworkHeader(name, value);
    networkHeaders.value.push({
      id,
      name,
      value,
      enabled: true,
      sort_order: networkHeaders.value.length,
    });
  }

  async function removeHeader(id: number): Promise<void> {
    const index = networkHeaders.value.findIndex((h) => h.id === id);
    if (index !== -1) {
      await configService.deleteNetworkHeader(id);
      networkHeaders.value.splice(index, 1);
    }
  }

  async function updateHeader(
    id: number,
    name: string,
    value: string,
    enabled: boolean,
  ): Promise<void> {
    const header = networkHeaders.value.find((h) => h.id === id);
    if (header) {
      await configService.updateNetworkHeader(id, name, value, enabled);
      header.name = name;
      header.value = value;
      header.enabled = enabled;
    }
  }

  async function toggleHeader(id: number): Promise<void> {
    const header = networkHeaders.value.find((h) => h.id === id);
    if (header) {
      await updateHeader(id, header.name, header.value, !header.enabled);
    }
  }

  // ========================================
  // 解密密钥操作
  // ========================================

  async function addDecryptionKey(
    kid: string | null,
    key: string,
  ): Promise<void> {
    const id = await configService.addDecryptionKey(kid, key);
    decryptionKeys.value.push({
      id,
      kid,
      key,
      sort_order: decryptionKeys.value.length,
    });
  }

  async function removeDecryptionKey(id: number): Promise<void> {
    const index = decryptionKeys.value.findIndex((k) => k.id === id);
    if (index !== -1) {
      await configService.deleteDecryptionKey(id);
      decryptionKeys.value.splice(index, 1);
    }
  }

  // ========================================
  // 导入/导出
  // ========================================

  async function exportConfig(filePath: string): Promise<void> {
    await configService.exportConfig(filePath);
  }

  async function importConfig(filePath: string): Promise<void> {
    const config = await configService.importConfig(filePath);

    appSettings.value = config.app;
    m3u8dlSettings.value = config.m3u8dl;
    ffmpegSettings.value = config.ffmpeg;
    networkSettings.value = config.network;
    decryptionSettings.value = config.decryption;
    networkHeaders.value = config.headers;
    decryptionKeys.value = config.keys;

    setLocale(appSettings.value.language);
    applyTheme(appSettings.value.theme);
  }

  // ========================================
  // 重置
  // ========================================

  /** 将某个配置表的默认值逐字段写回数据库 */
  async function persistDefaults<T extends object>(
    table: ConfigTable,
    defaults: T,
  ): Promise<void> {
    for (const [field, value] of Object.entries(defaults)) {
      await configService.updateSettingField(
        table,
        field,
        value as string | number | boolean | null,
      );
    }
  }

  async function resetSettings(): Promise<void> {
    // 1. 删除所有网络请求头和解密密钥（持久化到数据库）
    for (const id of networkHeaders.value.map((h) => h.id)) {
      await removeHeader(id);
    }
    for (const id of decryptionKeys.value.map((k) => k.id)) {
      await removeDecryptionKey(id);
    }

    // 2. 将各配置表重置为默认值并写回数据库
    await persistDefaults("app", DEFAULT_APP_SETTINGS);
    await persistDefaults("m3u8dl", DEFAULT_M3U8DL_SETTINGS);
    await persistDefaults("ffmpeg", DEFAULT_FFMPEG_SETTINGS);
    await persistDefaults("network", DEFAULT_NETWORK_SETTINGS);
    await persistDefaults("decryption", DEFAULT_DECRYPTION_SETTINGS);

    // 3. 更新内存状态
    appSettings.value = { ...DEFAULT_APP_SETTINGS };
    m3u8dlSettings.value = { ...DEFAULT_M3U8DL_SETTINGS };
    ffmpegSettings.value = { ...DEFAULT_FFMPEG_SETTINGS };
    networkSettings.value = { ...DEFAULT_NETWORK_SETTINGS };
    decryptionSettings.value = { ...DEFAULT_DECRYPTION_SETTINGS };
    networkHeaders.value = [];
    decryptionKeys.value = [];

    // 4. 应用重置后的语言和主题
    setLocale(appSettings.value.language);
    applyTheme(appSettings.value.theme);
  }

  // ========================================
  // 获取完整配置（用于下载等场景）
  // ========================================

  function getAllConfig(): AllConfig {
    return {
      app: { ...appSettings.value },
      m3u8dl: { ...m3u8dlSettings.value },
      ffmpeg: { ...ffmpegSettings.value },
      network: { ...networkSettings.value },
      decryption: { ...decryptionSettings.value },
      headers: [...networkHeaders.value],
      keys: [...decryptionKeys.value],
    };
  }

  return {
    // State
    appSettings,
    m3u8dlSettings,
    ffmpegSettings,
    networkSettings,
    decryptionSettings,
    networkHeaders,
    decryptionKeys,
    isLoading,
    isLoaded,
    error,

    // Computed
    theme,
    m3u8dlPath,
    ffmpegPath,
    ffprobePath,

    // Actions
    loadSettings,
    resetSettings,
    importConfig,
    exportConfig,
    getAllConfig,
    initTheme,

    // 字段更新方法
    updateAppField,
    updateM3U8DLField,
    updateFFmpegField,
    updateNetworkField,
    updateDecryptionField,

    // 便捷方法
    setSaveDir,
    setTmpDir,
    setLanguage,
    setTheme,

    // 请求头操作
    addHeader,
    removeHeader,
    updateHeader,
    toggleHeader,

    // 解密密钥操作
    addDecryptionKey,
    removeDecryptionKey,
  };
});
