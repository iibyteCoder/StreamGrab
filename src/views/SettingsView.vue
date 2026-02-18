<script setup lang="ts">
/**
 * SettingsView - 设置页面
 * 左侧导航 + 右侧内容的分栏布局
 */

import { ref, onMounted, watch } from "vue";
import { useSettings, useToast } from "@/composables";
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

const {
  settings,
  theme,
  isLoaded,
  resetSettings,
  setTheme,
  updateGeneral,
  updateDownload,
  updateMux,
  updateNetwork,
  updateDecryption,
  updateLive,
  updateAdvanced,
  updateUi,
  enableAutoSave,
} = useSettings();
const toast = useToast();

const activeTab = ref("general");

onMounted(() => {
  if (isLoaded.value) {
    enableAutoSave(500);
  }
});

// 当 settings 加载完成后启用自动保存
watch(isLoaded, (loaded) => {
  if (loaded) {
    enableAutoSave(500);
  }
});

const handleReset = async () => {
  try {
    await resetSettings();
    toast.success("设置已恢复为默认值");
  } catch {
    toast.error("恢复默认设置失败");
  }
};

// 获取当前 tab 的标题
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
            :settings="settings"
            @update:settings="updateGeneral($event)"
          />
        </div>

        <div v-show="activeTab === 'templates'">
          <TemplateManager />
        </div>

        <div v-show="activeTab === 'download'">
          <DownloadSettings
            :settings="settings"
            @update:settings="updateDownload($event)"
          />
        </div>

        <div v-show="activeTab === 'mux'">
          <MuxSettings
            :settings="settings"
            @update:settings="updateMux($event)"
          />
        </div>

        <div v-show="activeTab === 'network'">
          <NetworkSettings
            :settings="settings"
            @update:settings="updateNetwork($event)"
          />
        </div>

        <div v-show="activeTab === 'decryption'">
          <DecryptionSettings
            :settings="settings"
            @update:settings="updateDecryption($event)"
          />
        </div>

        <div v-show="activeTab === 'live'">
          <LiveSettings
            :settings="settings"
            @update:settings="updateLive($event)"
          />
        </div>

        <div v-show="activeTab === 'advanced'">
          <AdvancedSettings
            :settings="settings"
            @update:settings="updateAdvanced($event)"
            @reset="handleReset"
          />
        </div>

        <div v-show="activeTab === 'ui'">
          <UISettings
            :settings="settings"
            :theme="theme"
            @update:settings="updateUi($event)"
            @update:theme="setTheme($event)"
          />
        </div>
      </div>
    </div>
  </div>
</template>
