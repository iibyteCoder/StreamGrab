<script setup lang="ts">
/**
 * TemplateManager - 配置模板管理 UI 组件
 * 只负责 UI 展示，业务逻辑在 useTemplateManager 中
 *
 * 重构后：
 * - 使用 TemplateCard、TemplateEditDialog、TemplateDeleteDialog 子组件
 * - 主组件只负责布局和事件协调
 */

import { Button } from "@/components/ui/button";
import { AppIcon } from "@/components/common";
import { useTemplateManager } from "@/composables/useTemplateManager";
import {
  TemplateCard,
  TemplateEditDialog,
  TemplateDeleteDialog,
} from "@/components/template";

const manager = useTemplateManager();

// 保存并关闭
const handleSave = () => {
  if (manager.saveTemplate()) {
    manager.closeEditDialog();
  }
};
</script>

<template>
  <div class="space-y-6">
    <!-- 标题和操作按钮 -->
    <div class="flex items-center justify-between">
      <div>
        <h3 class="text-lg font-medium">配置模板</h3>
        <p class="text-sm text-muted-foreground">
          保存常用的下载配置，快速应用到新任务
        </p>
      </div>
      <Button @click="manager.createFromCurrentSettings()">
        <AppIcon name="Plus" :size="16" class="mr-1.5" />
        从当前设置创建
      </Button>
    </div>

    <!-- 预设模板 -->
    <div v-if="manager.presetTemplates.value.length > 0">
      <h4 class="text-sm font-medium text-muted-foreground mb-3">预设模板</h4>
      <div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
        <TemplateCard
          v-for="template in manager.presetTemplates.value"
          :key="template.id"
          :template="template"
          is-preset
          @apply="manager.applyTemplate(template)"
        />
      </div>
    </div>

    <!-- 用户模板 -->
    <div>
      <h4 class="text-sm font-medium text-muted-foreground mb-3">
        自定义模板
        <span v-if="manager.customTemplates.value.length > 0" class="ml-1"
          >({{ manager.customTemplates.value.length }})</span
        >
      </h4>

      <div
        v-if="manager.customTemplates.value.length === 0"
        class="text-center py-8 text-muted-foreground"
      >
        <AppIcon name="FileBox" :size="40" class="mx-auto mb-3 opacity-50" />
        <p>暂无自定义模板</p>
        <p class="text-sm mt-1">点击上方按钮从当前设置创建模板</p>
      </div>

      <div v-else class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
        <TemplateCard
          v-for="template in manager.customTemplates.value"
          :key="template.id"
          :template="template"
          @apply="manager.applyTemplate(template)"
          @edit="manager.editTemplate(template)"
          @duplicate="manager.duplicateTemplate(template)"
          @delete="manager.confirmDelete(template)"
        />
      </div>
    </div>

    <!-- 编辑模板对话框 -->
    <TemplateEditDialog
      v-model:open="manager.showEditDialog.value"
      :is-editing="!!manager.editingTemplate.value"
      :name="manager.editForm.value.name"
      :description="manager.editForm.value.description"
      @update:name="manager.editForm.value.name = String($event)"
      @update:description="manager.editForm.value.description = String($event)"
      @save="handleSave"
      @cancel="manager.closeEditDialog()"
    />

    <!-- 删除确认对话框 -->
    <TemplateDeleteDialog
      v-model:open="manager.showDeleteDialog.value"
      :template-name="manager.deletingTemplate.value?.name"
      @confirm="manager.deleteTemplate()"
      @cancel="manager.closeDeleteDialog()"
    />
  </div>
</template>
