<script setup lang="ts">
/**
 * LiveSettings - 直播设置组件
 */

import { Separator } from "@/components/ui/separator";
import { SettingSwitch, SettingInput, SettingsGroup } from "..";

interface Settings {
  live: {
    performAsVod: boolean;
    realTimeMerge: boolean;
    keepSegments: boolean;
    pipeMux: boolean;
    fixVttByAudio: boolean;
    recordLimit: string;
    waitTime: number;
    takeCount: number;
  };
}

interface Props {
  settings: Settings;
}

defineProps<Props>();

const emit = defineEmits<{
  (e: "update:settings", value: any): void;
}>();

// 更新设置
const updateLive = (value: any) => {
  emit("update:settings", value);
};
</script>

<template>
  <div class="space-y-2">
    <SettingsGroup title="直播录制" description="配置直播流录制选项">
      <div class="grid grid-cols-2 gap-x-8 gap-y-4">
        <SettingSwitch
          :model-value="settings.live.performAsVod"
          label="作为 VOD 处理"
          description="将直播流当作点播处理"
          @update:model-value="updateLive({ performAsVod: $event })"
        />

        <SettingSwitch
          :model-value="settings.live.realTimeMerge"
          label="实时合并"
          @update:model-value="updateLive({ realTimeMerge: $event })"
        />

        <SettingSwitch
          :model-value="settings.live.keepSegments"
          label="保留分片"
          @update:model-value="updateLive({ keepSegments: $event })"
        />

        <SettingSwitch
          :model-value="settings.live.pipeMux"
          label="管道混流"
          @update:model-value="updateLive({ pipeMux: $event })"
        />

        <SettingSwitch
          :model-value="settings.live.fixVttByAudio"
          label="通过音频修复 VTT"
          @update:model-value="updateLive({ fixVttByAudio: $event })"
        />
      </div>

      <Separator class="my-4" />

      <div class="grid grid-cols-3 gap-4">
        <SettingInput
          :model-value="settings.live.recordLimit"
          label="录制限制"
          placeholder="1:30:00"
          @update:model-value="updateLive({ recordLimit: $event })"
        />

        <SettingInput
          :model-value="settings.live.waitTime"
          label="等待 (秒)"
          type="number"
          :min="0"
          @update:model-value="
            updateLive({ waitTime: parseInt(String($event)) || 0 })
          "
        />

        <SettingInput
          :model-value="settings.live.takeCount"
          label="分片数"
          type="number"
          :min="0"
          @update:model-value="
            updateLive({ takeCount: parseInt(String($event)) || 0 })
          "
        />
      </div>
    </SettingsGroup>
  </div>
</template>
