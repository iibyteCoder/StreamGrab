/**
 * 设置状态管理
 */

import { defineStore } from 'pinia';
import { ref, watch } from 'vue';
import type { AppSettings } from '@/types';
import { DEFAULT_SETTINGS } from '@/utils/constants';

export const useSettingsStore = defineStore('settings', () => {
  // State
  const settings = ref<AppSettings>(JSON.parse(JSON.stringify(DEFAULT_SETTINGS)));
  const isLoading = ref(false);
  const isDirty = ref(false);
  const error = ref<string | null>(null);

  // Watch for changes
  watch(
    settings,
    () => {
      isDirty.value = true;
    },
    { deep: true }
  );

  // Actions
  async function loadSettings(): Promise<void> {
    isLoading.value = true;
    error.value = null;

    try {
      // 尝试从 Tauri 后端加载配置
      // TODO: 实现 Tauri 配置加载
      // const loaded = await invoke<AppSettings>('load_settings');
      // settings.value = { ...DEFAULT_SETTINGS, ...loaded };

      // 暂时使用 localStorage
      const saved = localStorage.getItem('streamgrab-settings');
      if (saved) {
        const parsed = JSON.parse(saved);
        settings.value = deepMerge(JSON.parse(JSON.stringify(DEFAULT_SETTINGS)), parsed);
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : '加载配置失败';
      console.error('Failed to load settings:', e);
    } finally {
      isLoading.value = false;
      isDirty.value = false;
    }
  }

  async function saveSettings(): Promise<void> {
    error.value = null;

    try {
      // TODO: 实现 Tauri 配置保存
      // await invoke('save_settings', { settings: settings.value });

      // 暂时使用 localStorage
      localStorage.setItem('streamgrab-settings', JSON.stringify(settings.value));
      isDirty.value = false;
    } catch (e) {
      error.value = e instanceof Error ? e.message : '保存配置失败';
      console.error('Failed to save settings:', e);
      throw e;
    }
  }

  function resetSettings(): void {
    settings.value = JSON.parse(JSON.stringify(DEFAULT_SETTINGS));
    isDirty.value = true;
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
    isDirty,
    error,

    // Actions
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

// Helper function for deep merge
function deepMerge<T extends Record<string, unknown>>(target: T, source: Partial<T>): T {
  const result = { ...target };

  for (const key in source) {
    if (Object.prototype.hasOwnProperty.call(source, key)) {
      const sourceValue = source[key];
      const targetValue = result[key];

      if (
        sourceValue !== null &&
        typeof sourceValue === 'object' &&
        !Array.isArray(sourceValue) &&
        targetValue !== null &&
        typeof targetValue === 'object' &&
        !Array.isArray(targetValue)
      ) {
        result[key] = deepMerge(
          targetValue as Record<string, unknown>,
          sourceValue as Record<string, unknown>
        ) as T[Extract<keyof T, string>];
      } else {
        result[key] = sourceValue as T[Extract<keyof T, string>];
      }
    }
  }

  return result;
}
