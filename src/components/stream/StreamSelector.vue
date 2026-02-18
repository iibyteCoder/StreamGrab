<script setup lang="ts">
/**
 * StreamSelector - 流选择器 UI 组件
 * 只负责 UI 展示，业务逻辑在 useStreamSelector 中
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
import {
  useStreamSelector,
  formatBandwidth,
  getStreamName,
  getVideoDescription,
  getAudioDescription,
  getSubtitleDescription,
} from "@/composables/useStreamSelector";
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
              <div
                v-if="streamInfo.videos.length === 0"
                class="py-8 text-center text-muted-foreground"
              >
                没有可用的视频流
              </div>
              <div v-else class="space-y-2">
                <button
                  v-for="video in streamInfo.videos"
                  :key="video.id"
                  class="w-full text-left p-3 rounded-lg border transition-all hover:bg-accent"
                  :class="
                    selector.selectedVideos.value.has(video.id)
                      ? 'border-primary bg-primary/10'
                      : 'border-border'
                  "
                  @click="selector.toggleVideo(video.id)"
                >
                  <div class="flex items-center justify-between">
                    <div class="flex items-center gap-3">
                      <div
                        class="w-5 h-5 rounded-full border-2 flex items-center justify-center"
                        :class="
                          selector.selectedVideos.value.has(video.id)
                            ? 'border-primary bg-primary'
                            : 'border-muted-foreground'
                        "
                      >
                        <AppIcon
                          v-if="selector.selectedVideos.value.has(video.id)"
                          name="Check"
                          :size="12"
                          class="text-primary-foreground"
                        />
                      </div>
                      <div>
                        <div class="font-medium">
                          {{ getStreamName(video) }}
                        </div>
                        <div class="text-sm text-muted-foreground">
                          {{ getVideoDescription(video) }}
                        </div>
                      </div>
                    </div>
                    <div class="text-sm text-muted-foreground">
                      {{ formatBandwidth(video.bandwidth) }}
                    </div>
                  </div>
                </button>
              </div>
            </TabsContent>

            <!-- 音频流 -->
            <TabsContent value="audio" class="mt-0">
              <div
                v-if="streamInfo.audios.length === 0"
                class="py-8 text-center text-muted-foreground"
              >
                没有可用的音频流
              </div>
              <template v-else>
                <div class="flex justify-end mb-2">
                  <Button
                    variant="ghost"
                    size="sm"
                    class="text-xs h-7"
                    @click="selector.toggleAllAudio()"
                  >
                    {{
                      selector.selectedAudios.value.size ===
                      streamInfo.audios.length
                        ? "取消全选"
                        : "全选"
                    }}
                  </Button>
                </div>
                <div class="space-y-2">
                  <button
                    v-for="audio in streamInfo.audios"
                    :key="audio.id"
                    class="w-full text-left p-3 rounded-lg border transition-all hover:bg-accent"
                    :class="
                      selector.selectedAudios.value.has(audio.id)
                        ? 'border-primary bg-primary/10'
                        : 'border-border'
                    "
                    @click="selector.toggleAudio(audio.id)"
                  >
                    <div class="flex items-center justify-between">
                      <div class="flex items-center gap-3">
                        <div
                          class="w-5 h-5 rounded border flex items-center justify-center"
                          :class="
                            selector.selectedAudios.value.has(audio.id)
                              ? 'border-primary bg-primary'
                              : 'border-muted-foreground'
                          "
                        >
                          <AppIcon
                            v-if="selector.selectedAudios.value.has(audio.id)"
                            name="Check"
                            :size="12"
                            class="text-primary-foreground"
                          />
                        </div>
                        <div>
                          <div class="font-medium flex items-center gap-2">
                            {{ getStreamName(audio) }}
                            <span
                              v-if="audio.isDefault"
                              class="text-xs text-primary"
                              >默认</span
                            >
                          </div>
                          <div class="text-sm text-muted-foreground">
                            {{ getAudioDescription(audio) }}
                          </div>
                        </div>
                      </div>
                      <div class="text-sm text-muted-foreground">
                        {{ formatBandwidth(audio.bandwidth) }}
                      </div>
                    </div>
                  </button>
                </div>
              </template>
            </TabsContent>

            <!-- 字幕流 -->
            <TabsContent value="subtitle" class="mt-0">
              <div
                v-if="streamInfo.subtitles.length === 0"
                class="py-8 text-center text-muted-foreground"
              >
                没有可用的字幕流
              </div>
              <template v-else>
                <div class="flex justify-end mb-2">
                  <Button
                    variant="ghost"
                    size="sm"
                    class="text-xs h-7"
                    @click="selector.toggleAllSubtitle()"
                  >
                    {{
                      selector.selectedSubtitles.value.size ===
                      streamInfo.subtitles.length
                        ? "取消全选"
                        : "全选"
                    }}
                  </Button>
                </div>
                <div class="space-y-2">
                  <button
                    v-for="subtitle in streamInfo.subtitles"
                    :key="subtitle.id"
                    class="w-full text-left p-3 rounded-lg border transition-all hover:bg-accent"
                    :class="
                      selector.selectedSubtitles.value.has(subtitle.id)
                        ? 'border-primary bg-primary/10'
                        : 'border-border'
                    "
                    @click="selector.toggleSubtitle(subtitle.id)"
                  >
                    <div class="flex items-center gap-3">
                      <div
                        class="w-5 h-5 rounded border flex items-center justify-center"
                        :class="
                          selector.selectedSubtitles.value.has(subtitle.id)
                            ? 'border-primary bg-primary'
                            : 'border-muted-foreground'
                        "
                      >
                        <AppIcon
                          v-if="
                            selector.selectedSubtitles.value.has(subtitle.id)
                          "
                          name="Check"
                          :size="12"
                          class="text-primary-foreground"
                        />
                      </div>
                      <div>
                        <div class="font-medium flex items-center gap-2">
                          {{ getStreamName(subtitle) }}
                          <span
                            v-if="subtitle.isDefault"
                            class="text-xs text-primary"
                            >默认</span
                          >
                          <span
                            v-if="subtitle.isForced"
                            class="text-xs text-yellow-500"
                            >强制</span
                          >
                        </div>
                        <div class="text-sm text-muted-foreground">
                          {{ getSubtitleDescription(subtitle) }}
                        </div>
                      </div>
                    </div>
                  </button>
                </div>
              </template>
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
