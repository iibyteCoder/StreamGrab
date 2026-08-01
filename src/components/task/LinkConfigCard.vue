<script setup lang="ts">
/**
 * 单条链接配置卡（L1 字段 + 高级手风琴）。
 * 纯编辑；目录浏览 / 解析经 emit 上抛向导（子组件不直接调 service）。
 */
import { computed, ref } from "vue";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { AppIcon } from "@/components/common";
import LinkAdvancedSection from "./LinkAdvancedSection.vue";
import { typeBadgeLabel } from "./addTaskTypes";
import type { StagedLink } from "./addTaskTypes";

const props = defineProps<{
  recentDirs: string[];
  defaultDir: string;
  parsing: boolean;
}>();
const emit = defineEmits<{
  (e: "parse"): void;
  (e: "browseSaveDir"): void;
}>();
const link = defineModel<StagedLink>({ required: true });

const showAdvanced = ref(false);
const badge = computed(() => typeBadgeLabel(link.value.detectedType));
const saveDirPlaceholder = computed(() => props.defaultDir || "使用全局默认");
</script>

<template>
  <div class="space-y-4">
    <!-- 链接 -->
    <div class="space-y-1.5">
      <Label class="text-xs text-muted-foreground">链接</Label>
      <Input v-model="link.url" class="h-9 text-sm" />
      <div class="flex items-center gap-2 text-xs">
        <span
          class="rounded-full bg-primary/20 px-2 py-0.5 font-medium text-primary"
        >
          {{ badge }}
        </span>
        <span v-if="link.parseFailed" class="text-red-400">解析失败</span>
        <span v-else-if="link.streamInfo" class="text-muted-foreground"
          >已解析</span
        >
      </div>
    </div>

    <!-- 保存位置 + 记忆下拉 -->
    <div class="space-y-1.5">
      <Label class="text-xs text-muted-foreground">保存位置</Label>
      <div class="flex gap-2">
        <Input
          v-model="link.saveDir"
          :placeholder="saveDirPlaceholder"
          class="h-9 flex-1 text-sm"
        />
        <Button
          variant="outline"
          size="sm"
          class="h-9 px-3"
          @click="emit('browseSaveDir')"
        >
          <AppIcon name="FolderOpen" :size="14" />
        </Button>
      </div>
      <DropdownMenu v-if="recentDirs.length">
        <DropdownMenuTrigger as-child>
          <button
            class="flex cursor-pointer items-center gap-1 text-xs text-muted-foreground transition-colors hover:text-foreground"
          >
            <AppIcon name="History" :size="12" />
            最近：{{ recentDirs.slice(0, 3).join(" · ") }}
          </button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="start" class="max-w-[320px]">
          <DropdownMenuItem
            v-for="d in recentDirs"
            :key="d"
            @click="link.saveDir = d"
          >
            <span class="truncate">{{ d }}</span>
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>

    <!-- 文件名 -->
    <div class="space-y-1.5">
      <Label class="text-xs text-muted-foreground">文件名</Label>
      <Input
        v-model="link.fileName"
        placeholder="自动从 URL 提取"
        class="h-9 text-sm"
      />
    </div>

    <!-- 高级设置（手风琴） -->
    <div class="border-t border-border/60 pt-3">
      <button
        class="flex cursor-pointer items-center gap-1.5 text-sm text-muted-foreground transition-colors hover:text-foreground"
        @click="showAdvanced = !showAdvanced"
      >
        <AppIcon
          :name="showAdvanced ? 'ChevronDown' : 'ChevronRight'"
          :size="14"
        />
        高级设置
      </button>
      <div v-if="showAdvanced" class="mt-3">
        <LinkAdvancedSection
          v-model="link"
          :parsing="parsing"
          @parse="emit('parse')"
        />
      </div>
    </div>
  </div>
</template>
