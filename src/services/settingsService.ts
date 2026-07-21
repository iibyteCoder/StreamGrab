/**
 * 设置服务
 *
 * 与后端 settings 命令组对应：应用设置（app_settings）+ 按工具分离的配置（tool_settings）。
 * 「部分更新」方法走后端递归合并，Store 只发增量。
 */

import { invokeTauri } from "./tauri";
import type { AppSettings, FfmpegConfig, Nm3u8dlConfig } from "@/domain";

/** 深部分类型（嵌套配置的增量更新） */
export type DeepPartial<T> = {
  [P in keyof T]?: T[P] extends Array<infer U>
    ? Array<U>
    : T[P] extends object
      ? DeepPartial<T[P]>
      : T[P];
};

class SettingsService {
  // ===== 应用设置 =====

  getAppSettings(): Promise<AppSettings> {
    return invokeTauri<AppSettings>("get_app_settings");
  }

  saveAppSettings(settings: AppSettings): Promise<void> {
    return invokeTauri("save_app_settings", { settings });
  }

  /** 部分更新应用设置，返回合并后的完整配置 */
  patchAppSettings(partial: DeepPartial<AppSettings>): Promise<AppSettings> {
    return invokeTauri<AppSettings>("patch_app_settings", { partial });
  }

  // ===== N_m3u8DL-RE 配置 =====

  getNm3u8dlConfig(): Promise<Nm3u8dlConfig> {
    return invokeTauri<Nm3u8dlConfig>("get_tool_settings", {
      toolId: "nm3u8dl",
    });
  }

  saveNm3u8dlConfig(config: Nm3u8dlConfig): Promise<void> {
    return invokeTauri("save_tool_settings", {
      toolId: "nm3u8dl",
      config,
    });
  }

  /** 部分更新，返回合并后的完整配置 */
  patchNm3u8dlConfig(
    partial: DeepPartial<Nm3u8dlConfig>,
  ): Promise<Nm3u8dlConfig> {
    return invokeTauri<Nm3u8dlConfig>("patch_tool_settings", {
      toolId: "nm3u8dl",
      partial,
    });
  }

  // ===== FFmpeg 配置 =====

  getFfmpegConfig(): Promise<FfmpegConfig> {
    return invokeTauri<FfmpegConfig>("get_tool_settings", {
      toolId: "ffmpeg",
    });
  }

  saveFfmpegConfig(config: FfmpegConfig): Promise<void> {
    return invokeTauri("save_tool_settings", { toolId: "ffmpeg", config });
  }

  /** 部分更新，返回合并后的完整配置 */
  patchFfmpegConfig(partial: DeepPartial<FfmpegConfig>): Promise<FfmpegConfig> {
    return invokeTauri<FfmpegConfig>("patch_tool_settings", {
      toolId: "ffmpeg",
      partial,
    });
  }

  // ===== 导入导出 =====

  /** 导出全部设置（应用 + 全部工具配置） */
  exportConfig(): Promise<Record<string, unknown>> {
    return invokeTauri<Record<string, unknown>>("export_config");
  }

  /** 从 JSON 文件导入设置（部分导入：只合并存在的字段） */
  importConfig(filePath: string): Promise<void> {
    return invokeTauri("import_config", { filePath });
  }
}

export const settingsService = new SettingsService();
