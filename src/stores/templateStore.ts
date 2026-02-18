/**
 * 配置模板状态管理
 * 用于保存和管理用户自定义的下载配置模板
 */

import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type { ConfigTemplate, AppSettings } from "@/types";

const STORAGE_KEY = "streamgrab-config-templates";

/**
 * 生成唯一 ID
 */
function generateId(): string {
  return `tpl-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`;
}

/**
 * 从 localStorage 加载模板
 */
function loadTemplatesFromStorage(): ConfigTemplate[] {
  try {
    const data = localStorage.getItem(STORAGE_KEY);
    if (data) {
      const templates = JSON.parse(data);
      // 转换日期
      return templates.map((t: ConfigTemplate) => ({
        ...t,
        createdAt: new Date(t.createdAt),
        updatedAt: new Date(t.updatedAt),
      }));
    }
  } catch (error) {
    console.error("Failed to load templates from storage:", error);
  }
  return [];
}

/**
 * 保存模板到 localStorage
 */
function saveTemplatesToStorage(templates: ConfigTemplate[]): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(templates));
  } catch (error) {
    console.error("Failed to save templates to storage:", error);
  }
}

/**
 * 默认模板
 */
const DEFAULT_TEMPLATES: ConfigTemplate[] = [
  {
    id: "default-best",
    name: "最佳质量",
    description: "自动选择最高质量的视频和音频流",
    settings: {
      download: {
        autoSelect: true,
        selectVideo: "best",
        selectAudio: "best",
      },
    } as Partial<AppSettings>,
    createdAt: new Date(),
    updatedAt: new Date(),
  },
  {
    id: "default-1080p",
    name: "1080P",
    description: "选择 1080P 分辨率的视频流",
    settings: {
      download: {
        autoSelect: false,
        selectVideo: 'res="1920*"',
        selectAudio: "best",
      },
    } as Partial<AppSettings>,
    createdAt: new Date(),
    updatedAt: new Date(),
  },
  {
    id: "default-720p",
    name: "720P",
    description: "选择 720P 分辨率的视频流，适合带宽有限的场景",
    settings: {
      download: {
        autoSelect: false,
        selectVideo: 'res="1280*"',
        selectAudio: "best",
      },
    } as Partial<AppSettings>,
    createdAt: new Date(),
    updatedAt: new Date(),
  },
];

export const useTemplateStore = defineStore("template", () => {
  // State
  const templates = ref<ConfigTemplate[]>([]);
  const isInitialized = ref(false);

  // Getters
  const templateCount = computed(() => templates.value.length);

  const customTemplates = computed(() =>
    templates.value.filter((t) => !t.id.startsWith("default-")),
  );

  const presetTemplates = computed(() =>
    templates.value.filter((t) => t.id.startsWith("default-")),
  );

  // Actions

  /**
   * 初始化 Store
   */
  function initialize(): void {
    if (isInitialized.value) return;

    // 加载用户模板
    const savedTemplates = loadTemplatesFromStorage();

    // 合并默认模板和用户模板
    const defaultIds = new Set(DEFAULT_TEMPLATES.map((t) => t.id));
    const userTemplates = savedTemplates.filter((t) => !defaultIds.has(t.id));

    templates.value = [...DEFAULT_TEMPLATES, ...userTemplates];
    isInitialized.value = true;
  }

  /**
   * 获取模板
   */
  function getTemplate(id: string): ConfigTemplate | undefined {
    return templates.value.find((t) => t.id === id);
  }

  /**
   * 添加模板
   */
  function addTemplate(
    name: string,
    description: string,
    settings: Partial<AppSettings>,
  ): ConfigTemplate {
    const now = new Date();
    const template: ConfigTemplate = {
      id: generateId(),
      name,
      description,
      settings,
      createdAt: now,
      updatedAt: now,
    };

    templates.value.push(template);
    saveToStorage();

    return template;
  }

  /**
   * 更新模板
   */
  function updateTemplate(
    id: string,
    updates: Partial<Pick<ConfigTemplate, "name" | "description" | "settings">>,
  ): boolean {
    const index = templates.value.findIndex((t) => t.id === id);
    if (index === -1) return false;

    // 不允许修改默认模板
    if (id.startsWith("default-")) return false;

    const template = templates.value[index]!;
    templates.value[index] = {
      id: template.id,
      name: updates.name ?? template.name,
      description: updates.description ?? template.description,
      settings: updates.settings ?? template.settings,
      createdAt: template.createdAt,
      updatedAt: new Date(),
    };

    saveToStorage();
    return true;
  }

  /**
   * 删除模板
   */
  function deleteTemplate(id: string): boolean {
    const index = templates.value.findIndex((t) => t.id === id);
    if (index === -1) return false;

    // 不允许删除默认模板
    if (id.startsWith("default-")) return false;

    templates.value.splice(index, 1);
    saveToStorage();

    return true;
  }

  /**
   * 复制模板
   */
  function duplicateTemplate(id: string): ConfigTemplate | null {
    const template = getTemplate(id);
    if (!template) return null;

    return addTemplate(
      `${template.name} (副本)`,
      template.description,
      template.settings,
    );
  }

  /**
   * 应用模板到设置
   */
  function applyTemplate(
    templateId: string,
    currentSettings: AppSettings,
  ): AppSettings {
    const template = getTemplate(templateId);
    if (!template) return currentSettings;

    // 深度合并设置
    const newSettings = JSON.parse(JSON.stringify(currentSettings));

    for (const key of Object.keys(template.settings) as Array<
      keyof AppSettings
    >) {
      if (template.settings[key]) {
        newSettings[key] = {
          ...newSettings[key],
          ...template.settings[key],
        };
      }
    }

    return newSettings;
  }

  /**
   * 保存到存储
   */
  function saveToStorage(): void {
    // 只保存用户模板
    const userTemplates = templates.value.filter(
      (t) => !t.id.startsWith("default-"),
    );
    saveTemplatesToStorage(userTemplates);
  }

  return {
    // State
    templates,
    isInitialized,

    // Getters
    templateCount,
    customTemplates,
    presetTemplates,

    // Actions
    initialize,
    getTemplate,
    addTemplate,
    updateTemplate,
    deleteTemplate,
    duplicateTemplate,
    applyTemplate,
  };
});
