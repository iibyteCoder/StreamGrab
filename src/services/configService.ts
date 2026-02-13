/**
 * 配置服务
 * 管理应用配置的持久化
 */

import { invokeTauri } from './tauri';
import type { AppSettings } from '@/types';
import { DEFAULT_SETTINGS } from '@/utils/constants';

/**
 * 配置文件名
 */
const CONFIG_FILE_NAME = 'settings.json';

/**
 * 配置服务类
 */
class ConfigService {
  private cachedSettings: AppSettings | null = null;
  private saveTimeout: ReturnType<typeof setTimeout> | null = null;

  /**
   * 加载配置
   * 如果本地没有配置文件，返回默认配置
   */
  async loadSettings(): Promise<AppSettings> {
    try {
      const settings = await invokeTauri<AppSettings>('load_config', {
        file_name: CONFIG_FILE_NAME,
      });

      // 合并默认值（处理新增配置项）
      this.cachedSettings = this.mergeWithDefaults(settings);
      return this.cachedSettings;
    } catch (error) {
      console.warn('加载配置失败，使用默认配置:', error);
      this.cachedSettings = JSON.parse(JSON.stringify(DEFAULT_SETTINGS));
      return this.cachedSettings;
    }
  }

  /**
   * 保存配置
   * 使用防抖机制，避免频繁写入
   * @param settings 配置对象
   * @param immediate 是否立即保存
   */
  async saveSettings(settings: AppSettings, immediate = false): Promise<void> {
    this.cachedSettings = settings;

    if (immediate) {
      await this.doSave(settings);
      return;
    }

    // 防抖保存
    if (this.saveTimeout) {
      clearTimeout(this.saveTimeout);
    }

    this.saveTimeout = setTimeout(async () => {
      await this.doSave(settings);
      this.saveTimeout = null;
    }, 500);
  }

  /**
   * 执行保存操作
   */
  private async doSave(settings: AppSettings): Promise<void> {
    try {
      await invokeTauri('save_config', {
        file_name: CONFIG_FILE_NAME,
        config: settings,
      });
    } catch (error) {
      console.error('保存配置失败:', error);
      throw error;
    }
  }

  /**
   * 重置配置为默认值
   */
  async resetSettings(): Promise<AppSettings> {
    const defaultSettings = JSON.parse(JSON.stringify(DEFAULT_SETTINGS));
    await this.saveSettings(defaultSettings, true);
    this.cachedSettings = defaultSettings;
    return defaultSettings;
  }

  /**
   * 获取缓存的配置
   */
  getCachedSettings(): AppSettings | null {
    return this.cachedSettings;
  }

  /**
   * 合并默认值
   * 确保所有配置项都存在（处理版本升级时新增的配置）
   */
  private mergeWithDefaults(settings: Partial<AppSettings>): AppSettings {
    const result = JSON.parse(JSON.stringify(DEFAULT_SETTINGS)) as AppSettings;

    // 递归合并
    this.deepMerge(result, settings);

    return result;
  }

  /**
   * 深度合并对象
   */
  private deepMerge(target: Record<string, unknown>, source: Record<string, unknown> | undefined): void {
    if (!source) return;

    for (const key of Object.keys(source)) {
      if (
        key in target &&
        typeof target[key] === 'object' &&
        target[key] !== null &&
        !Array.isArray(target[key]) &&
        typeof source[key] === 'object' &&
        source[key] !== null &&
        !Array.isArray(source[key])
      ) {
        this.deepMerge(
          target[key] as Record<string, unknown>,
          source[key] as Record<string, unknown>
        );
      } else if (source[key] !== undefined) {
        target[key] = source[key];
      }
    }
  }

  /**
   * 导出配置到文件
   * @param filePath 导出路径
   */
  async exportConfig(filePath: string): Promise<void> {
    if (!this.cachedSettings) {
      throw new Error('没有可导出的配置');
    }

    await invokeTauri('export_config', {
      file_path: filePath,
      config: this.cachedSettings,
    });
  }

  /**
   * 从文件导入配置
   * @param filePath 导入路径
   */
  async importConfig(filePath: string): Promise<AppSettings> {
    const settings = await invokeTauri<AppSettings>('import_config', {
      file_path: filePath,
    });

    // 验证并合并
    this.cachedSettings = this.mergeWithDefaults(settings);

    // 保存到本地
    await this.saveSettings(this.cachedSettings, true);

    return this.cachedSettings;
  }

  /**
   * 获取配置文件路径
   */
  async getConfigPath(): Promise<string> {
    return await invokeTauri<string>('get_config_path_cmd', {
      file_name: CONFIG_FILE_NAME,
    });
  }
}

// 导出单例
export const configService = new ConfigService();
