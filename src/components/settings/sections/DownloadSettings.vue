<script setup lang="ts">
/**
 * DownloadSettings - 下载设置 UI 组件
 */

import { computed } from 'vue';
import { Separator } from '@/components/ui/separator';
import { Button } from '@/components/ui/button';
import { AppIcon } from '@/components/common';
import {
  SettingSwitch,
  SettingInput,
  SettingSlider,
  SettingSelect,
  SettingsGroup,
} from '..';
import type { DownloadSettings } from '@/types';

interface Props {
  settings: { download: DownloadSettings };
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: 'update:settings', value: Partial<DownloadSettings>): void;
}>();

// 格式化线程数显示
const threadCountDisplay = computed(() => `${props.settings.download.threadCount} 线程`);

// 更新设置
const updateDownload = (value: Partial<DownloadSettings>) => {
  emit('update:settings', value);
};

// 广告过滤关键字
const adKeywords = computed(() => props.settings.download.adFilter.keywords);

// 添加广告关键字
const addAdKeyword = () => {
  const keywords = [...adKeywords.value, ''];
  updateDownload({ adFilter: { ...props.settings.download.adFilter, keywords } });
};

// 删除广告关键字
const removeAdKeyword = (index: number) => {
  const keywords = adKeywords.value.filter((_, i) => i !== index);
  updateDownload({ adFilter: { ...props.settings.download.adFilter, keywords } });
};

// 更新广告关键字
const updateAdKeyword = (index: number, value: string) => {
  const keywords = [...adKeywords.value];
  keywords[index] = value;
  updateDownload({ adFilter: { ...props.settings.download.adFilter, keywords } });
};

// 切换广告过滤
const toggleAdFilter = (enabled: boolean) => {
  updateDownload({ adFilter: { ...props.settings.download.adFilter, enabled } });
};

// 字幕格式选项
const subFormatOptions = [
  { value: 'SRT', label: 'SRT' },
  { value: 'VTT', label: 'WebVTT' },
];
</script>

<template>
  <div class="space-y-2">
    <SettingsGroup title="下载参数" description="配置下载相关的核心参数">
      <SettingSlider
        :model-value="settings.download.threadCount"
        label="并发线程数"
        :min="1"
        :max="32"
        :step="1"
        :display-value="threadCountDisplay"
        @update:model-value="updateDownload({ threadCount: $event })"
      />

      <div class="grid grid-cols-3 gap-4">
        <SettingInput
          :model-value="settings.download.retryCount"
          label="重试次数"
          type="number"
          :min="0"
          :max="10"
          @update:model-value="updateDownload({ retryCount: parseInt(String($event)) || 3 })"
        />

        <SettingInput
          :model-value="settings.download.timeout"
          label="超时 (秒)"
          type="number"
          :min="5"
          :max="300"
          @update:model-value="updateDownload({ timeout: parseInt(String($event)) || 30 })"
        />

        <SettingInput
          :model-value="settings.download.maxSpeed"
          label="限速 (0=不限)"
          placeholder="0"
          @update:model-value="updateDownload({ maxSpeed: String($event) })"
        />
      </div>
    </SettingsGroup>

    <SettingsGroup title="流选择" description="默认选择视频/音频/字幕流">
      <SettingSwitch
        :model-value="settings.download.autoSelect"
        label="自动选择最佳流"
        description="自动选择最高质量的流"
        @update:model-value="updateDownload({ autoSelect: $event })"
      />

      <Separator class="my-4" />

      <SettingInput
        :model-value="settings.download.selectVideo"
        label="视频流选择"
        placeholder="例如: res=1080"
        @update:model-value="updateDownload({ selectVideo: String($event) })"
      />

      <SettingInput
        :model-value="settings.download.selectAudio"
        label="音频流选择"
        placeholder="例如: lang=zh"
        @update:model-value="updateDownload({ selectAudio: String($event) })"
      />

      <SettingInput
        :model-value="settings.download.selectSubtitle"
        label="字幕流选择"
        placeholder="例如: lang=zh"
        @update:model-value="updateDownload({ selectSubtitle: String($event) })"
      />
    </SettingsGroup>

    <SettingsGroup title="流排除" description="通过正则表达式排除不需要的流">
      <SettingInput
        :model-value="settings.download.dropVideo"
        label="排除视频流"
        placeholder="例如: codecs=av01"
        @update:model-value="updateDownload({ dropVideo: String($event) })"
      />

      <SettingInput
        :model-value="settings.download.dropAudio"
        label="排除音频流"
        placeholder="例如: lang=ja"
        @update:model-value="updateDownload({ dropAudio: String($event) })"
      />

      <SettingInput
        :model-value="settings.download.dropSubtitle"
        label="排除字幕流"
        placeholder="例如: name=forced"
        @update:model-value="updateDownload({ dropSubtitle: String($event) })"
      />
    </SettingsGroup>

    <SettingsGroup title="广告过滤" description="通过 URL 关键字跳过广告分片">
      <SettingSwitch
        :model-value="settings.download.adFilter.enabled"
        label="启用广告过滤"
        description="匹配关键字的分片将被跳过"
        @update:model-value="toggleAdFilter($event)"
      />

      <template v-if="settings.download.adFilter.enabled">
        <Separator class="my-4" />

        <div class="flex items-center justify-between">
          <span class="text-sm font-medium">过滤关键字（正则表达式）</span>
          <Button variant="outline" size="sm" @click="addAdKeyword">
            <AppIcon name="Plus" :size="14" class="mr-1" />
            添加
          </Button>
        </div>

        <div v-if="adKeywords.length === 0" class="text-sm text-muted-foreground py-2">
          暂无过滤关键字
        </div>

        <div v-else class="space-y-2">
          <div
            v-for="(_, index) in adKeywords"
            :key="index"
            class="flex items-center gap-2"
          >
            <input
              :value="adKeywords[index]"
              type="text"
              placeholder="例如: ad\.domain\.com"
              class="flex-1 h-9 px-3 text-sm rounded-md border border-input bg-transparent focus:outline-none focus:ring-2 focus:ring-ring"
              @input="updateAdKeyword(index, ($event.target as HTMLInputElement).value)"
            />
            <Button
              variant="ghost"
              size="icon"
              class="h-9 w-9 text-destructive hover:text-destructive"
              @click="removeAdKeyword(index)"
            >
              <AppIcon name="Trash2" :size="16" />
            </Button>
          </div>
        </div>
      </template>
    </SettingsGroup>

    <SettingsGroup title="下载选项">
      <div class="grid grid-cols-2 gap-x-8 gap-y-4">
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
      </div>
    </SettingsGroup>

    <SettingsGroup title="字幕设置" description="配置字幕下载相关选项">
      <SettingSelect
        :model-value="settings.download.subFormat"
        label="字幕格式"
        :options="subFormatOptions"
        @update:model-value="updateDownload({ subFormat: $event as 'SRT' | 'VTT' })"
      />

      <SettingSwitch
        :model-value="settings.download.autoSubtitleFix"
        label="自动修正时间轴"
        description="自动修正字幕时间轴偏移"
        @update:model-value="updateDownload({ autoSubtitleFix: $event })"
      />

      <SettingSwitch
        :model-value="settings.download.subOnly"
        label="仅下载字幕"
        description="只下载字幕文件，不下载视频"
        @update:model-value="updateDownload({ subOnly: $event })"
      />
    </SettingsGroup>
  </div>
</template>
