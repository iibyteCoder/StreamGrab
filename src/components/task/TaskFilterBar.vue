<script setup lang="ts">
/**
 * TaskFilterBar - 任务过滤栏组件
 * 提供搜索和排序功能
 */

import { computed } from "vue";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { AppIcon } from "@/components/common";
import type { SortOrder } from "@/composables/useTaskFilter";

interface Props {
  search: string;
  sort: SortOrder;
  placeholder?: string;
}

const props = withDefaults(defineProps<Props>(), {
  placeholder: "搜索任务...",
});

const emit = defineEmits<{
  (e: "update:search", value: string): void;
  (e: "update:sort", value: SortOrder): void;
}>();

const sortOptions: { value: SortOrder; label: string }[] = [
  { value: "newest", label: "最新优先" },
  { value: "oldest", label: "最早优先" },
  { value: "status", label: "按状态" },
];

const localSearch = computed({
  get: () => props.search,
  set: (value) => emit("update:search", value),
});

const localSort = computed({
  get: () => props.sort,
  set: (value) => emit("update:sort", value),
});
</script>

<template>
  <div class="flex items-center gap-2">
    <!-- 搜索框 -->
    <div class="relative flex-1">
      <AppIcon
        name="Search"
        :size="16"
        class="absolute left-2.5 top-1/2 -translate-y-1/2 text-muted-foreground pointer-events-none"
      />
      <Input
        v-model="localSearch"
        :placeholder="placeholder"
        class="pl-8 h-8 text-sm"
      />
    </div>

    <!-- 排序选择 -->
    <Select v-model="localSort">
      <SelectTrigger class="w-[100px] h-8 text-sm">
        <SelectValue placeholder="排序" />
      </SelectTrigger>
      <SelectContent>
        <SelectItem
          v-for="option in sortOptions"
          :key="option.value"
          :value="option.value"
        >
          {{ option.label }}
        </SelectItem>
      </SelectContent>
    </Select>
  </div>
</template>
