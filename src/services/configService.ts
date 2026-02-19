/**
 * 配置服务
 * 管理应用配置的持久化（使用 SQLite）
 */

import { invokeTauri } from "./tauri";
import type { AppSettings } from "@/types";
import { DEFAULT_SETTINGS } from "@/utils/constants";

// 配置模块的 key 类型
export type SettingsKey = keyof AppSettings;

/**
 * 文件信息接口
 */
export interface FileInfo {
  /** 文件完整路径 */
  path: string;
  /** 文件名 */
  fileName: string;
  /** 文件扩展名 */
  extension: string;
  /** 文件大小（字节） */
  size: number;
  /** 修改时间（Unix 毫秒时间戳） */
  modified: number | null;
  /** 文件是否存在 */
  exists: boolean;
}

/**
 * 配置服务类
 */
class ConfigService {
  /**
   * 加载所有配置
   */
  async loadSettings(): Promise<AppSettings> {
    const settingsMap =
      await invokeTauri<Record<string, Record<string, unknown>>>(
        "load_settings",
      );

    const settings = this.mapToAppSettings(settingsMap);
    return this.mergeWithDefaults(settings);
  }

  /**
   * 保存单个配置模块
   * @param key 配置模块名称
   * @param value 配置值
   */
  async saveSetting<K extends SettingsKey>(
    key: K,
    value: AppSettings[K],
  ): Promise<void> {
    await invokeTauri("save_setting", { key, value });
  }

  /**
   * 保存所有配置（用于导入配置后）
   */
  async saveAllSettings(settings: AppSettings): Promise<void> {
    const settingsMap = this.appSettingsToMap(settings);
    await invokeTauri("save_settings", { settings: settingsMap });
  }

  /**
   * 重置单个配置模块
   */
  async resetSetting(key: SettingsKey): Promise<void> {
    await invokeTauri("reset_setting", { key });
  }

  /**
   * 重置所有配置为默认值
   */
  async resetAllSettings(): Promise<void> {
    await invokeTauri("reset_all_settings");
  }

  /**
   * 导出配置到文件
   */
  async exportConfig(filePath: string): Promise<void> {
    await invokeTauri("export_config", { filePath });
  }

  /**
   * 从文件导入配置
   */
  async importConfig(filePath: string): Promise<AppSettings> {
    await invokeTauri("import_config", { filePath });
    return this.loadSettings();
  }

  // ========== 工具方法 ==========

  async getDbPath(): Promise<string> {
    return await invokeTauri<string>("get_db_path");
  }

  async openInExplorer(path: string): Promise<void> {
    await invokeTauri("open_in_explorer", { path });
  }

  async fileExists(path: string): Promise<boolean> {
    return await invokeTauri<boolean>("file_exists", { path });
  }

  async deleteFileOrFolder(path: string): Promise<void> {
    await invokeTauri("delete_file_or_folder", { path });
  }

  async selectDirectory(): Promise<string | null> {
    return await invokeTauri<string | null>("select_directory");
  }

  async selectFile(
    filters?: Array<{ name: string; extensions: string[] }>,
  ): Promise<string | null> {
    return await invokeTauri<string | null>("select_file", { filters });
  }

  /**
   * 获取文件详细信息
   * @param path 文件路径
   */
  async getFileInfo(path: string): Promise<FileInfo> {
    const info = await invokeTauri<{
      path: string;
      file_name: string;
      extension: string;
      size: number;
      modified: number | null;
      exists: boolean;
    }>("get_file_info", { path });

    return {
      path: info.path,
      fileName: info.file_name,
      extension: info.extension,
      size: info.size,
      modified: info.modified,
      exists: info.exists,
    };
  }

  // ========== 私有方法 ==========

  private mapToAppSettings(
    map: Record<string, Record<string, unknown>>,
  ): Partial<AppSettings> {
    return {
      general: map["general"] as unknown as AppSettings["general"],
      download: map["download"] as unknown as AppSettings["download"],
      mux: map["mux"] as unknown as AppSettings["mux"],
      network: map["network"] as unknown as AppSettings["network"],
      live: map["live"] as unknown as AppSettings["live"],
      decryption: map["decryption"] as unknown as AppSettings["decryption"],
      advanced: map["advanced"] as unknown as AppSettings["advanced"],
      ui: map["ui"] as unknown as AppSettings["ui"],
    };
  }

  private appSettingsToMap(
    settings: AppSettings,
  ): Record<string, Record<string, unknown>> {
    const keys: SettingsKey[] = [
      "general",
      "download",
      "mux",
      "network",
      "live",
      "decryption",
      "advanced",
      "ui",
    ];
    const result: Record<string, Record<string, unknown>> = {};
    for (const key of keys) {
      result[key] = settings[key] as unknown as Record<string, unknown>;
    }
    return result;
  }

  private mergeWithDefaults(settings: Partial<AppSettings>): AppSettings {
    const result = JSON.parse(JSON.stringify(DEFAULT_SETTINGS)) as AppSettings;
    this.deepMerge(
      result as unknown as Record<string, unknown>,
      settings as unknown as Record<string, unknown>,
    );
    return result;
  }

  private deepMerge(
    target: Record<string, unknown>,
    source: Record<string, unknown> | undefined,
  ): void {
    if (!source) return;
    for (const key of Object.keys(source)) {
      if (
        key in target &&
        typeof target[key] === "object" &&
        target[key] !== null &&
        !Array.isArray(target[key]) &&
        typeof source[key] === "object" &&
        source[key] !== null &&
        !Array.isArray(source[key])
      ) {
        this.deepMerge(
          target[key] as Record<string, unknown>,
          source[key] as Record<string, unknown>,
        );
      } else if (source[key] !== undefined) {
        target[key] = source[key];
      }
    }
  }
}

export const configService = new ConfigService();
