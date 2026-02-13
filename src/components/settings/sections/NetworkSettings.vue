<script setup lang="ts">
/**
 * NetworkSettings - 网络设置组件
 */

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { SettingSwitch, SettingInput } from '..';

interface Settings {
  network: {
    useSystemProxy: boolean;
    customProxy: string;
    baseUrl: string;
    appendUrlParams: boolean;
  };
}

interface Props {
  settings: Settings;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: 'update:settings', value: any): void;
}>();

// 更新设置
const updateNetwork = (value: any) => {
  emit('update:settings', value);
};
</script>

<template>
  <div class="space-y-4">
    <Card>
      <CardHeader>
        <CardTitle class="text-base">代理设置</CardTitle>
        <CardDescription>配置网络代理选项</CardDescription>
      </CardHeader>
      <CardContent class="space-y-4">
        <SettingSwitch
          :model-value="settings.network.useSystemProxy"
          label="使用系统代理"
          @update:model-value="updateNetwork({ useSystemProxy: $event })"
        />

        <SettingInput
          :model-value="settings.network.customProxy"
          label="自定义代理"
          placeholder="http://127.0.0.1:7890"
          class="flex-1"
          @update:model-value="updateNetwork({ customProxy: $event })"
        />

        <SettingInput
          :model-value="settings.network.baseUrl"
          label="Base URL"
          placeholder="替换 URL 的基础部分"
          class="flex-1"
          @update:model-value="updateNetwork({ baseUrl: $event })"
        />

        <SettingSwitch
          :model-value="settings.network.appendUrlParams"
          label="附加 URL 参数"
          @update:model-value="updateNetwork({ appendUrlParams: $event })"
        />
      </CardContent>
    </Card>
  </div>
</template>
