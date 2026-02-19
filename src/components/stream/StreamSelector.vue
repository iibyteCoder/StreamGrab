<script setup lang="ts">
/**
 * StreamSelector - 流选择器 UI 组件
 * 只负责 UI 展示，业务逻辑在 useStreamSelector 中
 *
 * 重构后：
 * - 使用 StreamList 通用组件减少重复代码
 * - 主组件只负责布局和事件协调
 */

import { toRef } from "vue";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { ScrollArea } from "@/components/ui/scroll-area";
import { AppIcon } from "@/components/common";
import { StreamList } from "@/components/stream";
import { useStreamSelector } from "@/composables/useStreamSelector";
import type { StreamInfo, StreamSelection } from "@/types";

// Props
const props = defineProps<{
  open: boolean;
  streamInfo: StreamInfo | null;
  loading?: boolean;
}>();

// Emits
const emit = defineEmits<{
  (e: "update:open", value: boolean): void;
  (e: "confirm", selection: StreamSelection): void;
  (e: "cancel"): void;
}>();

// 使用 composable
const selector = useStreamSelector(toRef(props, "streamInfo"));

// 确认选择
const handleConfirm = () => {
  emit("confirm", selector.getSelection());
  emit("update:open", false);
};

// 取消
const handleCancel = () => {
  emit("cancel");
  emit("update:open", false);
};
</script>

<template>
  <Dialog :open="open" @update:open="emit('update:open', $event)">
    <DialogContent class="max-w-2xl max-h-[85vh] flex flex-col">
      <DialogHeader>
        <DialogTitle class="flex items-center gap-2">
          <AppIcon name="ListVideo" :size="20" />
          选择流
        </DialogTitle>
        <DialogDescription v-if="selector.stats.value">
          共 {{ selector.stats.value.videoCount }} 个视频流、{{
            selector.stats.value.audioCount
          }}
          个音频流、{{ selector.stats.value.subtitleCount }} 个字幕流
          <span v-if="selector.stats.value.duration !== '未知'">
            · 时长 {{ selector.stats.value.duration }}</span
          >
          <span v-if="selector.stats.value.isLive" class="text-red-400 ml-1"
            >· 直播</span
          >
          <span
            v-if="selector.stats.value.isEncrypted"
            class="text-yellow-400 ml-1"
            >· 加密</span
          >
        </DialogDescription>
      </DialogHeader>

      <!-- 加载状态 -->
      <div v-if="loading" class="flex-1 flex items-center justify-center py-12">
        <div class="flex flex-col items-center gap-3">
          <AppIcon
            name="Loader2"
            :size="32"
            class="animate-spin text-primary"
          />
          <span class="text-muted-foreground">正在解析流信息...</span>
        </div>
      </div>

      <!-- 无数据 -->
      <div
        v-else-if="!streamInfo"
        class="flex-1 flex items-center justify-center py-12"
      >
        <div class="flex flex-col items-center gap-3">
          <AppIcon
            name="AlertCircle"
            :size="32"
            class="text-muted-foreground"
          />
          <span class="text-muted-foreground">无法获取流信息</span>
        </div>
      </div>

      <!-- 流列表 -->
      <template v-else>
        <Tabs
          v-model="selector.activeTab.value"
          class="flex-1 min-h-0 flex flex-col"
        >
          <TabsList class="grid w-full grid-cols-3 shrink-0">
            <TabsTrigger value="video">
              <AppIcon name="Video" :size="16" class="mr-1.5" />
              视频 ({{ streamInfo.videos.length }})
            </TabsTrigger>
            <TabsTrigger value="audio">
              <AppIcon name="Music" :size="16" class="mr-1.5" />
              音频 ({{ streamInfo.audios.length }})
            </TabsTrigger>
            <TabsTrigger value="subtitle">
              <AppIcon name="Subtitles" :size="16" class="mr-1.5" />
              字幕 ({{ streamInfo.subtitles.length }})
            </TabsTrigger>
          </TabsList>

          <ScrollArea class="flex-1 mt-3 -mx-6 px-6">
            <!-- 视频流 -->
            <TabsContent value="video" class="mt-0">
              <StreamList
                :streams="streamInfo.videos"
                :selected-ids="selector.selectedVideos.value"
                type="video"
                empty-text="没有可用的视频流"
                @toggle="selector.toggleVideo"
              />
            </TabsContent>

            <!-- 音频流 -->
            <TabsContent value="audio" class="mt-0">
              <StreamList
                :streams="streamInfo.audios"
                :selected-ids="selector.selectedAudios.value"
                type="audio"
                show-select-all
                empty-text="没有可用的音频流"
                @toggle="selector.toggleAudio"
                @toggle-all="selector.toggleAllAudio"
              />
            </TabsContent>

            <!-- 字幕流 -->
            <TabsContent value="subtitle" class="mt-0">
              <StreamList
                :streams="streamInfo.subtitles"
                :selected-ids="selector.selectedSubtitles.value"
                type="subtitle"
                show-select-all
                empty-text="没有可用的字幕流"
                @toggle="selector.toggleSubtitle"
                @toggle-all="selector.toggleAllSubtitle"
              />
            </TabsContent>
          </ScrollArea>
        </Tabs>
      </template>

      <DialogFooter class="shrink-0">
        <Button variant="outline" @click="handleCancel">取消</Button>
        <Button :disabled="!selector.canConfirm.value" @click="handleConfirm">
          <AppIcon name="Download" :size="16" class="mr-1.5" />
          确认下载
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
