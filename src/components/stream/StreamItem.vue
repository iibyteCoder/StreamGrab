<script setup lang="ts">
/**
 * StreamItem - 单个流项目组件
 * 纯展示组件：显示单个流的选择状态和信息
 */

import { computed } from "vue";
import { AppIcon } from "@/components/common";
import {
  formatBandwidth,
  getStreamName,
  getVideoDescription,
  getAudioDescription,
  getSubtitleDescription,
} from "@/composables/useStreamSelector";
import type { VideoStream, AudioStream, SubtitleStream } from "@/domain";

type StreamType = VideoStream | AudioStream | SubtitleStream;

interface Props {
  stream: StreamType;
  isSelected: boolean;
  showBandwidth?: boolean;
  type: "video" | "audio" | "subtitle";
}

const props = withDefaults(defineProps<Props>(), {
  showBandwidth: true,
});

const emit = defineEmits<{
  (e: "toggle"): void;
}>();

// 获取描述文本
const description = computed(() => {
  switch (props.type) {
    case "video":
      return getVideoDescription(props.stream as VideoStream);
    case "audio":
      return getAudioDescription(props.stream as AudioStream);
    case "subtitle":
      return getSubtitleDescription(props.stream as SubtitleStream);
    default:
      return "";
  }
});

// 带宽显示
const bandwidth = computed(() => {
  if (!props.showBandwidth || !props.stream.bandwidth) return null;
  return formatBandwidth(props.stream.bandwidth);
});

// 是否显示默认标签
const showDefaultLabel = computed(() => {
  const stream = props.stream as AudioStream | SubtitleStream;
  return stream.isDefault;
});

// 是否显示强制标签
const showForcedLabel = computed(() => {
  const stream = props.stream as SubtitleStream;
  return props.type === "subtitle" && stream.isForced;
});

// 选择框样式（视频用圆形，其他用方形）
const isCheckbox = computed(() => props.type !== "video");
</script>

<template>
  <button
    class="w-full text-left p-3 rounded-lg border transition-all hover:bg-accent"
    :class="isSelected ? 'border-primary bg-primary/10' : 'border-border'"
    @click="emit('toggle')"
  >
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-3">
        <!-- 选择指示器 -->
        <div
          class="w-5 h-5 flex items-center justify-center"
          :class="[
            isCheckbox ? 'rounded border' : 'rounded-full border-2',
            isSelected
              ? 'border-primary bg-primary'
              : 'border-muted-foreground',
          ]"
        >
          <AppIcon
            v-if="isSelected"
            name="Check"
            :size="12"
            class="text-primary-foreground"
          />
        </div>

        <!-- 流信息 -->
        <div>
          <div class="font-medium flex items-center gap-2">
            {{ getStreamName(stream) }}
            <span v-if="showDefaultLabel" class="text-xs text-primary"
              >默认</span
            >
            <span v-if="showForcedLabel" class="text-xs text-yellow-500"
              >强制</span
            >
          </div>
          <div class="text-sm text-muted-foreground">
            {{ description }}
          </div>
        </div>
      </div>

      <!-- 带宽 -->
      <div v-if="bandwidth" class="text-sm text-muted-foreground">
        {{ bandwidth }}
      </div>
    </div>
  </button>
</template>
