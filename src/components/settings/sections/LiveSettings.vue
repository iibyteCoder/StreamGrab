<script setup lang="ts">
/**
 * LiveSettings - 直播设置组件
 *
 * 数据源：Nm3u8dlConfig 的 live_* 字段
 * 更新：emit DeepPartial<Nm3u8dlConfig>
 */

import { useI18n } from "vue-i18n";
import { SettingSwitch, SettingInput, SettingsGroup } from "..";
import type { Nm3u8dlConfig } from "@/domain";
import type { DeepPartial } from "@/services";

/** 直播相关字段提取 */
export interface LiveFields {
  live_perform_as_vod: boolean;
  live_real_time_merge: boolean;
  live_keep_segments: boolean;
  live_pipe_mux: boolean;
  live_fix_vtt_by_audio: boolean;
  live_record_limit: string | null;
  live_wait_time: number;
  live_take_count: number;
}

interface Props {
  live: LiveFields;
}

defineProps<Props>();

const { t } = useI18n();

const emit = defineEmits<{
  (e: "update", value: DeepPartial<Nm3u8dlConfig>): void;
}>();

function patchLive(patch: Partial<LiveFields>) {
  emit("update", patch as DeepPartial<Nm3u8dlConfig>);
}
</script>

<template>
  <SettingsGroup
    :title="t('settings.live.mode')"
    :description="t('settings.live.modeDesc', '配置直播流录制选项')"
  >
    <div class="px-5 py-1.5">
      <div class="grid grid-cols-2 gap-x-8">
        <SettingSwitch
          :padded="false"
          :model-value="live.live_perform_as_vod"
          :label="t('settings.live.performAsVod', '作为 VOD 处理')"
          :description="
            t('settings.live.performAsVodDesc', '将直播流当作点播处理')
          "
          @update:model-value="patchLive({ live_perform_as_vod: $event })"
        />

        <SettingSwitch
          :padded="false"
          :model-value="live.live_real_time_merge"
          :label="t('settings.live.realtimeMerge')"
          @update:model-value="patchLive({ live_real_time_merge: $event })"
        />

        <SettingSwitch
          :padded="false"
          :model-value="live.live_keep_segments"
          :label="t('settings.live.keepSegments')"
          @update:model-value="patchLive({ live_keep_segments: $event })"
        />

        <SettingSwitch
          :padded="false"
          :model-value="live.live_pipe_mux"
          :label="t('settings.live.pipeMux', '管道混流')"
          @update:model-value="patchLive({ live_pipe_mux: $event })"
        />

        <SettingSwitch
          :padded="false"
          :model-value="live.live_fix_vtt_by_audio"
          :label="t('settings.live.fixVttByAudio', '通过音频修复 VTT')"
          @update:model-value="patchLive({ live_fix_vtt_by_audio: $event })"
        />
      </div>
    </div>

    <div class="px-5 py-1.5">
      <div class="grid grid-cols-3 gap-x-4">
        <SettingInput
          :padded="false"
          :model-value="live.live_record_limit || ''"
          :label="t('settings.live.durationLimit')"
          placeholder="1:30:00"
          @update:model-value="
            patchLive({ live_record_limit: String($event) || null })
          "
        />

        <SettingInput
          :padded="false"
          :model-value="live.live_wait_time"
          :label="t('settings.live.waitTime')"
          type="number"
          :min="0"
          @update:model-value="
            patchLive({ live_wait_time: parseInt(String($event)) || 0 })
          "
        />

        <SettingInput
          :padded="false"
          :model-value="live.live_take_count"
          :label="t('settings.live.segmentCount')"
          type="number"
          :min="0"
          @update:model-value="
            patchLive({ live_take_count: parseInt(String($event)) || 0 })
          "
        />
      </div>
    </div>
  </SettingsGroup>
</template>
