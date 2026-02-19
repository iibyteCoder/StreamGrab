<script setup lang="ts">
/**
 * TemplateCard - 模板卡片组件
 * 纯展示组件：显示单个模板信息
 */

import { computed } from "vue";
import { Button } from "@/components/ui/button";
import { AppIcon } from "@/components/common";
import type { ConfigTemplate } from "@/types";

interface Props {
  template: ConfigTemplate;
  isPreset?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  isPreset: false,
});

const emit = defineEmits<{
  (e: "apply"): void;
  (e: "edit"): void;
  (e: "duplicate"): void;
  (e: "delete"): void;
}>();

const formattedDate = computed(() =>
  new Date(props.template.updatedAt).toLocaleDateString(),
);

const icon = computed(() => (props.isPreset ? "Bookmark" : "FileText"));
const iconClass = computed(() =>
  props.isPreset ? "text-primary" : "text-muted-foreground",
);
</script>

<template>
  <div
    class="group relative border rounded-lg p-4 hover:border-primary/50 transition-colors"
  >
    <div class="flex items-start justify-between mb-2">
      <div class="flex items-center gap-2">
        <AppIcon :name="icon as any" :size="16" :class="iconClass" />
        <span class="font-medium">{{ template.name }}</span>
      </div>
      <div class="flex items-center gap-1">
        <!-- 预设标签 -->
        <span v-if="isPreset" class="text-xs text-muted-foreground">预设</span>
        <!-- 用户模板操作按钮 -->
        <template v-else>
          <Button
            variant="ghost"
            size="icon"
            class="h-7 w-7 opacity-0 group-hover:opacity-100 transition-opacity"
            @click="emit('duplicate')"
          >
            <AppIcon name="Copy" :size="14" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            class="h-7 w-7 opacity-0 group-hover:opacity-100 transition-opacity"
            @click="emit('edit')"
          >
            <AppIcon name="Pencil" :size="14" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            class="h-7 w-7 text-destructive hover:text-destructive opacity-0 group-hover:opacity-100 transition-opacity"
            @click="emit('delete')"
          >
            <AppIcon name="Trash2" :size="14" />
          </Button>
        </template>
      </div>
    </div>

    <p class="text-sm text-muted-foreground mb-3 line-clamp-2">
      {{ template.description || (isPreset ? "" : "无描述") }}
    </p>

    <div class="flex items-center justify-between">
      <span v-if="!isPreset" class="text-xs text-muted-foreground">
        {{ formattedDate }}
      </span>
      <span v-else></span>
      <Button variant="outline" size="sm" @click="emit('apply')">应用</Button>
    </div>
  </div>
</template>
