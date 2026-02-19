<script setup lang="ts">
/**
 * AdKeywordManager - 广告过滤关键字管理器
 * 纯展示组件：管理广告过滤关键字列表
 */

import { computed } from "vue";
import { Button } from "@/components/ui/button";
import { AppIcon } from "@/components/common";

interface Props {
  keywords: string[];
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: "update:keywords", value: string[]): void;
}>();

const isEmpty = computed(() => props.keywords.length === 0);

const addKeyword = () => {
  emit("update:keywords", [...props.keywords, ""]);
};

const removeKeyword = (index: number) => {
  const newKeywords = props.keywords.filter((_, i) => i !== index);
  emit("update:keywords", newKeywords);
};

const updateKeyword = (index: number, value: string) => {
  const newKeywords = [...props.keywords];
  newKeywords[index] = value;
  emit("update:keywords", newKeywords);
};
</script>

<template>
  <div class="flex items-center justify-between">
    <span class="text-sm font-medium">过滤关键字（正则表达式）</span>
    <Button variant="outline" size="sm" @click="addKeyword">
      <AppIcon name="Plus" :size="14" class="mr-1" />
      添加
    </Button>
  </div>

  <div v-if="isEmpty" class="text-sm text-muted-foreground py-2">
    暂无过滤关键字
  </div>

  <div v-else class="space-y-2">
    <div
      v-for="(_, index) in keywords"
      :key="index"
      class="flex items-center gap-2"
    >
      <input
        :value="keywords[index]"
        type="text"
        placeholder="例如: ad\.domain\.com"
        class="flex-1 h-9 px-3 text-sm rounded-md border border-input bg-transparent focus:outline-none focus:ring-2 focus:ring-ring"
        @input="updateKeyword(index, ($event.target as HTMLInputElement).value)"
      />
      <Button
        variant="ghost"
        size="icon"
        class="h-9 w-9 text-destructive hover:text-destructive"
        @click="removeKeyword(index)"
      >
        <AppIcon name="Trash2" :size="16" />
      </Button>
    </div>
  </div>
</template>
