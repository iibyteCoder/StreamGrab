<script setup lang="ts">
/**
 * 单条链接聚焦配置（L2）。
 * 按 detectedType 动态渲染：通用三件（文件名/保存位置/定时）始终可见；
 * 流媒体专属项（限速/范围/容器/字幕/流选择/解密）经 isOptionVisible 控制。
 * 流选择内联嵌入 StreamPickerInline，不再弹窗套弹窗。
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
import { useDownloader } from "@/composables";
import { isStreamingType } from "@/domain/url";
import { isOptionVisible } from "./linkOptionVisibility";
import type { StagedLink } from "./staging-types";
import type { MuxFormat, StreamSelection, SubtitleFormat } from "@/domain";

const props = defineProps<{
  saveDirPlaceholder: string;
}>();

const emit = defineEmits<{ (e: "done"): void }>();

/** 绑定整条 StagedLink（深层字段就地修改，响应式回传父） */
const link = defineModel<StagedLink>({ required: true });

const { parseUrl, isParsing } = useDownloader();

const isStreaming = computed(
  () =>
    link.value.detectedType !== null &&
    isStreamingType(link.value.detectedType),
);

/** 流选择折叠 */
const showStreamPicker = ref(false);

/** 单链接流媒体：进入即自动解析一次（沿用原单链接体验） */
const autoParsed = ref(false);
async function ensureParsed() {
  if (!isStreaming.value || autoParsed.value) return;
  autoParsed.value = true;
  await handleParse();
}

async function handleParse() {
  if (!isStreaming.value) return;
  const info = await parseUrl(link.value.url);
  if (info) {
    link.value.streamInfo = info;
    link.value.status = "parsed";
    showStreamPicker.value = true;
  }
}

function handleStreamConfirm(sel: StreamSelection) {
  link.value.overrides.selection = sel;
  showStreamPicker.value = false;
}

function handleStreamCancel() {
  showStreamPicker.value = false;
}

/** 最小调度时间 */
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

const typeBadgeLabel = computed(() => {
  const t = link.value.detectedType;
  if (!t) return "";
  const labels: Record<string, string> = {
    hls: "HLS",
    dash: "DASH",
    mss: "MSS",
    httpVideo: "直链视频",
    unknown: "未知",
  };
  return labels[t] ?? "";
});

// 单链接流媒体场景由父（AddTaskDialog）在挂载后调用 ensureParsed
defineExpose({ ensureParsed });
</script>

<template>
  <div class="space-y-4">
    <!-- 行头：文件名 + 类型徽章 -->
    <div class="space-y-1.5">
      <Label class="text-xs text-muted-foreground">文件名</Label>
      <Input
        v-model="link.fileName"
        placeholder="自动从 URL 提取"
        class="h-9 text-sm"
      />
      <div class="flex items-center gap-2 text-xs">
        <span
          v-if="typeBadgeLabel"
          class="rounded-full bg-primary/20 px-2 py-0.5 font-medium text-primary"
          >{{ typeBadgeLabel }}</span
        >
      </div>
    </div>

    <!-- 保存位置（通用） -->
    <div class="space-y-1.5">
      <Label class="text-xs text-muted-foreground">保存位置</Label>
      <Input
        v-model="link.saveDir"
        :placeholder="props.saveDirPlaceholder"
        class="h-9 text-sm"
      />
    </div>

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
        <!-- 解析流 -->
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
              @click="handleParse"
            >
              <AppIcon
                v-if="isParsing"
                name="Loader2"
                :size="14"
                class="mr-1.5 animate-spin"
              />
              <AppIcon v-else name="Search" :size="14" class="mr-1.5" />
              {{ link.streamInfo ? "重新解析" : "解析流" }}
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
          <p
            v-if="link.overrides.selection"
            class="text-xs text-muted-foreground/70"
          >
            已选：视频 {{ link.overrides.selection.video ?? "自动" }} · 音频
            {{ link.overrides.selection.audio ?? "自动" }} · 字幕
            {{ link.overrides.selection.subtitle ?? "自动" }}
          </p>
          <!-- 内联流选择体 -->
          <div
            v-if="showStreamPicker && link.streamInfo"
            class="rounded-lg border bg-muted/30 p-3"
          >
            <StreamPickerInline
              :stream-info="link.streamInfo"
              :loading="isParsing"
              @confirm="handleStreamConfirm"
              @cancel="handleStreamCancel"
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
            :model-value="link.overrides.muxFormat ?? ''"
            @update:model-value="
              (v) =>
                (link.overrides.muxFormat = (v ? String(v) : undefined) as
                  | MuxFormat
                  | undefined)
            "
          >
            <SelectTrigger class="h-9 text-sm">
              <SelectValue placeholder="跟随全局" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="">跟随全局</SelectItem>
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
            :model-value="link.overrides.subtitleFormat ?? ''"
            @update:model-value="
              (v) =>
                (link.overrides.subtitleFormat = (v ? String(v) : undefined) as
                  | SubtitleFormat
                  | undefined)
            "
          >
            <SelectTrigger class="h-9 text-sm">
              <SelectValue placeholder="跟随全局" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="">跟随全局</SelectItem>
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

    <!-- 完成 -->
    <div class="flex justify-end border-t pt-3">
      <Button size="sm" @click="emit('done')">
        <AppIcon name="Check" :size="16" class="mr-1.5" />
        完成
      </Button>
    </div>
  </div>
</template>

<style scoped>
.datetime-dark {
  color-scheme: dark;
}
</style>
