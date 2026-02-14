/**
 * 设置状态管理
 */

import { defineStore } from 'pinia';
import { ref, computed, watch } from 'vue';
import type { AppSettings } from '@/types';
import { DEFAULT_SETTINGS } from '@/utils/constants';
import { configService } from '@/services';
import { setLocale } from '@/locales';

export const useSettingsStore = defineStore('settings', () => {
  // State
  const settings = ref<AppSettings>(JSON.parse(JSON.stringify(DEFAULT_SETTINGS)));
  const isLoading = ref(false);
  const isLoaded = ref(false);  // 标记是否已从存储加载
  const isDirty = ref(false);
  const error = ref<string | null>(null);

  // Computed
  const theme = computed(() => settings.value.ui.theme);

  // Watch for changes - 使用防抖保存（仅在已加载后生效）
  let saveTimeout: ReturnType<typeof setTimeout> | null = null;
  watch(
    settings,
    () => {
      // 只有在已加载后才触发自动保存，避免初始化时覆盖用户设置
      if (!isLoaded.value) return;

      isDirty.value = true;
      // 自动保存（防抖）
      if (saveTimeout) clearTimeout(saveTimeout);
      saveTimeout = setTimeout(() => {
        saveSettings().catch(console.error);
      }, 1000);
    },
    { deep: true }
  );

  // Actions
  /**
   * 直接设置配置（用于从存储加载后更新）
   */
  function setSettings(newSettings: AppSettings): void {
    settings.value = newSettings;
    isDirty.value = false;
  }

  async function loadSettings(): Promise<void> {
    isLoading.value = true;
    error.value = null;

    try {
      // 从 Tauri 后端加载配置
      const loaded = await configService.loadSettings();
      settings.value = loaded;
      isLoaded.value = true;  // 标记已加载
      // 应用语言设置
      setLocale(loaded.general.language);
    } catch (e) {
      error.value = e instanceof Error ? e.message : '加载配置失败';
      console.error('Failed to load settings:', e);
      // 加载失败时使用默认配置
      settings.value = JSON.parse(JSON.stringify(DEFAULT_SETTINGS));
      isLoaded.value = true;  // 即使失败也标记为已加载，避免阻止后续操作
    } finally {
      isLoading.value = false;
      isDirty.value = false;
    }
  }

  async function saveSettings(): Promise<void> {
    if (!isDirty.value) return;

    error.value = null;

    try {
      await configService.saveSettings(settings.value, true);
      isDirty.value = false;
    } catch (e) {
      error.value = e instanceof Error ? e.message : '保存配置失败';
      console.error('Failed to save settings:', e);
      throw e;
    }
  }

  async function resetSettings(): Promise<void> {
    const defaultSettings = await configService.resetSettings();
    settings.value = defaultSettings;
  }

  function resetSection<K extends keyof AppSettings>(section: K): void {
    settings.value[section] = JSON.parse(JSON.stringify(DEFAULT_SETTINGS[section]));
    isDirty.value = true;
  }

  function updateSettings<K extends keyof AppSettings>(
    section: K,
    value: Partial<AppSettings[K]>
  ): void {
    settings.value[section] = {
      ...settings.value[section],
      ...value,
    };
  }

  function updateGeneral(value: Partial<AppSettings['general']>): void {
    updateSettings('general', value);
  }

  function updateDownload(value: Partial<AppSettings['download']>): void {
    updateSettings('download', value);
  }

  function updateMux(value: Partial<AppSettings['mux']>): void {
    updateSettings('mux', value);
  }

  function updateNetwork(value: Partial<AppSettings['network']>): void {
    updateSettings('network', value);
  }

  function updateLive(value: Partial<AppSettings['live']>): void {
    updateSettings('live', value);
  }

  function updateDecryption(value: Partial<AppSettings['decryption']>): void {
    updateSettings('decryption', value);
  }

  function updateAdvanced(value: Partial<AppSettings['advanced']>): void {
    updateSettings('advanced', value);
  }

  function updateUi(value: Partial<AppSettings['ui']>): void {
    updateSettings('ui', value);
  }

  // 保存目录
  function setSaveDir(dir: string): void {
    settings.value.general.saveDir = dir;
  }

  // 临时目录
  function setTmpDir(dir: string): void {
    settings.value.general.tmpDir = dir;
  }

  // 语言
  function setLanguage(lang: AppSettings['general']['language']): void {
    settings.value.general.language = lang;
    setLocale(lang);
  }

  // 主题
  function setTheme(theme: AppSettings['ui']['theme']): void {
    settings.value.ui.theme = theme;
    applyTheme(theme);
  }

  // 应用主题
  function applyTheme(theme: AppSettings['ui']['theme']): void {
    const root = document.documentElement;

    if (theme === 'system') {
      const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
      root.classList.toggle('dark', prefersDark);
    } else {
      root.classList.toggle('dark', theme === 'dark');
    }
  }

  // 初始化主题
  function initTheme(): void {
    applyTheme(settings.value.ui.theme);

    // 监听系统主题变化
    window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (e) => {
      if (settings.value.ui.theme === 'system') {
        document.documentElement.classList.toggle('dark', e.matches);
      }
    });
  }

  // 添加请求头
  function addHeader(key: string, value: string): void {
    settings.value.network.headers.push({
      key,
      value,
      enabled: true,
    });
  }

  // 移除请求头
  function removeHeader(index: number): void {
    settings.value.network.headers.splice(index, 1);
  }

  // 更新请求头
  function updateHeader(index: number, key: string, value: string): void {
    if (settings.value.network.headers[index]) {
      settings.value.network.headers[index].key = key;
      settings.value.network.headers[index].value = value;
    }
  }

  // 切换请求头启用状态
  function toggleHeader(index: number): void {
    if (settings.value.network.headers[index]) {
      settings.value.network.headers[index].enabled = !settings.value.network.headers[index].enabled;
    }
  }

  return {
    // State
    settings,
    isLoading,
    isLoaded,
    isDirty,
    error,

    // Computed
    theme,

    // Actions
    setSettings,
    loadSettings,
    saveSettings,
    resetSettings,
    resetSection,
    updateSettings,
    updateGeneral,
    updateDownload,
    updateMux,
    updateNetwork,
    updateLive,
    updateDecryption,
    updateAdvanced,
    updateUi,
    setSaveDir,
    setTmpDir,
    setLanguage,
    setTheme,
    initTheme,
    addHeader,
    removeHeader,
    updateHeader,
    toggleHeader,
  };
});
