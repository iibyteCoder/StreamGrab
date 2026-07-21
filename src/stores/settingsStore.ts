/**
 * 设置状态管理
 *
 * 三层配置：AppSettings / Nm3u8dlConfig / FfmpegConfig
 * 全部从 @/domain 导入类型；增量更新走 patch 接口。
 */

import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type {
  AppSettings,
  Nm3u8dlConfig,
  FfmpegConfig,
  Theme,
  Language,
} from "@/domain";
import {
  DEFAULT_APP_SETTINGS,
  DEFAULT_NM3U8DL_CONFIG,
  DEFAULT_FFMPEG_CONFIG,
} from "@/domain";
import { settingsService, type DeepPartial } from "@/services";
import { systemService } from "@/services";
import { setLocale } from "@/locales";

export const useSettingsStore = defineStore("settings", () => {
  // ========================================
  // State
  // ========================================

  const appSettings = ref<AppSettings>({ ...DEFAULT_APP_SETTINGS });
  const nm3u8dlConfig = ref<Nm3u8dlConfig>({ ...DEFAULT_NM3U8DL_CONFIG });
  const ffmpegConfig = ref<FfmpegConfig>({ ...DEFAULT_FFMPEG_CONFIG });
  const loaded = ref(false);
  const loading = ref(false);

  // ========================================
  // Computed — 常用字段便捷访问
  // ========================================

  const defaultSaveDir = computed(() => appSettings.value.default_save_dir);
  const autoStartDownload = computed(
    () => appSettings.value.auto_start_download,
  );
  const clipboardWatchEnabled = computed(
    () => appSettings.value.clipboard_watch,
  );
  const showNotification = computed(() => appSettings.value.show_notification);
  const minimizeToTray = computed(() => appSettings.value.minimize_to_tray);
  const theme = computed<Theme>(() => appSettings.value.theme);
  const language = computed<Language>(() => appSettings.value.language);
  const checkUpdate = computed(() => appSettings.value.check_update);

  // ========================================
  // Actions — 加载
  // ========================================

  /** 并行加载三组配置 */
  async function loadSettings(): Promise<void> {
    loading.value = true;
    try {
      const [app, nm3u8dl, ffmpeg] = await Promise.all([
        settingsService.getAppSettings(),
        settingsService.getNm3u8dlConfig(),
        settingsService.getFfmpegConfig(),
      ]);

      // 防御性兜底：后端异常返回 null 时回落默认值（正常路径后端保证完整配置）
      appSettings.value = app ?? { ...DEFAULT_APP_SETTINGS };
      nm3u8dlConfig.value = nm3u8dl ?? { ...DEFAULT_NM3U8DL_CONFIG };
      ffmpegConfig.value = ffmpeg ?? { ...DEFAULT_FFMPEG_CONFIG };

      loaded.value = true;

      // 应用副作用
      applyLanguage(app.language);
      applyTheme(app.theme);
    } catch (e) {
      console.error("Failed to load settings:", e);
      // 使用默认值
      appSettings.value = { ...DEFAULT_APP_SETTINGS };
      nm3u8dlConfig.value = { ...DEFAULT_NM3U8DL_CONFIG };
      ffmpegConfig.value = { ...DEFAULT_FFMPEG_CONFIG };
      loaded.value = true;
    } finally {
      loading.value = false;
    }
  }

  // ========================================
  // Actions — 更新
  // ========================================

  /** 部分更新应用设置：patch 后端 → 更新 state → 应用副作用 */
  async function updateAppSettings(
    partial: DeepPartial<AppSettings>,
  ): Promise<void> {
    const merged = await settingsService.patchAppSettings(partial);
    const oldTheme = appSettings.value.theme;
    const oldLang = appSettings.value.language;
    appSettings.value = merged;

    // 副作用：主题
    if (merged.theme !== oldTheme) {
      applyTheme(merged.theme);
    }
    // 副作用：语言
    if (merged.language !== oldLang) {
      applyLanguage(merged.language);
    }
  }

  /** 部分更新 N_m3u8DL-RE 配置 */
  async function updateNm3u8dlConfig(
    partial: DeepPartial<Nm3u8dlConfig>,
  ): Promise<void> {
    const merged = await settingsService.patchNm3u8dlConfig(partial);
    nm3u8dlConfig.value = merged;
  }

  /** 部分更新 FFmpeg 配置 */
  async function updateFfmpegConfig(
    partial: DeepPartial<FfmpegConfig>,
  ): Promise<void> {
    const merged = await settingsService.patchFfmpegConfig(partial);
    ffmpegConfig.value = merged;
  }

  // ========================================
  // Actions — 导入导出
  // ========================================

  /** 导出全部配置为 JSON 文件 */
  async function exportConfig(): Promise<void> {
    const dir = await systemService.selectDirectory();
    if (!dir) return;

    const config = await settingsService.exportConfig();
    const json = JSON.stringify(config, null, 2);

    // 浏览器 Blob 下载（Tauri webview 自动保存到下载目录）
    const blob = new Blob([json], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `streamgrab-config-${Date.now()}.json`;
    a.click();
    URL.revokeObjectURL(url);
  }

  /** 从 JSON 文件导入配置 */
  async function importConfig(filePath: string): Promise<void> {
    await settingsService.importConfig(filePath);
    await loadSettings();
  }

  // ========================================
  // Actions — 副作用
  // ========================================

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

  function applyLanguage(lang: Language): void {
    setLocale(lang);
  }

  /** 初始化主题监听（在应用启动时调用一次） */
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
  // Actions — 重置
  // ========================================

  /** 重置全部配置为默认值 */
  async function resetSettings(): Promise<void> {
    await Promise.all([
      settingsService.patchAppSettings(DEFAULT_APP_SETTINGS),
      settingsService.patchNm3u8dlConfig(DEFAULT_NM3U8DL_CONFIG),
      settingsService.patchFfmpegConfig(DEFAULT_FFMPEG_CONFIG),
    ]);

    appSettings.value = { ...DEFAULT_APP_SETTINGS };
    nm3u8dlConfig.value = { ...DEFAULT_NM3U8DL_CONFIG };
    ffmpegConfig.value = { ...DEFAULT_FFMPEG_CONFIG };

    applyTheme(DEFAULT_APP_SETTINGS.theme);
    applyLanguage(DEFAULT_APP_SETTINGS.language);
  }

  return {
    // State
    appSettings,
    nm3u8dlConfig,
    ffmpegConfig,
    loaded,
    loading,

    // Computed
    defaultSaveDir,
    autoStartDownload,
    clipboardWatchEnabled,
    showNotification,
    minimizeToTray,
    theme,
    language,
    checkUpdate,

    // Actions
    loadSettings,
    updateAppSettings,
    updateNm3u8dlConfig,
    updateFfmpegConfig,
    exportConfig,
    importConfig,
    initTheme,
    resetSettings,
  };
});
