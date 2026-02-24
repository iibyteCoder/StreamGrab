<script setup lang="ts">
/**
 * SettingsView - 设置页面
 * 左侧导航 + 右侧内容的分栏布局
 */

import { ref, computed } from "vue";
import { useSettingsStore } from "@/stores";
import { useToast } from "@/composables";
import SettingsNav from "@/components/settings/SettingsNav.vue";
import {
  GeneralSettings,
  DownloadSettings,
  MuxSettings,
  NetworkSettings,
  DecryptionSettings,
  LiveSettings,
  AdvancedSettings,
  UISettings,
} from "@/components/settings/sections";
import { TemplateManager } from "@/components/template";

const settingsStore = useSettingsStore();
const toast = useToast();

const activeTab = ref("general");

// ============================================
// 适配层：将新配置结构转换为组件期望的格式
// ============================================

// 常规设置（组件期望的格式）
const generalSettings = computed(() => ({
  saveDir: settingsStore.appSettings.default_save_dir,
  tmpDir: settingsStore.appSettings.default_tmp_dir,
  language: settingsStore.appSettings.language,
  autoStartDownload: settingsStore.appSettings.auto_start_download,
  minimizeToTray: settingsStore.appSettings.minimize_to_tray,
  checkUpdate: settingsStore.appSettings.check_update,
}));

// 下载设置（组件期望的格式）
const downloadSettings = computed(() => ({
  threadCount: settingsStore.m3u8dlSettings.thread_count,
  retryCount: settingsStore.m3u8dlSettings.retry_count,
  timeout: settingsStore.m3u8dlSettings.timeout,
  maxSpeed: settingsStore.m3u8dlSettings.max_speed,
  autoSelect: settingsStore.m3u8dlSettings.auto_select,
  selectVideo: settingsStore.m3u8dlSettings.select_video || "",
  selectAudio: settingsStore.m3u8dlSettings.select_audio || "",
  selectSubtitle: settingsStore.m3u8dlSettings.select_subtitle || "",
  dropVideo: settingsStore.m3u8dlSettings.drop_video || "",
  dropAudio: settingsStore.m3u8dlSettings.drop_audio || "",
  dropSubtitle: settingsStore.m3u8dlSettings.drop_subtitle || "",
  savePattern: { enabled: false, template: "", presetId: "basic" },
  adFilter: { enabled: false, keywords: [] },
  checkSegmentsCount: settingsStore.m3u8dlSettings.check_segments_count,
  delAfterDone: settingsStore.m3u8dlSettings.del_after_done,
  skipMerge: settingsStore.m3u8dlSettings.skip_merge,
  writeMetaJson: settingsStore.m3u8dlSettings.write_meta_json,
  binaryMerge: settingsStore.m3u8dlSettings.binary_merge,
  concurrentDownload: settingsStore.m3u8dlSettings.concurrent_download,
  subOnly: settingsStore.m3u8dlSettings.sub_only,
  subFormat: settingsStore.m3u8dlSettings.sub_format,
  autoSubtitleFix: settingsStore.m3u8dlSettings.auto_subtitle_fix,
}));

// 混流设置
const muxSettings = computed(() => ({
  format: settingsStore.m3u8dlSettings.mux_format,
  muxer: settingsStore.m3u8dlSettings.muxer,
  binPath: settingsStore.m3u8dlSettings.mux_bin_path || "",
  keepOriginal: settingsStore.m3u8dlSettings.mux_keep_original,
  skipSubtitles: settingsStore.m3u8dlSettings.mux_skip_subtitles,
  noDateInfo: settingsStore.m3u8dlSettings.no_date_info,
  useConcatDemuxer: settingsStore.m3u8dlSettings.use_ffmpeg_concat_demuxer,
  muxImports: [],
}));

// 网络设置
const networkSettings = computed(() => ({
  useSystemProxy: settingsStore.networkSettings.use_system_proxy,
  customProxy: settingsStore.networkSettings.custom_proxy || "",
  headers: settingsStore.networkHeaders.map((h) => ({
    key: h.name,
    value: h.value,
    enabled: h.enabled,
  })),
  baseUrl: settingsStore.networkSettings.base_url || "",
  appendUrlParams: settingsStore.networkSettings.append_url_params,
}));

// 解密设置
const decryptionSettings = computed(() => ({
  keys: settingsStore.decryptionKeys.map((k) => ({
    kid: k.kid || "",
    key: k.key,
  })),
  keyTextFile: settingsStore.decryptionSettings.key_text_file || "",
  engine: settingsStore.decryptionSettings.decryption_engine,
  binPath: settingsStore.decryptionSettings.decryption_bin_path || "",
  realTimeDecryption: settingsStore.decryptionSettings.real_time_decryption,
  customHls: {
    enabled: settingsStore.decryptionSettings.custom_hls_enabled,
    method: settingsStore.decryptionSettings.custom_hls_method,
    key: {
      type: settingsStore.decryptionSettings.custom_hls_key_type,
      value: settingsStore.decryptionSettings.custom_hls_key_value || "",
    },
    iv: {
      type: settingsStore.decryptionSettings.custom_hls_iv_type,
      value: settingsStore.decryptionSettings.custom_hls_iv_value || "",
    },
  },
}));

// 直播设置
const liveSettings = computed(() => ({
  performAsVod: settingsStore.m3u8dlSettings.live_perform_as_vod,
  realTimeMerge: settingsStore.m3u8dlSettings.live_real_time_merge,
  keepSegments: settingsStore.m3u8dlSettings.live_keep_segments,
  pipeMux: settingsStore.m3u8dlSettings.live_pipe_mux,
  fixVttByAudio: settingsStore.m3u8dlSettings.live_fix_vtt_by_audio,
  recordLimit: settingsStore.m3u8dlSettings.live_record_limit || "",
  waitTime: settingsStore.m3u8dlSettings.live_wait_time,
  takeCount: settingsStore.m3u8dlSettings.live_take_count,
}));

// 高级设置
const advancedSettings = computed(() => ({
  ffmpegPath: settingsStore.ffmpegSettings.ffmpeg_path,
  n_m3u8dlPath: settingsStore.m3u8dlSettings.n_m3u8dl_path,
  logLevel: settingsStore.appSettings.log_level,
  logFilePath: settingsStore.appSettings.log_file_path,
  noLog: settingsStore.appSettings.no_log,
  allowHlsMultiExtMap: settingsStore.m3u8dlSettings.allow_hls_multi_ext_map,
  disableUpdateCheck: !settingsStore.appSettings.check_update,
  urlProcessorArgs: settingsStore.m3u8dlSettings.url_processor_args || "",
}));

// UI 设置
const uiSettings = computed(() => ({
  theme: settingsStore.appSettings.theme,
  showNotification: settingsStore.appSettings.show_notification,
  clipboardWatch: settingsStore.appSettings.clipboard_watch,
}));

// ============================================
// 更新处理函数
// ============================================

const handleUpdateGeneral = async (value: any) => {
  if (value.saveDir !== undefined)
    await settingsStore.updateAppField("default_save_dir", value.saveDir);
  if (value.tmpDir !== undefined)
    await settingsStore.updateAppField("default_tmp_dir", value.tmpDir);
  if (value.language !== undefined)
    await settingsStore.setLanguage(value.language);
  if (value.autoStartDownload !== undefined)
    await settingsStore.updateAppField(
      "auto_start_download",
      value.autoStartDownload,
    );
  if (value.minimizeToTray !== undefined)
    await settingsStore.updateAppField(
      "minimize_to_tray",
      value.minimizeToTray,
    );
  if (value.checkUpdate !== undefined)
    await settingsStore.updateAppField("check_update", value.checkUpdate);
};

const handleUpdateDownload = async (value: any) => {
  if (value.threadCount !== undefined)
    await settingsStore.updateM3U8DLField("thread_count", value.threadCount);
  if (value.retryCount !== undefined)
    await settingsStore.updateM3U8DLField("retry_count", value.retryCount);
  if (value.timeout !== undefined)
    await settingsStore.updateM3U8DLField("timeout", value.timeout);
  if (value.maxSpeed !== undefined)
    await settingsStore.updateM3U8DLField("max_speed", value.maxSpeed);
  if (value.autoSelect !== undefined)
    await settingsStore.updateM3U8DLField("auto_select", value.autoSelect);
  if (value.selectVideo !== undefined)
    await settingsStore.updateM3U8DLField(
      "select_video",
      value.selectVideo || null,
    );
  if (value.selectAudio !== undefined)
    await settingsStore.updateM3U8DLField(
      "select_audio",
      value.selectAudio || null,
    );
  if (value.selectSubtitle !== undefined)
    await settingsStore.updateM3U8DLField(
      "select_subtitle",
      value.selectSubtitle || null,
    );
  if (value.dropVideo !== undefined)
    await settingsStore.updateM3U8DLField(
      "drop_video",
      value.dropVideo || null,
    );
  if (value.dropAudio !== undefined)
    await settingsStore.updateM3U8DLField(
      "drop_audio",
      value.dropAudio || null,
    );
  if (value.dropSubtitle !== undefined)
    await settingsStore.updateM3U8DLField(
      "drop_subtitle",
      value.dropSubtitle || null,
    );
  if (value.checkSegmentsCount !== undefined)
    await settingsStore.updateM3U8DLField(
      "check_segments_count",
      value.checkSegmentsCount,
    );
  if (value.delAfterDone !== undefined)
    await settingsStore.updateM3U8DLField("del_after_done", value.delAfterDone);
  if (value.skipMerge !== undefined)
    await settingsStore.updateM3U8DLField("skip_merge", value.skipMerge);
  if (value.writeMetaJson !== undefined)
    await settingsStore.updateM3U8DLField(
      "write_meta_json",
      value.writeMetaJson,
    );
  if (value.binaryMerge !== undefined)
    await settingsStore.updateM3U8DLField("binary_merge", value.binaryMerge);
  if (value.concurrentDownload !== undefined)
    await settingsStore.updateM3U8DLField(
      "concurrent_download",
      value.concurrentDownload,
    );
  if (value.subOnly !== undefined)
    await settingsStore.updateM3U8DLField("sub_only", value.subOnly);
  if (value.subFormat !== undefined)
    await settingsStore.updateM3U8DLField("sub_format", value.subFormat);
  if (value.autoSubtitleFix !== undefined)
    await settingsStore.updateM3U8DLField(
      "auto_subtitle_fix",
      value.autoSubtitleFix,
    );
};

const handleUpdateMux = async (value: any) => {
  if (value.format !== undefined)
    await settingsStore.updateM3U8DLField("mux_format", value.format);
  if (value.muxer !== undefined)
    await settingsStore.updateM3U8DLField("muxer", value.muxer);
  if (value.binPath !== undefined)
    await settingsStore.updateM3U8DLField(
      "mux_bin_path",
      value.binPath || null,
    );
  if (value.keepOriginal !== undefined)
    await settingsStore.updateM3U8DLField(
      "mux_keep_original",
      value.keepOriginal,
    );
  if (value.skipSubtitles !== undefined)
    await settingsStore.updateM3U8DLField(
      "mux_skip_subtitles",
      value.skipSubtitles,
    );
  if (value.noDateInfo !== undefined)
    await settingsStore.updateM3U8DLField("no_date_info", value.noDateInfo);
  if (value.useConcatDemuxer !== undefined)
    await settingsStore.updateM3U8DLField(
      "use_ffmpeg_concat_demuxer",
      value.useConcatDemuxer,
    );
};

const handleUpdateNetwork = async (value: any) => {
  if (value.useSystemProxy !== undefined)
    await settingsStore.updateNetworkField(
      "use_system_proxy",
      value.useSystemProxy,
    );
  if (value.customProxy !== undefined)
    await settingsStore.updateNetworkField(
      "custom_proxy",
      value.customProxy || null,
    );
  if (value.baseUrl !== undefined)
    await settingsStore.updateNetworkField("base_url", value.baseUrl || null);
  if (value.appendUrlParams !== undefined)
    await settingsStore.updateNetworkField(
      "append_url_params",
      value.appendUrlParams,
    );
};

const handleUpdateDecryption = async (value: any) => {
  if (value.keyTextFile !== undefined)
    await settingsStore.updateDecryptionField(
      "key_text_file",
      value.keyTextFile || null,
    );
  if (value.engine !== undefined)
    await settingsStore.updateDecryptionField(
      "decryption_engine",
      value.engine,
    );
  if (value.binPath !== undefined)
    await settingsStore.updateDecryptionField(
      "decryption_bin_path",
      value.binPath || null,
    );
  if (value.realTimeDecryption !== undefined)
    await settingsStore.updateDecryptionField(
      "real_time_decryption",
      value.realTimeDecryption,
    );
  if (value.customHls !== undefined) {
    await settingsStore.updateDecryptionField(
      "custom_hls_enabled",
      value.customHls.enabled,
    );
    if (value.customHls.method !== undefined)
      await settingsStore.updateDecryptionField(
        "custom_hls_method",
        value.customHls.method,
      );
    if (value.customHls.key !== undefined) {
      await settingsStore.updateDecryptionField(
        "custom_hls_key_type",
        value.customHls.key.type,
      );
      await settingsStore.updateDecryptionField(
        "custom_hls_key_value",
        value.customHls.key.value || null,
      );
    }
    if (value.customHls.iv !== undefined) {
      await settingsStore.updateDecryptionField(
        "custom_hls_iv_type",
        value.customHls.iv.type,
      );
      await settingsStore.updateDecryptionField(
        "custom_hls_iv_value",
        value.customHls.iv.value || null,
      );
    }
  }
};

const handleUpdateLive = async (value: any) => {
  if (value.performAsVod !== undefined)
    await settingsStore.updateM3U8DLField(
      "live_perform_as_vod",
      value.performAsVod,
    );
  if (value.realTimeMerge !== undefined)
    await settingsStore.updateM3U8DLField(
      "live_real_time_merge",
      value.realTimeMerge,
    );
  if (value.keepSegments !== undefined)
    await settingsStore.updateM3U8DLField(
      "live_keep_segments",
      value.keepSegments,
    );
  if (value.pipeMux !== undefined)
    await settingsStore.updateM3U8DLField("live_pipe_mux", value.pipeMux);
  if (value.fixVttByAudio !== undefined)
    await settingsStore.updateM3U8DLField(
      "live_fix_vtt_by_audio",
      value.fixVttByAudio,
    );
  if (value.recordLimit !== undefined)
    await settingsStore.updateM3U8DLField(
      "live_record_limit",
      value.recordLimit || null,
    );
  if (value.waitTime !== undefined)
    await settingsStore.updateM3U8DLField("live_wait_time", value.waitTime);
  if (value.takeCount !== undefined)
    await settingsStore.updateM3U8DLField("live_take_count", value.takeCount);
};

const handleUpdateAdvanced = async (value: any) => {
  if (value.ffmpegPath !== undefined)
    await settingsStore.updateFFmpegField("ffmpeg_path", value.ffmpegPath);
  if (value.n_m3u8dlPath !== undefined)
    await settingsStore.updateM3U8DLField("n_m3u8dl_path", value.n_m3u8dlPath);
  if (value.logLevel !== undefined)
    await settingsStore.updateAppField("log_level", value.logLevel);
  if (value.logFilePath !== undefined)
    await settingsStore.updateAppField("log_file_path", value.logFilePath);
  if (value.noLog !== undefined)
    await settingsStore.updateAppField("no_log", value.noLog);
  if (value.allowHlsMultiExtMap !== undefined)
    await settingsStore.updateM3U8DLField(
      "allow_hls_multi_ext_map",
      value.allowHlsMultiExtMap,
    );
  if (value.disableUpdateCheck !== undefined)
    await settingsStore.updateAppField(
      "check_update",
      !value.disableUpdateCheck,
    );
  if (value.urlProcessorArgs !== undefined)
    await settingsStore.updateM3U8DLField(
      "url_processor_args",
      value.urlProcessorArgs || null,
    );
};

const handleUpdateUi = async (value: any) => {
  if (value.theme !== undefined) await settingsStore.setTheme(value.theme);
  if (value.showNotification !== undefined)
    await settingsStore.updateAppField(
      "show_notification",
      value.showNotification,
    );
  if (value.clipboardWatch !== undefined)
    await settingsStore.updateAppField("clipboard_watch", value.clipboardWatch);
};

const handleReset = async () => {
  try {
    await settingsStore.resetSettings();
    toast.success("设置已恢复为默认值");
  } catch {
    toast.error("恢复默认设置失败");
  }
};

// Tab 标题
const getTabTitle = (tab: string): string => {
  const titles: Record<string, string> = {
    general: "常规设置",
    templates: "下载模板",
    download: "下载设置",
    mux: "混流设置",
    network: "网络设置",
    decryption: "解密设置",
    live: "直播设置",
    advanced: "高级设置",
    ui: "界面设置",
  };
  return titles[tab] || "设置";
};
</script>

<template>
  <div class="flex h-full">
    <!-- 左侧导航 -->
    <SettingsNav v-model="activeTab" />

    <!-- 右侧内容区 -->
    <div class="flex-1 overflow-y-auto">
      <div class="mx-auto max-w-3xl p-6">
        <!-- 页面标题 -->
        <div class="mb-6">
          <h1 class="text-xl font-semibold text-foreground">
            {{ getTabTitle(activeTab) }}
          </h1>
        </div>

        <!-- 内容区域 -->
        <div v-show="activeTab === 'general'">
          <GeneralSettings
            :settings="{ general: generalSettings }"
            @update:settings="handleUpdateGeneral"
          />
        </div>

        <div v-show="activeTab === 'templates'">
          <TemplateManager />
        </div>

        <div v-show="activeTab === 'download'">
          <DownloadSettings
            :settings="{ download: downloadSettings }"
            @update:settings="handleUpdateDownload"
          />
        </div>

        <div v-show="activeTab === 'mux'">
          <MuxSettings
            :settings="{ mux: muxSettings }"
            @update:settings="handleUpdateMux"
          />
        </div>

        <div v-show="activeTab === 'network'">
          <NetworkSettings
            :settings="{ network: networkSettings }"
            @update:settings="handleUpdateNetwork"
          />
        </div>

        <div v-show="activeTab === 'decryption'">
          <DecryptionSettings
            :settings="{ decryption: decryptionSettings }"
            @update:settings="handleUpdateDecryption"
          />
        </div>

        <div v-show="activeTab === 'live'">
          <LiveSettings
            :settings="{ live: liveSettings }"
            @update:settings="handleUpdateLive"
          />
        </div>

        <div v-show="activeTab === 'advanced'">
          <AdvancedSettings
            :settings="{ advanced: advancedSettings }"
            @update:settings="handleUpdateAdvanced"
            @reset="handleReset"
          />
        </div>

        <div v-show="activeTab === 'ui'">
          <UISettings
            :settings="{ ui: uiSettings }"
            :theme="settingsStore.appSettings.theme"
            @update:settings="handleUpdateUi"
            @update:theme="settingsStore.setTheme"
          />
        </div>
      </div>
    </div>
  </div>
</template>
