/**
 * 模板管理组合式函数
 * 负责配置模板的业务逻辑
 */

import { ref, computed, onMounted } from "vue";
import { useTemplateStore, useSettingsStore } from "@/stores";
import { useToast } from "./useToast";
import type { ConfigTemplate, AppSettings } from "@/types";

/**
 * 模板管理组合式函数
 */
export function useTemplateManager() {
  const templateStore = useTemplateStore();
  const settingsStore = useSettingsStore();
  const toast = useToast();

  // 对话框状态
  const showEditDialog = ref(false);
  const showDeleteDialog = ref(false);
  const editingTemplate = ref<ConfigTemplate | null>(null);
  const deletingTemplate = ref<ConfigTemplate | null>(null);

  // 表单数据
  const editForm = ref({
    name: "",
    description: "",
  });

  // 初始化
  onMounted(() => {
    templateStore.initialize();
  });

  // 预设模板
  const presetTemplates = computed(() => templateStore.presetTemplates);

  // 用户模板
  const customTemplates = computed(() => templateStore.customTemplates);

  // 从当前设置创建新模板
  const createFromCurrentSettings = () => {
    editingTemplate.value = null;
    editForm.value = { name: "", description: "" };
    showEditDialog.value = true;
  };

  // 编辑模板
  const editTemplate = (template: ConfigTemplate) => {
    if (template.id.startsWith("default-")) {
      toast.warning("无法编辑预设模板");
      return;
    }
    editingTemplate.value = template;
    editForm.value = {
      name: template.name,
      description: template.description,
    };
    showEditDialog.value = true;
  };

  // 提取当前设置
  const extractCurrentSettings = (): Partial<AppSettings> => {
    const settings = settingsStore.settings;
    return {
      download: { ...settings.download },
      mux: { ...settings.mux },
      network: { ...settings.network },
      live: { ...settings.live },
      decryption: { ...settings.decryption },
    };
  };

  // 保存模板
  const saveTemplate = () => {
    if (!editForm.value.name.trim()) {
      toast.warning("请输入模板名称");
      return false;
    }

    if (editingTemplate.value) {
      const success = templateStore.updateTemplate(editingTemplate.value.id, {
        name: editForm.value.name,
        description: editForm.value.description,
      });
      if (success) {
        toast.success("模板已更新");
      } else {
        toast.error("更新失败");
      }
      return success;
    } else {
      templateStore.addTemplate(
        editForm.value.name,
        editForm.value.description,
        extractCurrentSettings(),
      );
      toast.success("模板已创建");
      return true;
    }
  };

  // 确认删除
  const confirmDelete = (template: ConfigTemplate) => {
    if (template.id.startsWith("default-")) {
      toast.warning("无法删除预设模板");
      return;
    }
    deletingTemplate.value = template;
    showDeleteDialog.value = true;
  };

  // 删除模板
  const deleteTemplate = () => {
    if (deletingTemplate.value) {
      const success = templateStore.deleteTemplate(deletingTemplate.value.id);
      if (success) {
        toast.success("模板已删除");
      } else {
        toast.error("删除失败");
      }
    }
    showDeleteDialog.value = false;
    deletingTemplate.value = null;
  };

  // 复制模板
  const duplicateTemplate = (template: ConfigTemplate) => {
    const newTemplate = templateStore.duplicateTemplate(template.id);
    if (newTemplate) {
      toast.success("模板已复制");
    } else {
      toast.error("复制失败");
    }
  };

  // 应用模板
  const applyTemplate = (template: ConfigTemplate) => {
    const newSettings = templateStore.applyTemplate(
      template.id,
      settingsStore.settings,
    );
    settingsStore.setSettings(newSettings);
    toast.success(`已应用模板: ${template.name}`);
  };

  // 关闭编辑对话框
  const closeEditDialog = () => {
    showEditDialog.value = false;
  };

  // 关闭删除对话框
  const closeDeleteDialog = () => {
    showDeleteDialog.value = false;
    deletingTemplate.value = null;
  };

  return {
    // 状态
    showEditDialog,
    showDeleteDialog,
    editingTemplate,
    deletingTemplate,
    editForm,

    // 计算属性
    presetTemplates,
    customTemplates,

    // 方法
    createFromCurrentSettings,
    editTemplate,
    saveTemplate,
    confirmDelete,
    deleteTemplate,
    duplicateTemplate,
    applyTemplate,
    closeEditDialog,
    closeDeleteDialog,
  };
}
