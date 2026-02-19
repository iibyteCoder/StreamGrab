/**
 * 设置状态管理
 * 修改配置时立即保存到数据库
 */

import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type { AppSettings } from "@/types";
import { DEFAULT_SETTINGS } from "@/utils/constants";
import { configService } from "@/services";
import { setLocale } from "@/locales";

export const useSettingsStore = defineStore("settings", () => {
  // State
  const settings = ref<AppSettings>(
    JSON.parse(JSON.stringify(DEFAULT_SETTINGS)),
  );
  const isLoading = ref(false);
  const isLoaded = ref(false);
  const error = ref<string | null>(null);

  // Computed
  const theme = computed(() => settings.value.ui.theme);

  // Actions

  /**
   * 加载配置
   */
  async function loadSettings(): Promise<void> {
    isLoading.value = true;
    error.value = null;

    try {
      const loaded = await configService.loadSettings();
      settings.value = loaded;
      isLoaded.value = true;
      setLocale(loaded.general.language);
    } catch (e) {
      error.value = e instanceof Error ? e.message : "加载配置失败";
      console.error("Failed to load settings:", e);
      settings.value = JSON.parse(JSON.stringify(DEFAULT_SETTINGS));
      isLoaded.value = true;
    } finally {
      isLoading.value = false;
    }
  }

  /**
   * 更新配置模块并立即保存
   */
  async function updateSection<K extends keyof AppSettings>(
    section: K,
    value: Partial<AppSettings[K]>,
  ): Promise<void> {
    // 更新本地状态
    settings.value[section] = {
      ...settings.value[section],
      ...value,
    } as AppSettings[K];

    // 立即保存到数据库
    try {
      await configService.saveSetting(section, settings.value[section]);
    } catch (e) {
      console.error(`Failed to save ${section} settings:`, e);
      throw e;
    }
  }

  // 便捷方法
  const updateGeneral = (value: Partial<AppSettings["general"]>) =>
    updateSection("general", value);

  const updateDownload = (value: Partial<AppSettings["download"]>) =>
    updateSection("download", value);

  const updateMux = (value: Partial<AppSettings["mux"]>) =>
    updateSection("mux", value);

  const updateNetwork = (value: Partial<AppSettings["network"]>) =>
    updateSection("network", value);

  const updateLive = (value: Partial<AppSettings["live"]>) =>
    updateSection("live", value);

  const updateDecryption = (value: Partial<AppSettings["decryption"]>) =>
    updateSection("decryption", value);

  const updateAdvanced = (value: Partial<AppSettings["advanced"]>) =>
    updateSection("advanced", value);

  const updateUi = (value: Partial<AppSettings["ui"]>) =>
    updateSection("ui", value);

  /**
   * 设置保存目录
   */
  const setSaveDir = (dir: string) => updateGeneral({ saveDir: dir });

  /**
   * 设置临时目录
   */
  const setTmpDir = (dir: string) => updateGeneral({ tmpDir: dir });

  /**
   * 设置语言
   */
  const setLanguage = (lang: AppSettings["general"]["language"]) => {
    updateGeneral({ language: lang });
    setLocale(lang);
  };

  /**
   * 设置主题
   */
  const setTheme = (newTheme: AppSettings["ui"]["theme"]) => {
    updateUi({ theme: newTheme });
    applyTheme(newTheme);
  };

  /**
   * 应用主题
   */
  function applyTheme(theme: AppSettings["ui"]["theme"]): void {
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

  /**
   * 初始化主题
   */
  function initTheme(): void {
    applyTheme(settings.value.ui.theme);
    window
      .matchMedia("(prefers-color-scheme: dark)")
      .addEventListener("change", (e) => {
        if (settings.value.ui.theme === "system") {
          document.documentElement.classList.toggle("dark", e.matches);
        }
      });
  }

  /**
   * 重置所有配置
   */
  async function resetSettings(): Promise<void> {
    await configService.resetAllSettings();
    settings.value = JSON.parse(JSON.stringify(DEFAULT_SETTINGS));
  }

  /**
   * 重置单个配置模块
   */
  async function resetSection<K extends keyof AppSettings>(
    section: K,
  ): Promise<void> {
    await configService.resetSetting(section);
    settings.value[section] = JSON.parse(
      JSON.stringify(DEFAULT_SETTINGS[section]),
    );
  }

  /**
   * 设置配置（用于导入配置后）
   */
  async function setSettings(newSettings: AppSettings): Promise<void> {
    settings.value = newSettings;
    await configService.saveAllSettings(newSettings);
  }

  // 请求头操作
  function addHeader(key: string, value: string): void {
    settings.value.network.headers.push({ key, value, enabled: true });
    updateSection("network", { headers: settings.value.network.headers });
  }

  function removeHeader(index: number): void {
    settings.value.network.headers.splice(index, 1);
    updateSection("network", { headers: settings.value.network.headers });
  }

  function updateHeader(index: number, key: string, value: string): void {
    if (settings.value.network.headers[index]) {
      settings.value.network.headers[index].key = key;
      settings.value.network.headers[index].value = value;
      updateSection("network", { headers: settings.value.network.headers });
    }
  }

  function toggleHeader(index: number): void {
    if (settings.value.network.headers[index]) {
      settings.value.network.headers[index].enabled =
        !settings.value.network.headers[index].enabled;
      updateSection("network", { headers: settings.value.network.headers });
    }
  }

  return {
    // State
    settings,
    isLoading,
    isLoaded,
    error,

    // Computed
    theme,

    // Actions
    loadSettings,
    setSettings,
    resetSettings,
    resetSection,

    // 更新方法
    updateSection,
    updateGeneral,
    updateDownload,
    updateMux,
    updateNetwork,
    updateLive,
    updateDecryption,
    updateAdvanced,
    updateUi,

    // 便捷方法
    setSaveDir,
    setTmpDir,
    setLanguage,
    setTheme,
    initTheme,

    // 请求头操作
    addHeader,
    removeHeader,
    updateHeader,
    toggleHeader,
  };
});
