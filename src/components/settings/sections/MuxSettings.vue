<script setup lang="ts">
/**
 * MuxSettings - 混流设置 UI 组件
 *
 * 数据源：FfmpegConfig 的 mux_* 字段
 * 更新：emit DeepPartial<FfmpegConfig>
 *
 * 注意：no_date_info / use_ffmpeg_concat_demuxer 属于 Nm3u8dlConfig，
 * 由 Nm3u8dlTab 直接管理，不在此组件内。
 */

import { useI18n } from "vue-i18n";
import { SettingSwitch, SettingSelect, SettingInput, SettingsGroup } from "..";
import type { FfmpegConfig, MuxFormat, Muxer } from "@/domain";
import type { DeepPartial } from "@/services";

/** 混流相关字段提取 */
export interface MuxFields {
  mux_format: MuxFormat;
  muxer: Muxer;
  mux_bin_path: string | null;
  mux_skip_subtitles: boolean;
  mux_keep_original: boolean;
}

interface Props {
  mux: MuxFields;
}

defineProps<Props>();

const { t } = useI18n();

const emit = defineEmits<{
  (e: "update", value: DeepPartial<FfmpegConfig>): void;
}>();

// 混流格式选项
const muxFormatOptions = [
  { value: "mp4", label: "MP4" },
  { value: "mkv", label: "MKV" },
];

// 混流器选项
const muxerOptions = [
  { value: "ffmpeg", label: "FFmpeg" },
  { value: "mkvmerge", label: "MKVMerge" },
];

function patchMux(patch: Partial<MuxFields>) {
  emit("update", patch as DeepPartial<FfmpegConfig>);
}
</script>

<template>
  <SettingsGroup
    :title="t('settings.mux.title', '混流配置')"
    :description="t('settings.mux.desc', '配置视频混流相关选项')"
  >
    <SettingSelect
      :model-value="mux.mux_format"
      :label="t('settings.mux.format')"
      :options="muxFormatOptions"
      placeholder="选择格式"
      @update:model-value="patchMux({ mux_format: $event as MuxFormat })"
    />

    <SettingSelect
      :model-value="mux.muxer"
      :label="t('settings.mux.muxer')"
      :options="muxerOptions"
      placeholder="选择混流器"
      @update:model-value="patchMux({ muxer: $event as Muxer })"
    />

    <SettingInput
      :model-value="mux.mux_bin_path || ''"
      :label="t('settings.mux.muxerPath', '混流器路径')"
      placeholder="留空则使用系统 PATH"
      @update:model-value="patchMux({ mux_bin_path: String($event) || null })"
    />
  </SettingsGroup>

  <SettingsGroup :title="t('settings.mux.muxOptions', '混流选项')">
    <div class="grid grid-cols-2 gap-x-8 gap-y-4">
      <SettingSwitch
        :model-value="mux.mux_keep_original"
        :label="t('settings.mux.keepOriginal')"
        :description="
          t('settings.mux.keepOriginalDesc', '混流后保留分离的音视频文件')
        "
        @update:model-value="patchMux({ mux_keep_original: $event })"
      />

      <SettingSwitch
        :model-value="mux.mux_skip_subtitles"
        :label="t('settings.mux.skipSubtitles', '跳过字幕')"
        @update:model-value="patchMux({ mux_skip_subtitles: $event })"
      />
    </div>
  </SettingsGroup>
</template>
