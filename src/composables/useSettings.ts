/**
 * 设置管理组合式函数
 * 封装设置的加载、保存和更新
 */

import { computed, watch, onMounted, onUnmounted } from 'vue';
import { useSettingsStore } from '@/stores';
import { configService } from '@/services';
import type { AppSettings } from '@/types';

/**
 * 设置组合式函数
 */
export function useSettings() {
  const store = useSettingsStore();

  // 是否已加载
  const isLoaded = computed(() => store.isLoaded);

  // 当前主题
  const theme = computed(() => store.theme);

  // 设置对象
  const settings = computed(() => store.settings);

  /**
   * 加载设置
   */
  const loadSettings = async (): Promise<void> => {
    try {
      await store.loadSettings();
    } catch (e) {
      console.error('Failed to load settings:', e);
      throw e;
    }
  };

  /**
   * 保存设置
   */
  const saveSettings = async (): Promise<void> => {
    try {
      await store.saveSettings();
    } catch (e) {
      console.error('Failed to save settings:', e);
      throw e;
    }
  };

  /**
   * 重置设置
   */
  const resetSettings = async (): Promise<void> => {
    try {
      await store.resetSettings();
    } catch (e) {
      console.error('Failed to reset settings:', e);
      throw e;
    }
  };

  /**
   * 更新通用设置
   */
  const updateGeneral = (updates: Partial<AppSettings['general']>): void => {
    store.updateGeneral(updates);
  };

  /**
   * 更新下载设置
   */
  const updateDownload = (updates: Partial<AppSettings['download']>): void => {
    store.updateDownload(updates);
  };

  /**
   * 更新混流设置
   */
  const updateMux = (updates: Partial<AppSettings['mux']>): void => {
    store.updateMux(updates);
  };

  /**
   * 更新网络设置
   */
  const updateNetwork = (updates: Partial<AppSettings['network']>): void => {
    store.updateNetwork(updates);
  };

  /**
   * 更新解密设置
   */
  const updateDecryption = (updates: Partial<AppSettings['decryption']>): void => {
    store.updateDecryption(updates);
  };

  /**
   * 更新直播设置
   */
  const updateLive = (updates: Partial<AppSettings['live']>): void => {
    store.updateLive(updates);
  };

  /**
   * 更新高级设置
   */
  const updateAdvanced = (updates: Partial<AppSettings['advanced']>): void => {
    store.updateAdvanced(updates);
  };

  /**
   * 更新 UI 设置
   */
  const updateUi = (updates: Partial<AppSettings['ui']>): void => {
    store.updateUi(updates);
  };

  /**
   * 设置主题
   */
  const setTheme = (newTheme: 'light' | 'dark' | 'system'): void => {
    store.setTheme(newTheme);
  };

  /**
   * 导出配置
   */
  const exportConfig = async (filePath: string): Promise<void> => {
    await configService.exportConfig(filePath);
  };

  /**
   * 导入配置
   */
  const importConfig = async (filePath: string): Promise<void> => {
    const imported = await configService.importConfig(filePath);
    store.setSettings(imported);
    store.initTheme();  // 导入后重新应用主题
  };

  /**
   * 自动保存的 watch
   */
  let autoSaveStopper: ReturnType<typeof watch> | null = null;

  /**
   * 启用自动保存
   */
  const enableAutoSave = (debounceMs = 1000): void => {
    if (autoSaveStopper) return;

    let timeout: ReturnType<typeof setTimeout> | null = null;

    autoSaveStopper = watch(
      () => store.settings,
      () => {
        if (timeout) clearTimeout(timeout);
        timeout = setTimeout(() => {
          saveSettings().catch(console.error);
        }, debounceMs);
      },
      { deep: true }
    );
  };

  /**
   * 禁用自动保存
   */
  const disableAutoSave = (): void => {
    if (autoSaveStopper) {
      autoSaveStopper();
      autoSaveStopper = null;
    }
  };

  // 组件挂载时加载设置
  onMounted(async () => {
    if (!isLoaded.value) {
      try {
        await loadSettings();
        store.initTheme();
      } catch (e) {
        console.error('Failed to initialize settings:', e);
      }
    }
  });

  // 组件卸载时清理
  onUnmounted(() => {
    disableAutoSave();
  });

  return {
    // State
    settings,
    theme,
    isLoaded,

    // Actions
    loadSettings,
    saveSettings,
    resetSettings,
    setTheme,

    // Update methods
    updateGeneral,
    updateDownload,
    updateMux,
    updateNetwork,
    updateDecryption,
    updateLive,
    updateAdvanced,
    updateUi,

    // Import/Export
    exportConfig,
    importConfig,

    // Auto save
    enableAutoSave,
    disableAutoSave,
  };
}
