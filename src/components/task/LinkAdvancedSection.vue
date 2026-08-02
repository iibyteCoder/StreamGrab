<script setup lang="ts">
/**
 * 高级设置区（L2/L3）：按引擎类型动态渲染。
 * 纯编辑 + 上抛 parse；解析由向导执行（本组件不触达 service）。
 */
import { computed, ref } from "vue";
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
import { StreamPickerInline } from "@/components/stream";
import { isStreamingType } from "@/domain/url";
import { isOptionVisible } from "./linkOptionVisibility";
import type { StagedLink } from "./addTaskTypes";
import type { MuxFormat, StreamSelection, SubtitleFormat } from "@/domain";

defineProps<{ parsing: boolean }>();
const emit = defineEmits<{ (e: "parse"): void }>();
const link = defineModel<StagedLink>({ required: true });

const isStreaming = computed(() => isStreamingType(link.value.detectedType));
const showStreamPicker = ref(false);

/** reka-ui 的 SelectItem 禁用空字符串值，用哨兵值表示「跟随全局」 */
const FOLLOW_GLOBAL = "__follow_global__";

const minScheduleTime = computed(() => {
  const now = new Date();
  const pad = (n: number) => n.toString().padStart(2, "0");
  return `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}T${pad(now.getHours())}:${pad(now.getMinutes())}`;
});
const scheduleTime = computed<string>({
  get: () => link.value.overrides.scheduledStartAt ?? "",
  set: (v: string) => {
    link.value.overrides.scheduledStartAt = v || undefined;
  },
});
function handleStreamConfirm(sel: StreamSelection) {
  link.value.overrides.selection = sel;
  showStreamPicker.value = false;
}
</script>

<template>
  <div class="space-y-4">
    <!-- 定时开始（通用） -->
    <div class="space-y-1.5">
      <div class="flex items-center justify-between">
        <Label class="cursor-pointer text-xs text-muted-foreground"
          >定时开始</Label
        >
        <Switch
          :checked="!!scheduleTime"
          @update:checked="
            (v: boolean) => (scheduleTime = v ? minScheduleTime : '')
          "
        />
      </div>
      <Input
        v-if="scheduleTime"
        v-model="scheduleTime"
        type="datetime-local"
        :min="minScheduleTime"
        class="datetime-dark h-9 text-sm"
      />
    </div>

    <!-- 流媒体专属（按 isOptionVisible 动态） -->
    <template v-if="isStreaming">
      <div class="space-y-3 border-t border-border/60 pt-3">
        <!-- 流选择 / 解析 / 重试 -->
        <div
          v-if="isOptionVisible('streamSelection', link.detectedType)"
          class="space-y-1.5"
        >
          <Label class="text-xs text-muted-foreground">流选择</Label>
          <div class="flex gap-2">
            <Button
              variant="outline"
              size="sm"
              class="h-9"
              :disabled="parsing"
              @click="emit('parse')"
            >
              <AppIcon
                v-if="parsing"
                name="Loader2"
                :size="14"
                class="mr-1.5 animate-spin"
              />
              <AppIcon v-else name="Search" :size="14" class="mr-1.5" />
              {{
                link.streamInfo
                  ? "重新解析"
                  : link.parseFailed
                    ? "重试解析"
                    : "解析流"
              }}
            </Button>
            <Button
              v-if="link.streamInfo"
              variant="ghost"
              size="sm"
              class="h-9"
              @click="showStreamPicker = !showStreamPicker"
            >
              <AppIcon name="ListVideo" :size="14" class="mr-1.5" />
              {{ showStreamPicker ? "收起" : "选择流" }}
            </Button>
          </div>
          <p v-if="link.parseFailed && !parsing" class="text-xs text-red-400">
            解析失败，可重试或直接添加（下载时按默认处理）
          </p>
          <p
            v-else-if="link.overrides.selection"
            class="text-xs text-muted-foreground/70"
          >
            已选：视频 {{ link.overrides.selection.video ?? "自动" }} · 音频
            {{ link.overrides.selection.audio ?? "自动" }} · 字幕
            {{ link.overrides.selection.subtitle ?? "自动" }}
          </p>
          <div
            v-if="showStreamPicker && link.streamInfo"
            class="rounded-lg border bg-muted/30 p-3"
          >
            <StreamPickerInline
              :stream-info="link.streamInfo"
              :loading="parsing"
              @confirm="handleStreamConfirm"
              @cancel="showStreamPicker = false"
            />
          </div>
        </div>

        <!-- 限速 -->
        <div
          v-if="isOptionVisible('maxSpeed', link.detectedType)"
          class="space-y-1.5"
        >
          <Label class="text-xs text-muted-foreground">限速</Label>
          <Input
            :model-value="link.overrides.maxSpeed ?? ''"
            @update:model-value="
              (v: string | number) =>
                (link.overrides.maxSpeed = v ? String(v) : undefined)
            "
            placeholder="如 10M，留空跟随全局"
            class="h-9 text-sm"
          />
        </div>

        <!-- 下载范围 -->
        <div
          v-if="isOptionVisible('customRange', link.detectedType)"
          class="space-y-1.5"
        >
          <Label class="text-xs text-muted-foreground">下载范围</Label>
          <Input
            :model-value="link.overrides.customRange ?? ''"
            @update:model-value="
              (v: string | number) =>
                (link.overrides.customRange = v ? String(v) : undefined)
            "
            placeholder="如 00:00:00-00:10:00"
            class="h-9 text-sm"
          />
        </div>

        <!-- 容器格式 -->
        <div
          v-if="isOptionVisible('muxFormat', link.detectedType)"
          class="space-y-1.5"
        >
          <Label class="text-xs text-muted-foreground">容器格式</Label>
          <Select
            :model-value="link.overrides.muxFormat ?? FOLLOW_GLOBAL"
            @update:model-value="
              (v) =>
                (link.overrides.muxFormat =
                  v === FOLLOW_GLOBAL ? undefined : (String(v) as MuxFormat))
            "
          >
            <SelectTrigger class="h-9 text-sm">
              <SelectValue placeholder="跟随全局" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem :value="FOLLOW_GLOBAL">跟随全局</SelectItem>
              <SelectItem value="mp4">MP4</SelectItem>
              <SelectItem value="mkv">MKV</SelectItem>
            </SelectContent>
          </Select>
        </div>

        <!-- 字幕格式 -->
        <div
          v-if="isOptionVisible('subtitleFormat', link.detectedType)"
          class="space-y-1.5"
        >
          <Label class="text-xs text-muted-foreground">字幕格式</Label>
          <Select
            :model-value="link.overrides.subtitleFormat ?? FOLLOW_GLOBAL"
            @update:model-value="
              (v) =>
                (link.overrides.subtitleFormat =
                  v === FOLLOW_GLOBAL
                    ? undefined
                    : (String(v) as SubtitleFormat))
            "
          >
            <SelectTrigger class="h-9 text-sm">
              <SelectValue placeholder="跟随全局" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem :value="FOLLOW_GLOBAL">跟随全局</SelectItem>
              <SelectItem value="SRT">SRT</SelectItem>
              <SelectItem value="VTT">VTT</SelectItem>
            </SelectContent>
          </Select>
        </div>

        <!-- 仅下载字幕 -->
        <div
          v-if="isOptionVisible('subtitlesOnly', link.detectedType)"
          class="flex items-center justify-between"
        >
          <Label class="cursor-pointer text-xs text-muted-foreground"
            >仅下载字幕</Label
          >
          <Switch
            :checked="!!link.overrides.subtitlesOnly"
            @update:checked="(v: boolean) => (link.overrides.subtitlesOnly = v)"
          />
        </div>

        <!-- 任务级解密密钥 -->
        <div
          v-if="isOptionVisible('key', link.detectedType)"
          class="space-y-1.5"
        >
          <Label class="text-xs text-muted-foreground">解密密钥</Label>
          <Input
            :model-value="link.overrides.key ?? ''"
            @update:model-value="
              (v: string | number) =>
                (link.overrides.key = v ? String(v) : undefined)
            "
            placeholder="全局密钥库为空时生效"
            class="h-9 text-sm"
          />
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped>
.datetime-dark {
  color-scheme: dark;
}
</style>
