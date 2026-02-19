<script setup lang="ts">
/**
 * TemplateEditDialog - 模板编辑对话框
 * 纯展示组件：编辑或创建模板
 */

import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

interface Props {
  open: boolean;
  isEditing: boolean;
  name: string;
  description: string;
}

defineProps<Props>();

const emit = defineEmits<{
  (e: "update:open", value: boolean): void;
  (e: "update:name", value: string | number): void;
  (e: "update:description", value: string | number): void;
  (e: "save"): void;
  (e: "cancel"): void;
}>();

const handleCancel = () => {
  emit("cancel");
  emit("update:open", false);
};

const handleSave = () => {
  emit("save");
};
</script>

<template>
  <Dialog :open="open" @update:open="emit('update:open', $event)">
    <DialogContent class="sm:max-w-md">
      <DialogHeader>
        <DialogTitle>{{ isEditing ? "编辑模板" : "创建模板" }}</DialogTitle>
      </DialogHeader>

      <div class="space-y-4 py-4">
        <div class="space-y-2">
          <Label for="name">模板名称</Label>
          <Input
            id="name"
            :model-value="name"
            placeholder="例如：B站 1080P"
            @update:model-value="emit('update:name', $event)"
          />
        </div>
        <div class="space-y-2">
          <Label for="description">描述</Label>
          <Input
            id="description"
            :model-value="description"
            placeholder="可选，用于说明模板用途"
            @update:model-value="emit('update:description', $event)"
          />
        </div>

        <div v-if="!isEditing" class="text-sm text-muted-foreground">
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
        <Button variant="outline" @click="handleCancel">取消</Button>
        <Button @click="handleSave">{{ isEditing ? "保存" : "创建" }}</Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
