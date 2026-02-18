<script setup lang="ts">
/**
 * UISettings - 界面设置组件
 */

import { SettingSelect, SettingSwitch, SettingsGroup } from "..";

interface Settings {
  ui: {
    showNotification: boolean;
    clipboardWatch: boolean;
  };
}

interface Props {
  settings: Settings;
  theme: string;
}

defineProps<Props>();

const emit = defineEmits<{
  (e: "update:settings", value: any): void;
  (e: "update:theme", value: any): void;
}>();

// 主题选项
const themeOptions = [
  { value: "light", label: "浅色" },
  { value: "dark", label: "深色" },
  { value: "system", label: "跟随系统" },
];

// 更新设置
const updateUI = (value: any) => {
  emit("update:settings", value);
};

// 更新主题
const updateTheme = (value: any) => {
  emit("update:theme", value);
};
</script>

<template>
  <div class="space-y-2">
    <SettingsGroup title="外观" description="自定义应用程序外观">
      <SettingSelect
        :model-value="theme"
        label="主题"
        :options="themeOptions"
        placeholder="选择主题"
        @update:model-value="updateTheme($event)"
      />

      <SettingSwitch
        :model-value="settings.ui.showNotification"
        label="显示通知"
        description="下载完成时显示系统通知"
        @update:model-value="updateUI({ showNotification: $event })"
      />

      <SettingSwitch
        :model-value="settings.ui.clipboardWatch"
        label="剪贴板监视"
        description="自动检测剪贴板中的 M3U8 链接"
        @update:model-value="updateUI({ clipboardWatch: $event })"
      />
    </SettingsGroup>
  </div>
</template>
