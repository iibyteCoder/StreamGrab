/**
 * 配置服务
 * 管理应用配置的持久化（使用 SQLite）
 */

import { invokeTauri } from './tauri';
import type { AppSettings } from '@/types';
import { DEFAULT_SETTINGS } from '@/utils/constants';

/**
 * 配置服务类
 */
class ConfigService {
  private cachedSettings: AppSettings | null = null;
  private saveTimeout: ReturnType<typeof setTimeout> | null = null;

  /**
   * 加载配置
   * 从 SQLite 数据库加载配置
   */
  async loadSettings(): Promise<AppSettings> {
    try {
      const settingsMap = await invokeTauri<Record<string, Record<string, unknown>>>('load_settings');

      // 将 key-value 形式转换为 AppSettings 结构
      const settings = this.mapToAppSettings(settingsMap);

      // 合并默认值（处理新增配置项）
      this.cachedSettings = this.mergeWithDefaults(settings);
      return this.cachedSettings!;
    } catch (error) {
      console.warn('加载配置失败，使用默认配置:', error);
      this.cachedSettings = JSON.parse(JSON.stringify(DEFAULT_SETTINGS));
      return this.cachedSettings!;
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
      const settingsMap = this.appSettingsToMap(settings);
      await invokeTauri('save_settings', { settings: settingsMap });
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
    await invokeTauri('reset_all_settings');
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
   * 将 Map 转换为 AppSettings
   */
  private mapToAppSettings(map: Record<string, Record<string, unknown>>): Partial<AppSettings> {
    const result: Partial<AppSettings> = {};

    if (map['general']) result.general = map['general'] as unknown as AppSettings['general'];
    if (map['download']) result.download = map['download'] as unknown as AppSettings['download'];
    if (map['mux']) result.mux = map['mux'] as unknown as AppSettings['mux'];
    if (map['network']) result.network = map['network'] as unknown as AppSettings['network'];
    if (map['live']) result.live = map['live'] as unknown as AppSettings['live'];
    if (map['decryption']) result.decryption = map['decryption'] as unknown as AppSettings['decryption'];
    if (map['advanced']) result.advanced = map['advanced'] as unknown as AppSettings['advanced'];
    if (map['ui']) result.ui = map['ui'] as unknown as AppSettings['ui'];

    return result;
  }

  /**
   * 将 AppSettings 转换为 Map
   */
  private appSettingsToMap(settings: AppSettings): Record<string, Record<string, unknown>> {
    return {
      general: settings.general as unknown as Record<string, unknown>,
      download: settings.download as unknown as Record<string, unknown>,
      mux: settings.mux as unknown as Record<string, unknown>,
      network: settings.network as unknown as Record<string, unknown>,
      live: settings.live as unknown as Record<string, unknown>,
      decryption: settings.decryption as unknown as Record<string, unknown>,
      advanced: settings.advanced as unknown as Record<string, unknown>,
      ui: settings.ui as unknown as Record<string, unknown>,
    };
  }

  /**
   * 合并默认值
   * 确保所有配置项都存在（处理版本升级时新增的配置）
   */
  private mergeWithDefaults(settings: Partial<AppSettings>): AppSettings {
    const result = JSON.parse(JSON.stringify(DEFAULT_SETTINGS)) as AppSettings;

    // 递归合并
    this.deepMerge(result as unknown as Record<string, unknown>, settings as unknown as Record<string, unknown>);

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
    await invokeTauri('export_config', { filePath });
  }

  /**
   * 从文件导入配置
   * @param filePath 导入路径
   */
  async importConfig(filePath: string): Promise<AppSettings> {
    await invokeTauri('import_config', { filePath });

    // 重新加载配置
    return this.loadSettings();
  }

  /**
   * 获取数据库文件路径
   */
  async getDbPath(): Promise<string> {
    return await invokeTauri<string>('get_db_path');
  }

  /**
   * 在文件管理器中打开路径
   * @param path 文件或文件夹路径
   */
  async openInExplorer(path: string): Promise<void> {
    await invokeTauri('open_in_explorer', { path });
  }

  /**
   * 检查文件是否存在
   * @param path 文件路径
   */
  async fileExists(path: string): Promise<boolean> {
    return await invokeTauri<boolean>('file_exists', { path });
  }

  /**
   * 删除文件或文件夹
   * @param path 文件或文件夹路径
   */
  async deleteFileOrFolder(path: string): Promise<void> {
    await invokeTauri('delete_file_or_folder', { path });
  }

  /**
   * 选择目录
   * @returns 选中的目录路径，取消返回 null
   */
  async selectDirectory(): Promise<string | null> {
    return await invokeTauri<string | null>('select_directory');
  }

  /**
   * 选择文件
   * @param filters 文件过滤器
   * @returns 选中的文件路径，取消返回 null
   */
  async selectFile(filters?: Array<{ name: string; extensions: string[] }>): Promise<string | null> {
    return await invokeTauri<string | null>('select_file', { filters });
  }
}

// 导出单例
export const configService = new ConfigService();
