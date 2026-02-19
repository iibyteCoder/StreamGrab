/**
 * 设置管理组合式函数
 * 封装设置的加载和更新
 */

import { computed, onMounted } from "vue";
import { useSettingsStore } from "@/stores";
import { configService } from "@/services";
import type { AppSettings } from "@/types";

/**
 * 设置组合式函数
 */
export function useSettings() {
  const store = useSettingsStore();

  // Computed
  const settings = computed(() => store.settings);
  const theme = computed(() => store.theme);
  const isLoaded = computed(() => store.isLoaded);

  /**
   * 加载设置
   */
  const loadSettings = (): Promise<void> => store.loadSettings();

  /**
   * 重置设置
   */
  const resetSettings = (): Promise<void> => store.resetSettings();

  // 更新方法 - 直接调用 store，会自动保存到数据库
  const updateGeneral = (value: Partial<AppSettings["general"]>) =>
    store.updateGeneral(value);

  const updateDownload = (value: Partial<AppSettings["download"]>) =>
    store.updateDownload(value);

  const updateMux = (value: Partial<AppSettings["mux"]>) =>
    store.updateMux(value);

  const updateNetwork = (value: Partial<AppSettings["network"]>) =>
    store.updateNetwork(value);

  const updateDecryption = (value: Partial<AppSettings["decryption"]>) =>
    store.updateDecryption(value);

  const updateLive = (value: Partial<AppSettings["live"]>) =>
    store.updateLive(value);

  const updateAdvanced = (value: Partial<AppSettings["advanced"]>) =>
    store.updateAdvanced(value);

  const updateUi = (value: Partial<AppSettings["ui"]>) => store.updateUi(value);

  const setTheme = (newTheme: "light" | "dark" | "system") =>
    store.setTheme(newTheme);

  /**
   * 导出配置
   */
  const exportConfig = (filePath: string): Promise<void> =>
    configService.exportConfig(filePath);

  /**
   * 导入配置
   */
  const importConfig = async (filePath: string): Promise<void> => {
    const imported = await configService.importConfig(filePath);
    await store.setSettings(imported);
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
    settings,
    theme,
    isLoaded,

    // Actions
    loadSettings,
    resetSettings,
    setTheme,

    // 更新方法
    updateGeneral,
    updateDownload,
    updateMux,
    updateNetwork,
    updateDecryption,
    updateLive,
    updateAdvanced,
    updateUi,

    // 导入导出
    exportConfig,
    importConfig,
  };
}
