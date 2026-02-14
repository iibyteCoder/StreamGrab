<script setup lang="ts">
/**
 * TemplateManager - 配置模板管理 UI 组件
 * 只负责 UI 展示，业务逻辑在 useTemplateManager 中
 */

import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/ui/dialog';
import { AppIcon } from '@/components/common';
import { useTemplateManager } from '@/composables/useTemplateManager';

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
        <p class="text-sm text-muted-foreground">保存常用的下载配置，快速应用到新任务</p>
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
        <div
          v-for="template in manager.presetTemplates.value"
          :key="template.id"
          class="group border rounded-lg p-4 hover:border-primary/50 transition-colors"
        >
          <div class="flex items-start justify-between mb-2">
            <div class="flex items-center gap-2">
              <AppIcon name="Bookmark" :size="16" class="text-primary" />
              <span class="font-medium">{{ template.name }}</span>
            </div>
            <span class="text-xs text-muted-foreground">预设</span>
          </div>
          <p class="text-sm text-muted-foreground mb-3 line-clamp-2">{{ template.description }}</p>
          <Button variant="outline" size="sm" class="w-full" @click="manager.applyTemplate(template)">应用</Button>
        </div>
      </div>
    </div>

    <!-- 用户模板 -->
    <div>
      <h4 class="text-sm font-medium text-muted-foreground mb-3">
        自定义模板
        <span v-if="manager.customTemplates.value.length > 0" class="ml-1">({{ manager.customTemplates.value.length }})</span>
      </h4>

      <div v-if="manager.customTemplates.value.length === 0" class="text-center py-8 text-muted-foreground">
        <AppIcon name="FileBox" :size="40" class="mx-auto mb-3 opacity-50" />
        <p>暂无自定义模板</p>
        <p class="text-sm mt-1">点击上方按钮从当前设置创建模板</p>
      </div>

      <div v-else class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
        <div
          v-for="template in manager.customTemplates.value"
          :key="template.id"
          class="group relative border rounded-lg p-4 hover:border-primary/50 transition-colors"
        >
          <div class="flex items-start justify-between mb-2">
            <div class="flex items-center gap-2">
              <AppIcon name="FileText" :size="16" class="text-muted-foreground" />
              <span class="font-medium">{{ template.name }}</span>
            </div>
            <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
              <Button variant="ghost" size="icon" class="h-7 w-7" @click="manager.duplicateTemplate(template)">
                <AppIcon name="Copy" :size="14" />
              </Button>
              <Button variant="ghost" size="icon" class="h-7 w-7" @click="manager.editTemplate(template)">
                <AppIcon name="Pencil" :size="14" />
              </Button>
              <Button
                variant="ghost"
                size="icon"
                class="h-7 w-7 text-destructive hover:text-destructive"
                @click="manager.confirmDelete(template)"
              >
                <AppIcon name="Trash2" :size="14" />
              </Button>
            </div>
          </div>
          <p class="text-sm text-muted-foreground mb-3 line-clamp-2">{{ template.description || '无描述' }}</p>
          <div class="flex items-center justify-between">
            <span class="text-xs text-muted-foreground">
              {{ new Date(template.updatedAt).toLocaleDateString() }}
            </span>
            <Button variant="outline" size="sm" @click="manager.applyTemplate(template)">应用</Button>
          </div>
        </div>
      </div>
    </div>

    <!-- 编辑模板对话框 -->
    <Dialog v-model:open="manager.showEditDialog.value">
      <DialogContent class="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>{{ manager.editingTemplate.value ? '编辑模板' : '创建模板' }}</DialogTitle>
        </DialogHeader>

        <div class="space-y-4 py-4">
          <div class="space-y-2">
            <Label for="name">模板名称</Label>
            <Input id="name" v-model="manager.editForm.value.name" placeholder="例如：B站 1080P" />
          </div>
          <div class="space-y-2">
            <Label for="description">描述</Label>
            <Input id="description" v-model="manager.editForm.value.description" placeholder="可选，用于说明模板用途" />
          </div>

          <div v-if="!manager.editingTemplate.value" class="text-sm text-muted-foreground">
            <p>将保存当前的所有下载设置到此模板：</p>
            <ul class="list-disc list-inside mt-2 space-y-1 text-xs">
              <li>下载设置（线程数、重试次数等）</li>
              <li>流选择设置</li>
              <li>混流设置</li>
              <li>网络设置</li>
              <li>直播设置</li>
              <li>解密设置</li>
            </ul>
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" @click="manager.closeEditDialog()">取消</Button>
          <Button @click="handleSave">{{ manager.editingTemplate.value ? '保存' : '创建' }}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

    <!-- 删除确认对话框 -->
    <Dialog v-model:open="manager.showDeleteDialog.value">
      <DialogContent class="sm:max-w-sm">
        <DialogHeader>
          <DialogTitle>确认删除</DialogTitle>
        </DialogHeader>
        <p class="text-sm text-muted-foreground">
          确定要删除模板 "{{ manager.deletingTemplate.value?.name }}" 吗？此操作不可恢复。
        </p>
        <DialogFooter>
          <Button variant="outline" @click="manager.closeDeleteDialog()">取消</Button>
          <Button variant="destructive" @click="manager.deleteTemplate()">删除</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </div>
</template>
