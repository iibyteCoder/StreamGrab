/**
 * 任务预设服务
 *
 * 预设 = 命名的 TaskOverrides 组合，DB 持久化（取代旧的 localStorage 模板）
 */

import { invokeTauri } from "./tauri";
import type { TaskPreset } from "@/domain";

class PresetService {
  loadPresets(): Promise<TaskPreset[]> {
    return invokeTauri<TaskPreset[]>("load_presets");
  }

  /** 保存预设（按 ID upsert） */
  savePreset(preset: TaskPreset): Promise<void> {
    return invokeTauri("save_preset", { preset });
  }

  deletePreset(id: string): Promise<void> {
    return invokeTauri("delete_preset", { id });
  }
}

export const presetService = new PresetService();
