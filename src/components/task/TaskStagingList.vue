<script setup lang="ts">
/**
 * L1 总览：粘贴框 + 批次公共默认 + 紧凑行清单。
 * 只渲染 + 通知；不构造 StagedLink（paste 事件交父）、不碰 overrides、不碰合并。
 */
import { ref } from "vue";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { AppIcon } from "@/components/common";
import { usePresetManager } from "@/composables";
import type { BatchDefaults, StagedLink } from "./staging-types";

const props = defineProps<{
  links: StagedLink[];
  batch: BatchDefaults;
  batchPresetId: string;
  globalSaveDir: string;
}>();

const emit = defineEmits<{
  (e: "update:batch", b: BatchDefaults): void;
  (e: "update:preset", presetId: string): void;
  (e: "paste", text: string): void;
  (e: "select", id: string): void;
  (e: "remove", id: string): void;
  (e: "commit"): void;
}>();

const { presets } = usePresetManager();

const pasteText = ref("");
const isDragging = ref(false);

function onPaste() {
  emit("paste", pasteText.value);
}

function onDragOver(e: DragEvent) {
  e.preventDefault();
  isDragging.value = true;
}
function onDragLeave() {
  isDragging.value = false;
}
function onDrop(e: DragEvent) {
  e.preventDefault();
  isDragging.value = false;
  const text = e.dataTransfer?.getData("text/plain");
  if (text) emit("paste", text);
}

function patchBatch(patch: Partial<BatchDefaults>) {
  emit("update:batch", { ...props.batch, ...patch });
}

const statusColor: Record<string, string> = {
  pending: "text-muted-foreground",
  parsed: "text-primary",
  ready: "text-primary",
  invalid: "text-red-500",
};
const statusLabel: Record<string, string> = {
  pending: "待配置",
  parsed: "已解析",
  ready: "就绪",
  invalid: "无效",
};

const saveDirPlaceholder = props.globalSaveDir || "使用全局默认";
</script>

<template>
  <div class="space-y-4">
    <!-- 粘贴框 -->
    <div
      class="relative"
      @dragover="onDragOver"
      @dragleave="onDragLeave"
      @drop="onDrop"
    >
      <div
        v-if="isDragging"
        class="absolute inset-0 z-10 flex items-center justify-center rounded-lg border-2 border-dashed border-primary bg-primary/10"
      >
        <span class="text-sm font-medium text-primary">释放以添加链接</span>
      </div>
      <textarea
        v-model="pasteText"
        placeholder="粘贴下载链接，每行一个（支持 M3U8 / DASH / MP4 等）"
        class="h-20 w-full resize-none rounded-lg border bg-muted/50 px-3 py-2 text-sm transition-colors focus:border-primary focus:outline-none focus:ring-2 focus:ring-primary/50"
        @blur="onPaste"
      />
    </div>

    <!-- 批次公共默认 -->
    <div
      class="grid grid-cols-1 gap-3 rounded-lg border bg-muted/20 p-3 sm:grid-cols-3"
    >
      <div class="space-y-1.5">
        <Label class="text-xs text-muted-foreground"
          >保存位置（本批默认）</Label
        >
        <div class="flex gap-2">
          <Input
            :model-value="props.batch.saveDir"
            :placeholder="saveDirPlaceholder"
            class="h-9 flex-1 text-sm"
            @update:model-value="
              (v: string | number) => patchBatch({ saveDir: String(v) })
            "
          />
          <!-- 浏览按钮由父（编排者）填充，守「子组件不直接调 service」 -->
          <slot name="saveDirBrowse" />
        </div>
      </div>
      <div class="space-y-1.5">
        <Label class="text-xs text-muted-foreground">预设（初值）</Label>
        <Select
          :model-value="props.batchPresetId"
          @update:model-value="(v) => emit('update:preset', String(v))"
        >
          <SelectTrigger class="h-9 text-sm">
            <SelectValue placeholder="不使用预设" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="__none__">不使用预设</SelectItem>
            <SelectItem v-for="p in presets" :key="p.id" :value="p.id">
              {{ p.name }}
            </SelectItem>
          </SelectContent>
        </Select>
      </div>
      <div class="flex items-end justify-between">
        <Label class="text-xs text-muted-foreground">自动开始</Label>
        <Switch
          :checked="props.batch.autoStart"
          @update:checked="(v: boolean) => patchBatch({ autoStart: v })"
        />
      </div>
    </div>

    <!-- 行清单 -->
    <div v-if="props.links.length > 1" class="space-y-2">
      <div
        v-for="row in props.links"
        :key="row.id"
        class="flex cursor-pointer items-center gap-3 rounded-lg border px-3 py-2 transition-colors hover:border-primary"
        @click="emit('select', row.id)"
      >
        <AppIcon
          name="FileVideo"
          :size="16"
          class="shrink-0 text-muted-foreground"
        />
        <div class="min-w-0 flex-1">
          <div class="truncate text-sm font-medium">
            {{ row.fileName || row.url }}
          </div>
          <div class="truncate text-xs text-muted-foreground">
            {{ row.url }}
          </div>
        </div>
        <span
          v-if="row.detectedType"
          class="rounded-full bg-primary/20 px-2 py-0.5 text-xs font-medium text-primary"
          >{{ row.detectedType.toUpperCase() }}</span
        >
        <span :class="['text-xs', statusColor[row.status]]">{{
          statusLabel[row.status]
        }}</span>
        <Button
          variant="ghost"
          size="sm"
          class="h-7 px-2"
          @click.stop="emit('remove', row.id)"
        >
          <AppIcon name="X" :size="14" />
        </Button>
      </div>
    </div>

    <!-- 提交 -->
    <div class="flex justify-end border-t pt-3">
      <Button :disabled="props.links.length === 0" @click="emit('commit')">
        <AppIcon name="Download" :size="16" class="mr-2" />
        全部添加
      </Button>
    </div>
  </div>
</template>
