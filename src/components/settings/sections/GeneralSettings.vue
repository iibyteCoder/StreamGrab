<script setup lang="ts">
/**
 * GeneralSettings - 常规设置组件
 */

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import {
  SettingSwitch,
  SettingSelect,
  SettingPath,
} from '..';

interface Settings {
  general: {
    saveDir: string;
    tmpDir: string;
    language: string;
    autoStartDownload: boolean;
    minimizeToTray: boolean;
    checkUpdate: boolean;
  };
}

interface Props {
  settings: Settings;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: 'update:settings', value: any): void;
}>();

// 语言选项
const languageOptions = [
  { value: 'zh-CN', label: '简体中文' },
  { value: 'zh-TW', label: '繁体中文' },
  { value: 'en-US', label: 'English' },
];

// 更新设置
const updateGeneral = (value: any) => {
  emit('update:settings', value);
};
</script>

<template>
  <div class="space-y-4">
    <Card>
      <CardHeader>
        <CardTitle class="text-base">存储位置</CardTitle>
        <CardDescription>设置下载和临时文件的保存位置</CardDescription>
      </CardHeader>
      <CardContent class="space-y-4">
        <SettingPath
          :model-value="settings.general.saveDir"
          label="下载目录"
          placeholder="./downloads"
          @update:model-value="updateGeneral({ saveDir: $event })"
          @select="updateGeneral({ saveDir: $event })"
        />
        <SettingPath
          :model-value="settings.general.tmpDir"
          label="临时目录"
          placeholder="./temp"
          @update:model-value="updateGeneral({ tmpDir: $event })"
          @select="updateGeneral({ tmpDir: $event })"
        />
      </CardContent>
    </Card>

    <Card>
      <CardHeader>
        <CardTitle class="text-base">应用行为</CardTitle>
        <CardDescription>配置应用程序的默认行为</CardDescription>
      </CardHeader>
      <CardContent class="space-y-4">
        <SettingSelect
          :model-value="settings.general.language"
          label="语言"
          :options="languageOptions"
          placeholder="选择语言"
          @update:model-value="updateGeneral({ language: $event })"
        />

        <SettingSwitch
          :model-value="settings.general.autoStartDownload"
          label="自动开始下载"
          description="添加任务后自动开始下载"
          @update:model-value="updateGeneral({ autoStartDownload: $event })"
        />

        <SettingSwitch
          :model-value="settings.general.minimizeToTray"
          label="最小化到托盘"
          description="关闭窗口时最小化到系统托盘"
          @update:model-value="updateGeneral({ minimizeToTray: $event })"
        />

        <SettingSwitch
          :model-value="settings.general.checkUpdate"
          label="检查更新"
          description="启动时自动检查新版本"
          @update:model-value="updateGeneral({ checkUpdate: $event })"
        />
      </CardContent>
    </Card>
  </div>
</template>
