/**
 * 配置服务
 *
 * 管理应用配置的持久化，使用字段级别更新
 */

import { invokeTauri } from "./tauri";
import type {
  AppSettings,
  M3U8DLSettings,
  FFmpegSettings,
  NetworkSettings,
  DecryptionSettings,
  NetworkHeader,
  DecryptionKey,
  AllConfig,
} from "@/domain/config";
import {
  DEFAULT_APP_SETTINGS,
  DEFAULT_M3U8DL_SETTINGS,
  DEFAULT_FFMPEG_SETTINGS,
  DEFAULT_NETWORK_SETTINGS,
  DEFAULT_DECRYPTION_SETTINGS,
} from "@/domain/config";

// 配置表类型
export type ConfigTable =
  | "app"
  | "m3u8dl"
  | "ffmpeg"
  | "network"
  | "decryption";

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
  // ========================================
  // 新版 API：字段级别更新
  // ========================================

  /**
   * 获取应用配置
   */
  async getAppSettings(): Promise<AppSettings> {
    return await invokeTauri<AppSettings>("get_app_settings");
  }

  /**
   * 更新应用配置字段
   * @param field 字段名
   * @param value 字段值
   */
  async updateAppSettingField(
    field: keyof AppSettings,
    value: string | boolean,
  ): Promise<void> {
    await invokeTauri("update_app_setting_field", {
      field,
      value: String(value),
    });
  }

  /**
   * 获取 M3U8DL 配置
   */
  async getM3U8DLSettings(): Promise<M3U8DLSettings> {
    return await invokeTauri<M3U8DLSettings>("get_m3u8dl_settings");
  }

  /**
   * 更新 M3U8DL 配置字段
   * @param field 字段名
   * @param value 字段值
   */
  async updateM3U8DLSettingField(
    field: keyof M3U8DLSettings,
    value: string | number | boolean | null,
  ): Promise<void> {
    await invokeTauri("update_m3u8dl_setting_field", {
      field,
      value: value === null ? "" : String(value),
    });
  }

  /**
   * 获取 FFmpeg 配置
   */
  async getFFmpegSettings(): Promise<FFmpegSettings> {
    return await invokeTauri<FFmpegSettings>("get_ffmpeg_settings");
  }

  /**
   * 更新 FFmpeg 配置字段
   * @param field 字段名
   * @param value 字段值
   */
  async updateFFmpegSettingField(
    field: keyof FFmpegSettings,
    value: string | number | boolean | null,
  ): Promise<void> {
    await invokeTauri("update_ffmpeg_setting_field", {
      field,
      value: value === null ? "" : String(value),
    });
  }

  /**
   * 获取网络配置
   */
  async getNetworkSettings(): Promise<NetworkSettings> {
    return await invokeTauri<NetworkSettings>("get_network_settings");
  }

  /**
   * 更新网络配置字段
   * @param field 字段名
   * @param value 字段值
   */
  async updateNetworkSettingField(
    field: keyof NetworkSettings,
    value: string | boolean | null,
  ): Promise<void> {
    await invokeTauri("update_network_setting_field", {
      field,
      value: value === null ? "" : String(value),
    });
  }

  /**
   * 获取解密配置
   */
  async getDecryptionSettings(): Promise<DecryptionSettings> {
    return await invokeTauri<DecryptionSettings>("get_decryption_settings");
  }

  /**
   * 更新解密配置字段
   * @param field 字段名
   * @param value 字段值
   */
  async updateDecryptionSettingField(
    field: keyof DecryptionSettings,
    value: string | boolean | null,
  ): Promise<void> {
    await invokeTauri("update_decryption_setting_field", {
      field,
      value: value === null ? "" : String(value),
    });
  }

  // ========================================
  // 网络请求头管理
  // ========================================

  /**
   * 获取所有网络请求头
   */
  async getNetworkHeaders(): Promise<NetworkHeader[]> {
    return await invokeTauri<NetworkHeader[]>("get_network_headers");
  }

  /**
   * 添加网络请求头
   */
  async addNetworkHeader(name: string, value: string): Promise<number> {
    return await invokeTauri<number>("add_network_header", { name, value });
  }

  /**
   * 更新网络请求头
   */
  async updateNetworkHeader(
    id: number,
    name: string,
    value: string,
    enabled: boolean,
  ): Promise<void> {
    await invokeTauri("update_network_header", { id, name, value, enabled });
  }

  /**
   * 删除网络请求头
   */
  async deleteNetworkHeader(id: number): Promise<void> {
    await invokeTauri("delete_network_header", { id });
  }

  // ========================================
  // 解密密钥管理
  // ========================================

  /**
   * 获取所有解密密钥
   */
  async getDecryptionKeys(): Promise<DecryptionKey[]> {
    return await invokeTauri<DecryptionKey[]>("get_decryption_keys");
  }

  /**
   * 添加解密密钥
   */
  async addDecryptionKey(kid: string | null, key: string): Promise<number> {
    return await invokeTauri<number>("add_decryption_key", { kid, key });
  }

  /**
   * 删除解密密钥
   */
  async deleteDecryptionKey(id: number): Promise<void> {
    await invokeTauri("delete_decryption_key", { id });
  }

  // ========================================
  // 加载所有配置
  // ========================================

  /**
   * 加载所有配置
   */
  async loadAllConfig(): Promise<AllConfig> {
    const settingsMap =
      await invokeTauri<Record<string, unknown>>("load_settings");

    return {
      app: (settingsMap["app"] as AppSettings) || DEFAULT_APP_SETTINGS,
      m3u8dl:
        (settingsMap["m3u8dl"] as M3U8DLSettings) || DEFAULT_M3U8DL_SETTINGS,
      ffmpeg:
        (settingsMap["ffmpeg"] as FFmpegSettings) || DEFAULT_FFMPEG_SETTINGS,
      network:
        (settingsMap["network"] as NetworkSettings) || DEFAULT_NETWORK_SETTINGS,
      decryption:
        (settingsMap["decryption"] as DecryptionSettings) ||
        DEFAULT_DECRYPTION_SETTINGS,
      headers: (settingsMap["headers"] as NetworkHeader[]) || [],
      keys: (settingsMap["keys"] as DecryptionKey[]) || [],
    };
  }

  // ========================================
  // 导入/导出
  // ========================================

  /**
   * 导出配置到文件
   */
  async exportConfig(filePath: string): Promise<void> {
    await invokeTauri("export_config", { filePath });
  }

  /**
   * 从文件导入配置
   */
  async importConfig(filePath: string): Promise<AllConfig> {
    await invokeTauri("import_config", { filePath });
    return this.loadAllConfig();
  }

  // ========================================
  // 工具方法
  // ========================================

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

  // ========================================
  // 通用字段更新方法
  // ========================================

  /**
   * 通用更新配置字段方法
   * @param table 配置表名
   * @param field 字段名
   * @param value 字段值
   */
  async updateSettingField(
    table: ConfigTable,
    field: string,
    value: string | number | boolean | null,
  ): Promise<void> {
    const stringValue = value === null ? "" : String(value);

    switch (table) {
      case "app":
        await this.updateAppSettingField(
          field as keyof AppSettings,
          stringValue,
        );
        break;
      case "m3u8dl":
        await this.updateM3U8DLSettingField(
          field as keyof M3U8DLSettings,
          value,
        );
        break;
      case "ffmpeg":
        await this.updateFFmpegSettingField(
          field as keyof FFmpegSettings,
          value,
        );
        break;
      case "network":
        await this.updateNetworkSettingField(
          field as keyof NetworkSettings,
          stringValue,
        );
        break;
      case "decryption":
        await this.updateDecryptionSettingField(
          field as keyof DecryptionSettings,
          stringValue,
        );
        break;
      default:
        throw new Error(`Unknown config table: ${table}`);
    }
  }
}

export const configService = new ConfigService();
