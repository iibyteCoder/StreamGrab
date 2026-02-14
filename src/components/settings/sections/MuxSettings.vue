<script setup lang="ts">
/**
 * MuxSettings - 混流设置 UI 组件
 * 只负责 UI 展示
 */

import { computed } from 'vue';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { AppIcon } from '@/components/common';
import { SettingSwitch, SettingSelect, SettingInput } from '..';
import type { MuxSettings, MuxImport } from '@/types';

interface Props {
  settings: { mux: MuxSettings };
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: 'update:settings', value: Partial<MuxSettings>): void;
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

// 外部媒体列表
const muxImports = computed(() => props.settings.mux.muxImports);

// 更新设置
const updateMux = (value: Partial<MuxSettings>) => {
  emit('update:settings', value);
};

// 添加外部媒体
const addMuxImport = () => {
  const newImports: MuxImport[] = [...muxImports.value, { path: '' }];
  updateMux({ muxImports: newImports });
};

// 删除外部媒体
const removeMuxImport = (index: number) => {
  const newImports = muxImports.value.filter((_, i) => i !== index);
  updateMux({ muxImports: newImports });
};

// 更新外部媒体路径
const updateMuxImportPath = (index: number, path: string) => {
  const newImports = [...muxImports.value];
  if (newImports[index]) {
    newImports[index] = { ...newImports[index], path };
  }
  updateMux({ muxImports: newImports });
};

// 更新外部媒体语言
const updateMuxImportLang = (index: number, lang: string) => {
  const newImports = [...muxImports.value];
  if (newImports[index]) {
    newImports[index] = { ...newImports[index], lang };
  }
  updateMux({ muxImports: newImports });
};

// 更新外部媒体名称
const updateMuxImportName = (index: number, name: string) => {
  const newImports = [...muxImports.value];
  if (newImports[index]) {
    newImports[index] = { ...newImports[index], name };
  }
  updateMux({ muxImports: newImports });
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
          @update:model-value="updateMux({ format: $event as 'mp4' | 'mkv' })"
        />

        <SettingSelect
          :model-value="settings.mux.muxer"
          label="混流器"
          :options="muxerOptions"
          placeholder="选择混流器"
          @update:model-value="updateMux({ muxer: $event as 'ffmpeg' | 'mkvmerge' })"
        />

        <SettingInput
          :model-value="settings.mux.binPath"
          label="混流器路径"
          placeholder="留空则使用系统 PATH"
          class="flex-1"
          @update:model-value="updateMux({ binPath: String($event) })"
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

    <!-- 外部媒体导入 -->
    <Card>
      <CardHeader>
        <CardTitle class="text-base">外部媒体导入</CardTitle>
        <CardDescription>导入外部音频或字幕文件进行混流</CardDescription>
      </CardHeader>
      <CardContent class="space-y-4">
        <div v-if="muxImports.length === 0" class="text-sm text-muted-foreground py-2">
          暂无外部媒体，点击下方按钮添加
        </div>

        <div v-else class="space-y-3">
          <div
            v-for="(item, index) in muxImports"
            :key="index"
            class="space-y-2"
          >
            <div class="flex items-center gap-2">
              <input
                :value="item.path"
                type="text"
                placeholder="文件路径"
                class="flex-1 h-9 px-3 text-sm rounded-md border border-input bg-transparent focus:outline-none focus:ring-2 focus:ring-ring"
                @input="updateMuxImportPath(index, ($event.target as HTMLInputElement).value)"
              />
              <Button
                variant="ghost"
                size="icon"
                class="h-9 w-9 text-destructive hover:text-destructive"
                @click="removeMuxImport(index)"
              >
                <AppIcon name="Trash2" :size="16" />
              </Button>
            </div>
            <div class="flex items-center gap-2">
              <input
                :value="item.lang || ''"
                type="text"
                placeholder="语言 (如: zh, en)"
                class="w-28 h-8 px-2 text-sm rounded-md border border-input bg-transparent focus:outline-none focus:ring-2 focus:ring-ring"
                @input="updateMuxImportLang(index, ($event.target as HTMLInputElement).value)"
              />
              <input
                :value="item.name || ''"
                type="text"
                placeholder="名称 (可选)"
                class="flex-1 h-8 px-2 text-sm rounded-md border border-input bg-transparent focus:outline-none focus:ring-2 focus:ring-ring"
                @input="updateMuxImportName(index, ($event.target as HTMLInputElement).value)"
              />
            </div>
          </div>
        </div>

        <Button variant="outline" size="sm" @click="addMuxImport">
          <AppIcon name="Plus" :size="14" class="mr-1" />
          添加外部媒体
        </Button>
      </CardContent>
    </Card>
  </div>
</template>
