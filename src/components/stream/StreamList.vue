<script setup lang="ts">
/**
 * StreamList - 流列表组件
 * 纯展示组件：显示流列表，支持多选
 */

import { computed } from "vue";
import { Button } from "@/components/ui/button";
import StreamItem from "./StreamItem.vue";
import type { VideoStream, AudioStream, SubtitleStream } from "@/domain";

type StreamType = VideoStream | AudioStream | SubtitleStream;

interface Props {
  streams: StreamType[];
  selectedIds: Set<string>;
  type: "video" | "audio" | "subtitle";
  showSelectAll?: boolean;
  showBandwidth?: boolean;
  emptyText?: string;
}

const props = withDefaults(defineProps<Props>(), {
  showSelectAll: false,
  showBandwidth: true,
  emptyText: "没有可用的流",
});

const emit = defineEmits<{
  (e: "toggle", id: string): void;
  (e: "toggleAll"): void;
}>();

const isEmpty = computed(() => props.streams.length === 0);

const isAllSelected = computed(
  () =>
    props.selectedIds.size === props.streams.length && props.streams.length > 0,
);

const selectAllText = computed(() =>
  isAllSelected.value ? "取消全选" : "全选",
);

const handleToggle = (id: string) => {
  emit("toggle", id);
};

const handleToggleAll = () => {
  emit("toggleAll");
};
</script>

<template>
  <div>
    <!-- 空状态 -->
    <div v-if="isEmpty" class="py-8 text-center text-muted-foreground">
      {{ emptyText }}
    </div>

    <!-- 流列表 -->
    <template v-else>
      <!-- 全选按钮 -->
      <div v-if="showSelectAll" class="flex justify-end mb-2">
        <Button
          variant="ghost"
          size="sm"
          class="text-xs h-7"
          @click="handleToggleAll"
        >
          {{ selectAllText }}
        </Button>
      </div>

      <!-- 流项目列表 -->
      <div class="space-y-2">
        <StreamItem
          v-for="stream in streams"
          :key="stream.id"
          :stream="stream"
          :is-selected="selectedIds.has(stream.id)"
          :type="type"
          :show-bandwidth="showBandwidth"
          @toggle="handleToggle(stream.id)"
        />
      </div>
    </template>
  </div>
</template>
