<script setup lang="ts">
/**
 * DownloadSettings - 下载设置组件
 */

import { computed } from 'vue';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Separator } from '@/components/ui/separator';
import {
  SettingSwitch,
  SettingInput,
  SettingSlider,
} from '..';

interface Settings {
  download: {
    threadCount: number;
    retryCount: number;
    timeout: number;
    maxSpeed: string;
    autoSelect: boolean;
    selectVideo: string;
    selectAudio: string;
    selectSubtitle: string;
    checkSegmentsCount: boolean;
    delAfterDone: boolean;
    skipMerge: boolean;
    writeMetaJson: boolean;
    binaryMerge: boolean;
    concurrentDownload: boolean;
  };
}

interface Props {
  settings: Settings;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: 'update:settings', value: any): void;
}>();

// 格式化线程数显示
const threadCountDisplay = computed(() => `${props.settings.download.threadCount} 线程`);

// 更新设置
const updateDownload = (value: any) => {
  emit('update:settings', value);
};
</script>

<template>
  <div class="space-y-4">
    <Card>
      <CardHeader>
        <CardTitle class="text-base">下载参数</CardTitle>
        <CardDescription>配置下载相关的核心参数</CardDescription>
      </CardHeader>
      <CardContent class="space-y-4">
        <SettingSlider
          :model-value="settings.download.threadCount"
          label="并发线程数"
          :min="1"
          :max="32"
          :step="1"
          :display-value="threadCountDisplay"
          @update:model-value="updateDownload({ threadCount: $event })"
        />

        <SettingInput
          :model-value="settings.download.retryCount"
          label="重试次数"
          type="number"
          :min="0"
          :max="10"
          class="w-24"
          @update:model-value="updateDownload({ retryCount: parseInt($event) || 3 })"
        />

        <SettingInput
          :model-value="settings.download.timeout"
          label="超时时间 (秒)"
          type="number"
          :min="5"
          :max="300"
          class="w-24"
          @update:model-value="updateDownload({ timeout: parseInt($event) || 30 })"
        />

        <SettingInput
          :model-value="settings.download.maxSpeed"
          label="最大下载速度"
          placeholder="0 = 不限制"
          class="w-32"
          @update:model-value="updateDownload({ maxSpeed: $event })"
        />
      </CardContent>
    </Card>

    <Card>
      <CardHeader>
        <CardTitle class="text-base">流选择</CardTitle>
        <CardDescription>默认选择视频/音频/字幕流</CardDescription>
      </CardHeader>
      <CardContent class="space-y-4">
        <SettingSwitch
          :model-value="settings.download.autoSelect"
          label="自动选择最佳流"
          description="自动选择最高质量的流"
          @update:model-value="updateDownload({ autoSelect: $event })"
        />

        <Separator />

        <SettingInput
          :model-value="settings.download.selectVideo"
          label="视频流选择"
          placeholder="例如: res=1080"
          class="flex-1"
          @update:model-value="updateDownload({ selectVideo: $event })"
        />

        <SettingInput
          :model-value="settings.download.selectAudio"
          label="音频流选择"
          placeholder="例如: lang=zh"
          class="flex-1"
          @update:model-value="updateDownload({ selectAudio: $event })"
        />

        <SettingInput
          :model-value="settings.download.selectSubtitle"
          label="字幕流选择"
          placeholder="例如: lang=zh"
          class="flex-1"
          @update:model-value="updateDownload({ selectSubtitle: $event })"
        />
      </CardContent>
    </Card>

    <Card>
      <CardHeader>
        <CardTitle class="text-base">下载选项</CardTitle>
      </CardHeader>
      <CardContent class="space-y-3">
        <SettingSwitch
          :model-value="settings.download.checkSegmentsCount"
          label="检查分片数量"
          @update:model-value="updateDownload({ checkSegmentsCount: $event })"
        />

        <SettingSwitch
          :model-value="settings.download.delAfterDone"
          label="完成后删除临时文件"
          @update:model-value="updateDownload({ delAfterDone: $event })"
        />

        <SettingSwitch
          :model-value="settings.download.skipMerge"
          label="跳过合并"
          @update:model-value="updateDownload({ skipMerge: $event })"
        />

        <SettingSwitch
          :model-value="settings.download.writeMetaJson"
          label="写入元数据 JSON"
          @update:model-value="updateDownload({ writeMetaJson: $event })"
        />

        <SettingSwitch
          :model-value="settings.download.binaryMerge"
          label="二进制合并"
          @update:model-value="updateDownload({ binaryMerge: $event })"
        />

        <SettingSwitch
          :model-value="settings.download.concurrentDownload"
          label="并发下载"
          @update:model-value="updateDownload({ concurrentDownload: $event })"
        />
      </CardContent>
    </Card>
  </div>
</template>
