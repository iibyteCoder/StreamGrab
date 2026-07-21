/**
 * 任务预设状态管理
 *
 * 预设 = 命名的 TaskOverrides 组合，DB 持久化（取代旧 localStorage 模板）
 */

import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type { TaskPreset, TaskOverrides } from "@/domain";
import { generateId } from "@/utils/id";
import { presetService } from "@/services";

export const usePresetStore = defineStore("preset", () => {
  // ========================================
  // State
  // ========================================

  const presets = ref<TaskPreset[]>([]);
  const loaded = ref(false);

  // ========================================
  // Computed
  // ========================================

  const count = computed(() => presets.value.length);

  // ========================================
  // Actions
  // ========================================

  /** 从后端加载全部预设 */
  async function loadPresets(): Promise<void> {
    try {
      presets.value = await presetService.loadPresets();
      loaded.value = true;
    } catch (e) {
      console.error("Failed to load presets:", e);
    }
  }

  /** 保存预设（按 ID upsert；新建时自动生成 id 和时间戳） */
  async function savePreset(params: {
    id?: string;
    name: string;
    icon?: string | null;
    description?: string | null;
    overrides: TaskOverrides;
  }): Promise<TaskPreset> {
    const now = new Date().toISOString();
    const isUpdate =
      !!params.id && presets.value.some((p) => p.id === params.id);

    const existing = params.id
      ? presets.value.find((p) => p.id === params.id)
      : undefined;

    const preset: TaskPreset = {
      id: params.id || generateId("preset-"),
      name: params.name,
      icon: params.icon ?? null,
      description: params.description ?? null,
      overrides: params.overrides,
      createdAt: existing?.createdAt ?? now,
      updatedAt: now,
    };

    await presetService.savePreset(preset);

    if (isUpdate && existing) {
      const idx = presets.value.findIndex((p) => p.id === preset.id);
      if (idx !== -1) presets.value[idx] = preset;
    } else {
      presets.value.push(preset);
    }

    return preset;
  }

  /** 删除预设 */
  async function deletePreset(id: string): Promise<void> {
    await presetService.deletePreset(id);
    presets.value = presets.value.filter((p) => p.id !== id);
  }

  /** 按 ID 查找预设 */
  function getPreset(id: string): TaskPreset | undefined {
    return presets.value.find((p) => p.id === id);
  }

  return {
    // State
    presets,
    loaded,

    // Computed
    count,

    // Actions
    loadPresets,
    savePreset,
    deletePreset,
    getPreset,
  };
});
