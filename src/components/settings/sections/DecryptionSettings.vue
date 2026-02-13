<script setup lang="ts">
/**
 * DecryptionSettings - 解密设置组件
 */

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { SettingSelect, SettingInput, SettingSwitch } from '..';

interface Settings {
  decryption: {
    engine: string;
    binPath: string;
    keyTextFile: string;
    realTimeDecryption: boolean;
  };
}

interface Props {
  settings: Settings;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: 'update:settings', value: any): void;
}>();

// 解密引擎选项
const decryptionEngineOptions = [
  { value: 'FFMPEG', label: 'FFmpeg' },
  { value: 'MP4DECRYPT', label: 'MP4Decrypt' },
  { value: 'SHAKA_PACKAGER', label: 'Shaka Packager' },
];

// 更新设置
const updateDecryption = (value: any) => {
  emit('update:settings', value);
};
</script>

<template>
  <div class="space-y-4">
    <Card>
      <CardHeader>
        <CardTitle class="text-base">解密引擎</CardTitle>
        <CardDescription>配置 DRM 解密相关选项</CardDescription>
      </CardHeader>
      <CardContent class="space-y-4">
        <SettingSelect
          :model-value="settings.decryption.engine"
          label="解密引擎"
          :options="decryptionEngineOptions"
          placeholder="选择引擎"
          @update:model-value="updateDecryption({ engine: $event })"
        />

        <SettingInput
          :model-value="settings.decryption.binPath"
          label="解密器路径"
          placeholder="留空则使用系统 PATH"
          class="flex-1"
          @update:model-value="updateDecryption({ binPath: $event })"
        />

        <SettingInput
          :model-value="settings.decryption.keyTextFile"
          label="密钥文本文件"
          placeholder="包含密钥的文本文件路径"
          class="flex-1"
          @update:model-value="updateDecryption({ keyTextFile: $event })"
        />

        <SettingSwitch
          :model-value="settings.decryption.realTimeDecryption"
          label="实时解密"
          description="下载时实时解密分片"
          @update:model-value="updateDecryption({ realTimeDecryption: $event })"
        />
      </CardContent>
    </Card>
  </div>
</template>
