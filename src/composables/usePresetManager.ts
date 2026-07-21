/**
 * 预设管理组合式函数
 *
 * presetStore 的 UI 侧封装：应用预设（返回 overrides 供对话框填充）、
 * 新建/编辑/删除预设。取代旧的 useTemplateManager。
 */

import { ref, computed, onMounted } from "vue";
import { usePresetStore } from "@/stores";
import { useToast } from "./useToast";
import type { TaskPreset, TaskOverrides } from "@/domain";
import { i18n } from "@/locales";

export function usePresetManager() {
  const store = usePresetStore();
  const toast = useToast();

  const showEditDialog = ref(false);
  const showDeleteDialog = ref(false);
  const editingPreset = ref<TaskPreset | null>(null);
  const deletingPreset = ref<TaskPreset | null>(null);

  const editForm = ref({
    name: "",
    description: "",
    icon: "" as string | null,
    overrides: {} as TaskOverrides,
  });

  const presets = computed(() => store.presets);
  const count = computed(() => store.count);

  /** 加载预设 */
  const loadPresets = (): Promise<void> => store.loadPresets();

  onMounted(() => {
    if (!store.loaded) {
      loadPresets();
    }
  });

  /** 应用预设 — 返回 overrides 供对话框填充 */
  const applyPreset = (presetId: string): TaskOverrides | null => {
    const preset = store.getPreset(presetId);
    if (!preset) {
      toast.error(i18n.global.t("settings.preset.notFound"));
      return null;
    }
    return { ...preset.overrides, presetId: preset.id };
  };

  /** 打开新建对话框 */
  const createFromOverrides = (overrides: TaskOverrides): void => {
    editingPreset.value = null;
    editForm.value = {
      name: "",
      description: "",
      icon: null,
      overrides: { ...overrides },
    };
    showEditDialog.value = true;
  };

  /** 打开编辑对话框 */
  const editPreset = (preset: TaskPreset): void => {
    editingPreset.value = preset;
    editForm.value = {
      name: preset.name,
      description: preset.description || "",
      icon: preset.icon || null,
      overrides: { ...preset.overrides },
    };
    showEditDialog.value = true;
  };

  /** 保存（新建或更新） */
  const savePreset = async (): Promise<boolean> => {
    if (!editForm.value.name.trim()) {
      toast.warning(i18n.global.t("settings.preset.nameRequired"));
      return false;
    }

    try {
      await store.savePreset({
        id: editingPreset.value?.id,
        name: editForm.value.name,
        icon: editForm.value.icon,
        description: editForm.value.description || null,
        overrides: editForm.value.overrides,
      });

      toast.success(
        editingPreset.value
          ? i18n.global.t("settings.preset.updated")
          : i18n.global.t("settings.preset.created"),
      );
      showEditDialog.value = false;
      return true;
    } catch (e) {
      toast.error(
        i18n.global.t("settings.preset.saveFailed", {
          error:
            e instanceof Error
              ? e.message
              : i18n.global.t("settings.preset.unknownError"),
        }),
      );
      return false;
    }
  };

  /** 确认删除 */
  const confirmDelete = (preset: TaskPreset): void => {
    deletingPreset.value = preset;
    showDeleteDialog.value = true;
  };

  /** 执行删除 */
  const deletePreset = async (): Promise<void> => {
    if (!deletingPreset.value) return;

    try {
      await store.deletePreset(deletingPreset.value.id);
      toast.success(i18n.global.t("settings.preset.deleted"));
    } catch {
      toast.error(i18n.global.t("settings.preset.deleteFailed"));
    }

    showDeleteDialog.value = false;
    deletingPreset.value = null;
  };

  /** 复制预设 */
  const duplicatePreset = async (preset: TaskPreset): Promise<void> => {
    try {
      await store.savePreset({
        name: `${preset.name} (${i18n.global.t("settings.preset.copySuffix")})`,
        description: preset.description,
        icon: preset.icon,
        overrides: { ...preset.overrides },
      });
      toast.success(i18n.global.t("settings.preset.copied"));
    } catch {
      toast.error(i18n.global.t("settings.preset.copyFailed"));
    }
  };

  const closeEditDialog = (): void => {
    showEditDialog.value = false;
  };

  const closeDeleteDialog = (): void => {
    showDeleteDialog.value = false;
    deletingPreset.value = null;
  };

  return {
    // State
    showEditDialog,
    showDeleteDialog,
    editingPreset,
    deletingPreset,
    editForm,

    // Computed
    presets,
    count,

    // Actions
    loadPresets,
    applyPreset,
    createFromOverrides,
    editPreset,
    savePreset,
    confirmDelete,
    deletePreset,
    duplicatePreset,
    closeEditDialog,
    closeDeleteDialog,
  };
}
