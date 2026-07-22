<script setup lang="ts">
/**
 * SettingsView - 设置中心容器
 *
 * 双栏布局：左侧导航栏 + 右侧内容区
 * 4 个分区：常规·界面 | N_m3u8DL-RE | FFmpeg | 任务预设
 * 挂载时加载设置与预设。
 */

import { computed, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import type * as LucideIcons from "lucide-vue-next";
import { useSettingsStore, usePresetStore } from "@/stores";
import { AppIcon } from "@/components/common";
import GeneralTab from "@/components/settings/tabs/GeneralTab.vue";
import Nm3u8dlTab from "@/components/settings/tabs/Nm3u8dlTab.vue";
import FfmpegTab from "@/components/settings/tabs/FfmpegTab.vue";
import PresetsTab from "@/components/settings/tabs/PresetsTab.vue";

type IconName = keyof typeof LucideIcons;
type SectionId = "general" | "nm3u8dl" | "ffmpeg" | "presets";

interface Section {
  id: SectionId;
  icon: IconName;
  label: string;
}

const { t } = useI18n();
const settingsStore = useSettingsStore();
const presetStore = usePresetStore();

// 当前分区
const activeSection = ref<SectionId>("general");

// 导航分区（响应式 label，跟随语言切换）
const sections = computed<Section[]>(() => [
  {
    id: "general",
    icon: "Settings",
    label: t("settings.general.langAppearance", "常规·界面"),
  },
  { id: "nm3u8dl", icon: "Download", label: "N_m3u8DL-RE" },
  { id: "ffmpeg", icon: "Film", label: "FFmpeg" },
  {
    id: "presets",
    icon: "Bookmark",
    label: t("settings.preset.title", "任务预设"),
  },
]);

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
  <div class="flex h-full min-h-0">
    <!-- 左侧导航栏 -->
    <nav class="w-48 shrink-0 border-r border-border/60 px-3 py-5">
      <ul class="space-y-1">
        <li v-for="section in sections" :key="section.id">
          <button
            type="button"
            class="flex w-full cursor-pointer items-center gap-2.5 rounded-lg px-3 py-2 text-left text-sm transition-colors duration-150 ease-out"
            :class="
              activeSection === section.id
                ? 'bg-muted/60 text-foreground'
                : 'text-muted-foreground hover:bg-muted/30 hover:text-foreground'
            "
            @click="activeSection = section.id"
          >
            <AppIcon :name="section.icon" :size="16" class="shrink-0" />
            <span class="truncate">{{ section.label }}</span>
          </button>
        </li>
      </ul>
    </nav>

    <!-- 右侧内容区 -->
    <div class="flex-1 overflow-y-auto">
      <div class="mx-auto max-w-2xl px-8 py-8">
        <GeneralTab v-if="activeSection === 'general'" />
        <Nm3u8dlTab v-else-if="activeSection === 'nm3u8dl'" />
        <FfmpegTab v-else-if="activeSection === 'ffmpeg'" />
        <PresetsTab v-else-if="activeSection === 'presets'" />
      </div>
    </div>
  </div>
</template>
