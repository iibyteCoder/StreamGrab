<script setup lang="ts">
/**
 * SettingsView - 设置中心容器
 *
 * 4 标签页：常规·界面 | N_m3u8DL-RE | FFmpeg | 任务预设
 * 挂载时加载设置与预设。
 */

import { onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { useSettingsStore } from "@/stores";
import { usePresetStore } from "@/stores";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import { AppIcon } from "@/components/common";
import GeneralTab from "@/components/settings/tabs/GeneralTab.vue";
import Nm3u8dlTab from "@/components/settings/tabs/Nm3u8dlTab.vue";
import FfmpegTab from "@/components/settings/tabs/FfmpegTab.vue";
import PresetsTab from "@/components/settings/tabs/PresetsTab.vue";

const { t } = useI18n();
const settingsStore = useSettingsStore();
const presetStore = usePresetStore();

onMounted(async () => {
  if (!settingsStore.loaded) {
    await settingsStore.loadSettings();
  }
  if (!presetStore.loaded) {
    await presetStore.loadPresets();
  }
});
</script>

<template>
  <div class="flex h-full flex-col">
    <!-- 标题栏 -->
    <div
      class="shrink-0 flex items-center justify-between px-6 py-4"
      style="border-bottom: 1px solid rgba(255, 255, 255, 0.08)"
    >
      <div class="flex items-center gap-3">
        <AppIcon
          name="Settings"
          :size="20"
          style="color: var(--accent-primary)"
        />
        <h1 class="text-lg font-semibold" style="color: var(--text-primary)">
          {{ t("settings.title") }}
        </h1>
      </div>
    </div>

    <!-- 标签页 -->
    <Tabs default-value="general" class="flex-1 flex flex-col overflow-hidden">
      <div class="shrink-0 px-6 pt-4">
        <TabsList class="w-full justify-start">
          <TabsTrigger value="general" class="cursor-pointer">
            <AppIcon name="Settings" :size="14" class="mr-1.5" />
            {{ t("settings.general.langAppearance", "常规·界面") }}
          </TabsTrigger>
          <TabsTrigger value="nm3u8dl" class="cursor-pointer">
            <AppIcon name="Download" :size="14" class="mr-1.5" />
            N_m3u8DL-RE
          </TabsTrigger>
          <TabsTrigger value="ffmpeg" class="cursor-pointer">
            <AppIcon name="Film" :size="14" class="mr-1.5" />
            FFmpeg
          </TabsTrigger>
          <TabsTrigger value="presets" class="cursor-pointer">
            <AppIcon name="Bookmark" :size="14" class="mr-1.5" />
            {{ t("settings.preset.title", "任务预设") }}
          </TabsTrigger>
        </TabsList>
      </div>

      <!-- 内容区 -->
      <div class="flex-1 overflow-y-auto">
        <div class="mx-auto max-w-3xl px-6 py-6">
          <TabsContent value="general" class="mt-0">
            <GeneralTab />
          </TabsContent>

          <TabsContent value="nm3u8dl" class="mt-0">
            <Nm3u8dlTab />
          </TabsContent>

          <TabsContent value="ffmpeg" class="mt-0">
            <FfmpegTab />
          </TabsContent>

          <TabsContent value="presets" class="mt-0">
            <PresetsTab />
          </TabsContent>
        </div>
      </div>
    </Tabs>
  </div>
</template>
