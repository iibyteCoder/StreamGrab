<script setup lang="ts">
/**
 * FfmpegTab - FFmpeg 工具标签页
 *
 * 顶部 ToolManagerCard + 混流默认 + 直链下载卡片。
 * 全部数据源：settingsStore.ffmpegConfig
 * 更新：settingsStore.updateFfmpegConfig(partial)
 */

import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { useSettingsStore } from "@/stores";
import ToolManagerCard from "../ToolManagerCard.vue";
import MuxSettings from "../sections/MuxSettings.vue";
import type { MuxFields } from "../sections/MuxSettings.vue";
import { SettingSwitch, SettingInput, SettingsGroup } from "..";
import type { FfmpegConfig } from "@/domain";
import type { DeepPartial } from "@/services";

const { t } = useI18n();
const settingsStore = useSettingsStore();
const config = computed(() => settingsStore.ffmpegConfig);

// ========================================
// 工具路径変更回調
// ========================================

function handlePathChange(path: string) {
  settingsStore.updateFfmpegConfig({ ffmpeg_path: path });
}

// ========================================
// 通用 patch
// ========================================

function patch(patch: DeepPartial<FfmpegConfig>) {
  settingsStore.updateFfmpegConfig(patch);
}

// ========================================
// 混流字段提取
// ========================================

const muxFields = computed<MuxFields>(() => ({
  mux_format: config.value.mux_format,
  muxer: config.value.muxer,
  mux_bin_path: config.value.mux_bin_path,
  mux_skip_subtitles: config.value.mux_skip_subtitles,
  mux_keep_original: config.value.mux_keep_original,
}));
</script>

<template>
  <div class="space-y-6">
    <!-- 工具管理卡片 -->
    <ToolManagerCard
      tool-id="ffmpeg"
      :config-path="config.ffmpeg_path"
      @path-change="handlePathChange"
    />

    <!-- 混流默认（复用 rewired 组件） -->
    <MuxSettings :mux="muxFields" @update="patch" />

    <!-- 直链下载 -->
    <SettingsGroup
      :title="t('settings.ffmpeg.directDownload', '直链下载')"
      :description="
        t('settings.ffmpeg.directDownloadDesc', '配置直链视频下载的默认参数')
      "
    >
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
              patch({ timeout: parseInt(String($event)) || 60 })
            "
          />

          <SettingInput
            :padded="false"
            :model-value="config.max_speed"
            :label="t('settings.download.maxSpeed', '限速 (0=不限)')"
            placeholder="0"
            @update:model-value="patch({ max_speed: String($event) })"
          />

          <SettingInput
            :padded="false"
            :model-value="config.connection_timeout"
            :label="t('settings.ffmpeg.connectionTimeout', '连接超时 (秒)')"
            type="number"
            :min="5"
            :max="120"
            @update:model-value="
              patch({
                connection_timeout: parseInt(String($event)) || 30,
              })
            "
          />

          <SettingInput
            :padded="false"
            :model-value="config.reconnect_attempts"
            :label="t('settings.ffmpeg.reconnectAttempts', '重连次数')"
            type="number"
            :min="0"
            :max="10"
            @update:model-value="
              patch({
                reconnect_attempts: parseInt(String($event)) || 3,
              })
            "
          />

          <SettingInput
            :padded="false"
            :model-value="config.reconnect_delay"
            :label="t('settings.ffmpeg.reconnectDelay', '重连间隔 (秒)')"
            type="number"
            :min="1"
            :max="60"
            @update:model-value="
              patch({
                reconnect_delay: parseInt(String($event)) || 5,
              })
            "
          />
        </div>
      </div>

      <div class="px-5 py-1.5">
        <div class="grid grid-cols-2 gap-x-8">
          <SettingSwitch
            :padded="false"
            :model-value="config.overwrite_existing"
            :label="t('settings.ffmpeg.overwriteExisting', '覆盖已有文件')"
            @update:model-value="patch({ overwrite_existing: $event })"
          />

          <SettingSwitch
            :padded="false"
            :model-value="config.preserve_timestamps"
            :label="t('settings.ffmpeg.preserveTimestamps', '保留时间戳')"
            @update:model-value="patch({ preserve_timestamps: $event })"
          />
        </div>
      </div>

      <SettingInput
        :model-value="config.user_agent || ''"
        label="User-Agent"
        :placeholder="t('settings.ffmpeg.leaveEmptyDefault', '留空使用默认')"
        @update:model-value="patch({ user_agent: String($event) || null })"
      />

      <SettingInput
        :model-value="config.referer || ''"
        label="Referer"
        :placeholder="t('settings.ffmpeg.leaveEmptyDefault', '留空使用默认')"
        @update:model-value="patch({ referer: String($event) || null })"
      />
    </SettingsGroup>
  </div>
</template>
