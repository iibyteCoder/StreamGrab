<script setup lang="ts">
/**
 * Nm3u8dlTab - N_m3u8DL-RE 工具标签页
 *
 * 顶部 ToolManagerCard + 下载默认 + 网络 + 解密 + 直播卡片。
 * 全部数据源：settingsStore.nm3u8dlConfig
 * 更新：settingsStore.updateNm3u8dlConfig(partial)
 */

import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { useSettingsStore } from "@/stores";
import ToolManagerCard from "../ToolManagerCard.vue";
import NetworkSettings from "../sections/NetworkSettings.vue";
import DecryptionSettings from "../sections/DecryptionSettings.vue";
import LiveSettings from "../sections/LiveSettings.vue";
import type { LiveFields } from "../sections/LiveSettings.vue";
import {
  SettingSwitch,
  SettingInput,
  SettingSlider,
  SettingSelect,
  SettingsGroup,
} from "..";
import type { Nm3u8dlConfig, SubtitleFormat } from "@/domain";
import type { DeepPartial } from "@/services";

const { t } = useI18n();
const settingsStore = useSettingsStore();
const config = computed(() => settingsStore.nm3u8dlConfig);

// ========================================
// 工具路径变更回调
// ========================================

function handlePathChange(path: string) {
  settingsStore.updateNm3u8dlConfig({ path });
}

// ========================================
// 通用 patch
// ========================================

function patch(patch: DeepPartial<Nm3u8dlConfig>) {
  settingsStore.updateNm3u8dlConfig(patch);
}

// ========================================
// 直播字段提取
// ========================================

const liveFields = computed<LiveFields>(() => ({
  live_perform_as_vod: config.value.live_perform_as_vod,
  live_real_time_merge: config.value.live_real_time_merge,
  live_keep_segments: config.value.live_keep_segments,
  live_pipe_mux: config.value.live_pipe_mux,
  live_fix_vtt_by_audio: config.value.live_fix_vtt_by_audio,
  live_record_limit: config.value.live_record_limit,
  live_wait_time: config.value.live_wait_time,
  live_take_count: config.value.live_take_count,
}));

// ========================================
// 选项
// ========================================

const subFormatOptions = [
  { value: "SRT", label: "SRT" },
  { value: "VTT", label: "WebVTT" },
];

const threadCountDisplay = computed(() => `${config.value.thread_count} 线程`);
</script>

<template>
  <div class="space-y-6">
    <!-- 工具管理卡片 -->
    <ToolManagerCard
      tool-id="nm3u8dl"
      :config-path="config.path"
      @path-change="handlePathChange"
    />

    <!-- 下载参数 -->
    <SettingsGroup
      :title="t('settings.download.basic', '下载参数')"
      :description="t('settings.download.basicDesc', '配置下载相关的核心参数')"
    >
      <SettingSlider
        :model-value="config.thread_count"
        :label="t('settings.download.threadCount', '并发线程数')"
        :min="1"
        :max="32"
        :step="1"
        :display-value="threadCountDisplay"
        @update:model-value="patch({ thread_count: $event })"
      />

      <div class="px-5 py-1.5">
        <div class="grid grid-cols-3 gap-x-4">
          <SettingInput
            :padded="false"
            :model-value="config.retry_count"
            :label="t('settings.download.retryCount', '重试次数')"
            type="number"
            :min="0"
            :max="10"
            @update:model-value="
              patch({ retry_count: parseInt(String($event)) || 3 })
            "
          />

          <SettingInput
            :padded="false"
            :model-value="config.timeout"
            :label="t('settings.download.timeout', '超时 (秒)')"
            type="number"
            :min="5"
            :max="300"
            @update:model-value="
              patch({ timeout: parseInt(String($event)) || 100 })
            "
          />

          <SettingInput
            :padded="false"
            :model-value="config.max_speed"
            :label="t('settings.download.maxSpeed', '限速 (0=不限)')"
            :placeholder="t('settings.download.maxSpeedPlaceholder', '0')"
            @update:model-value="patch({ max_speed: String($event) })"
          />
        </div>
      </div>
    </SettingsGroup>

    <!-- 流选择 -->
    <SettingsGroup
      :title="t('settings.download.streamSelection', '流选择')"
      :description="
        t(
          'settings.download.streamSelectionDesc',
          '默认选择/排除视频、音频、字幕流',
        )
      "
    >
      <SettingSwitch
        :model-value="config.auto_select"
        :label="t('settings.download.autoSelect', '自动选择最佳流')"
        :description="
          t('settings.download.autoSelectDesc', '自动选择最高质量的流')
        "
        @update:model-value="patch({ auto_select: $event })"
      />

      <SettingInput
        :model-value="config.select_video || ''"
        :label="t('settings.download.selectVideo', '视频流选择')"
        :placeholder="
          t('settings.download.selectVideoPlaceholder', '例如: res=1080')
        "
        @update:model-value="patch({ select_video: String($event) || null })"
      />

      <SettingInput
        :model-value="config.select_audio || ''"
        :label="t('settings.download.selectAudio', '音频流选择')"
        :placeholder="
          t('settings.download.selectAudioPlaceholder', '例如: lang=zh')
        "
        @update:model-value="patch({ select_audio: String($event) || null })"
      />

      <SettingInput
        :model-value="config.select_subtitle || ''"
        :label="t('settings.download.selectSubtitle', '字幕流选择')"
        :placeholder="
          t('settings.download.selectSubtitlePlaceholder', '例如: lang=zh')
        "
        @update:model-value="patch({ select_subtitle: String($event) || null })"
      />

      <SettingInput
        :model-value="config.drop_video || ''"
        :label="t('settings.download.dropVideo', '排除视频流')"
        :placeholder="
          t('settings.download.dropVideoPlaceholder', '例如: codecs=av01')
        "
        @update:model-value="patch({ drop_video: String($event) || null })"
      />

      <SettingInput
        :model-value="config.drop_audio || ''"
        :label="t('settings.download.dropAudio', '排除音频流')"
        :placeholder="
          t('settings.download.dropAudioPlaceholder', '例如: lang=ja')
        "
        @update:model-value="patch({ drop_audio: String($event) || null })"
      />

      <SettingInput
        :model-value="config.drop_subtitle || ''"
        :label="t('settings.download.dropSubtitle', '排除字幕流')"
        :placeholder="
          t('settings.download.dropSubtitlePlaceholder', '例如: name=forced')
        "
        @update:model-value="patch({ drop_subtitle: String($event) || null })"
      />
    </SettingsGroup>

    <!-- 下载选项 -->
    <SettingsGroup :title="t('settings.download.downloadOptions', '下载选项')">
      <div class="px-5 py-1.5">
        <div class="grid grid-cols-2 gap-x-8">
          <SettingSwitch
            :padded="false"
            :model-value="config.check_segments_count"
            :label="t('settings.download.checkSegmentsCount', '检查分片数量')"
            @update:model-value="patch({ check_segments_count: $event })"
          />

          <SettingSwitch
            :padded="false"
            :model-value="config.del_after_done"
            :label="t('settings.download.deleteTemp', '完成后删除临时文件')"
            @update:model-value="patch({ del_after_done: $event })"
          />

          <SettingSwitch
            :padded="false"
            :model-value="config.skip_merge"
            :label="t('settings.download.autoMerge', '跳过合并')"
            @update:model-value="patch({ skip_merge: $event })"
          />

          <SettingSwitch
            :padded="false"
            :model-value="config.write_meta_json"
            :label="t('settings.download.writeMetaJson', '写入元数据 JSON')"
            @update:model-value="patch({ write_meta_json: $event })"
          />

          <SettingSwitch
            :padded="false"
            :model-value="config.binary_merge"
            :label="t('settings.download.binaryMerge', '二进制合并')"
            @update:model-value="patch({ binary_merge: $event })"
          />

          <SettingSwitch
            :padded="false"
            :model-value="config.concurrent_download"
            :label="t('settings.download.concurrentDownload', '并发下载')"
            @update:model-value="patch({ concurrent_download: $event })"
          />

          <SettingSwitch
            :padded="false"
            :model-value="config.allow_hls_multi_ext_map"
            :label="t('settings.download.allowMultiExtMap', '允许多 EXT-X-MAP')"
            :description="
              t(
                'settings.download.allowMultiExtMapDesc',
                '允许 HLS 多个 EXT-X-MAP 标签',
              )
            "
            @update:model-value="patch({ allow_hls_multi_ext_map: $event })"
          />

          <SettingSwitch
            :padded="false"
            :model-value="config.use_ffmpeg_concat_demuxer"
            :label="
              t('settings.download.useConcatDemuxer', '使用 Concat 解复用器')
            "
            @update:model-value="patch({ use_ffmpeg_concat_demuxer: $event })"
          />

          <SettingSwitch
            :padded="false"
            :model-value="config.no_date_info"
            :label="t('settings.download.noDateInfo', '不包含日期信息')"
            @update:model-value="patch({ no_date_info: $event })"
          />
        </div>
      </div>

      <SettingInput
        :model-value="config.url_processor_args || ''"
        :label="t('settings.download.urlProcessorArgs', 'URL 处理器参数')"
        :placeholder="
          t(
            'settings.download.urlProcessorArgsPlaceholder',
            '传递给 URL 处理器的额外参数',
          )
        "
        @update:model-value="
          patch({ url_processor_args: String($event) || null })
        "
      />
    </SettingsGroup>

    <!-- 字幕设置 -->
    <SettingsGroup
      :title="t('settings.download.subtitleSettings', '字幕设置')"
      :description="
        t('settings.download.subtitleSettingsDesc', '配置字幕下载相关选项')
      "
    >
      <SettingSelect
        :model-value="config.sub_format"
        :label="t('settings.download.subtitleFormat', '字幕格式')"
        :options="subFormatOptions"
        @update:model-value="patch({ sub_format: $event as SubtitleFormat })"
      />

      <SettingSwitch
        :model-value="config.auto_subtitle_fix"
        :label="t('settings.download.autoFixTimeline', '自动修正时间轴')"
        :description="
          t('settings.download.autoFixTimelineDesc', '自动修正字幕时间轴偏移')
        "
        @update:model-value="patch({ auto_subtitle_fix: $event })"
      />

      <SettingSwitch
        :model-value="config.sub_only"
        :label="t('settings.download.downloadSubtitleOnly', '仅下载字幕')"
        :description="
          t(
            'settings.download.downloadSubtitleOnlyDesc',
            '只下载字幕文件，不下载视频',
          )
        "
        @update:model-value="patch({ sub_only: $event })"
      />
    </SettingsGroup>

    <!-- 网络设置（复用 rewired 组件） -->
    <NetworkSettings :network="config.network" @update="patch" />

    <!-- 解密设置（复用 rewired 组件） -->
    <DecryptionSettings :decryption="config.decryption" @update="patch" />

    <!-- 直播设置（复用 rewired 组件） -->
    <LiveSettings :live="liveFields" @update="patch" />
  </div>
</template>
