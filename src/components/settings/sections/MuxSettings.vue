<script setup lang="ts">
/**
 * MuxSettings - 混流设置组件
 */

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { SettingSwitch, SettingSelect, SettingInput } from '..';

interface Settings {
  mux: {
    format: string;
    muxer: string;
    binPath: string;
    keepOriginal: boolean;
    skipSubtitles: boolean;
    noDateInfo: boolean;
    useConcatDemuxer: boolean;
  };
}

interface Props {
  settings: Settings;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: 'update:settings', value: any): void;
}>();

// 混流格式选项
const muxFormatOptions = [
  { value: 'mp4', label: 'MP4' },
  { value: 'mkv', label: 'MKV' },
];

// 混流器选项
const muxerOptions = [
  { value: 'ffmpeg', label: 'FFmpeg' },
  { value: 'mkvmerge', label: 'MKVMerge' },
];

// 更新设置
const updateMux = (value: any) => {
  emit('update:settings', value);
};
</script>

<template>
  <div class="space-y-4">
    <Card>
      <CardHeader>
        <CardTitle class="text-base">混流配置</CardTitle>
        <CardDescription>配置视频混流相关选项</CardDescription>
      </CardHeader>
      <CardContent class="space-y-4">
        <SettingSelect
          :model-value="settings.mux.format"
          label="输出格式"
          :options="muxFormatOptions"
          placeholder="选择格式"
          @update:model-value="updateMux({ format: $event })"
        />

        <SettingSelect
          :model-value="settings.mux.muxer"
          label="混流器"
          :options="muxerOptions"
          placeholder="选择混流器"
          @update:model-value="updateMux({ muxer: $event })"
        />

        <SettingInput
          :model-value="settings.mux.binPath"
          label="混流器路径"
          placeholder="留空则使用系统 PATH"
          class="flex-1"
          @update:model-value="updateMux({ binPath: $event })"
        />
      </CardContent>
    </Card>

    <Card>
      <CardHeader>
        <CardTitle class="text-base">混流选项</CardTitle>
      </CardHeader>
      <CardContent class="space-y-3">
        <SettingSwitch
          :model-value="settings.mux.keepOriginal"
          label="保留原始文件"
          description="混流后保留分离的音视频文件"
          @update:model-value="updateMux({ keepOriginal: $event })"
        />

        <SettingSwitch
          :model-value="settings.mux.skipSubtitles"
          label="跳过字幕"
          @update:model-value="updateMux({ skipSubtitles: $event })"
        />

        <SettingSwitch
          :model-value="settings.mux.noDateInfo"
          label="不包含日期信息"
          @update:model-value="updateMux({ noDateInfo: $event })"
        />

        <SettingSwitch
          :model-value="settings.mux.useConcatDemuxer"
          label="使用 Concat 解复用器"
          @update:model-value="updateMux({ useConcatDemuxer: $event })"
        />
      </CardContent>
    </Card>
  </div>
</template>
