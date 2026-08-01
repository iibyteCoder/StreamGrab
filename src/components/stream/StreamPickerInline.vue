<script setup lang="ts">
/**
 * 流选择体（无 Dialog 外壳）。
 * 供 LinkAdvancedSection 内联嵌入；StreamSelector 也可包它做独立弹窗。
 * 业务逻辑在 useStreamSelector。
 */
import { toRef } from "vue";
import { Button } from "@/components/ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { ScrollArea } from "@/components/ui/scroll-area";
import { AppIcon } from "@/components/common";
import { StreamList } from "@/components/stream";
import { useStreamSelector } from "@/composables/useStreamSelector";
import type { StreamInfo, StreamSelection } from "@/domain";

const props = defineProps<{
  streamInfo: StreamInfo | null;
  loading?: boolean;
}>();

const emit = defineEmits<{
  (e: "confirm", selection: StreamSelection): void;
  (e: "cancel"): void;
}>();

const selector = useStreamSelector(toRef(props, "streamInfo"));

const handleConfirm = () => emit("confirm", selector.getSelection());
const handleCancel = () => emit("cancel");
</script>

<template>
  <div class="flex flex-col">
    <!-- 统计 -->
    <p v-if="selector.stats.value" class="mb-3 text-sm text-muted-foreground">
      共 {{ selector.stats.value.videoCount }} 个视频流、{{
        selector.stats.value.audioCount
      }}个音频流、{{ selector.stats.value.subtitleCount }}个字幕流
      <span v-if="selector.stats.value.duration !== '未知'"
        >· 时长 {{ selector.stats.value.duration }}</span
      >
      <span v-if="selector.stats.value.isLive" class="ml-1 text-red-400"
        >· 直播</span
      >
      <span v-if="selector.stats.value.isEncrypted" class="ml-1 text-yellow-400"
        >· 加密</span
      >
    </p>

    <!-- 加载 -->
    <div v-if="loading" class="flex items-center justify-center py-12">
      <div class="flex flex-col items-center gap-3">
        <AppIcon name="Loader2" :size="32" class="animate-spin text-primary" />
        <span class="text-muted-foreground">正在解析流信息...</span>
      </div>
    </div>

    <!-- 无数据 -->
    <div v-else-if="!streamInfo" class="flex items-center justify-center py-12">
      <div class="flex flex-col items-center gap-3">
        <AppIcon name="AlertCircle" :size="32" class="text-muted-foreground" />
        <span class="text-muted-foreground">无法获取流信息</span>
      </div>
    </div>

    <!-- 流列表 -->
    <template v-else>
      <Tabs v-model="selector.activeTab.value" class="flex min-h-0 flex-col">
        <TabsList class="grid w-full shrink-0 grid-cols-3">
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

        <ScrollArea class="mt-3 max-h-[40vh]">
          <TabsContent value="video" class="mt-0">
            <StreamList
              :streams="streamInfo.videos"
              :selected-ids="selector.selectedVideos.value"
              type="video"
              empty-text="没有可用的视频流"
              @toggle="selector.toggleVideo"
            />
          </TabsContent>
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

    <!-- 操作 -->
    <div class="mt-3 flex shrink-0 justify-end gap-2 border-t pt-3">
      <Button variant="outline" size="sm" @click="handleCancel">取消</Button>
      <Button
        size="sm"
        :disabled="!selector.canConfirm.value"
        @click="handleConfirm"
      >
        <AppIcon name="Check" :size="16" class="mr-1.5" />
        确认选择
      </Button>
    </div>
  </div>
</template>
