<script setup lang="ts">
/**
 * NetworkSettings - 网络设置组件
 */

import { computed } from 'vue';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Switch } from '@/components/ui/switch';
import { AppIcon } from '@/components/common';
import { SettingSwitch, SettingInput } from '..';
import type { HeaderConfig } from '@/types';

interface Settings {
  network: {
    useSystemProxy: boolean;
    customProxy: string;
    baseUrl: string;
    appendUrlParams: boolean;
    headers: HeaderConfig[];
  };
}

interface Props {
  settings: Settings;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: 'update:settings', value: any): void;
}>();

// 请求头列表
const headers = computed(() => props.settings.network.headers);

// 更新设置
const updateNetwork = (value: any) => {
  emit('update:settings', value);
};

// 添加请求头
const addHeader = () => {
  const newHeaders = [...headers.value, { key: '', value: '', enabled: true }];
  updateNetwork({ headers: newHeaders });
};

// 删除请求头
const removeHeader = (index: number) => {
  const newHeaders = headers.value.filter((_, i) => i !== index);
  updateNetwork({ headers: newHeaders });
};

// 更新请求头 Key
const updateHeaderKey = (index: number, key: string) => {
  const newHeaders = [...headers.value];
  const header = newHeaders[index];
  if (header) {
    newHeaders[index] = { key, value: header.value, enabled: header.enabled };
  }
  updateNetwork({ headers: newHeaders });
};

// 更新请求头 Value
const updateHeaderValue = (index: number, value: string) => {
  const newHeaders = [...headers.value];
  const header = newHeaders[index];
  if (header) {
    newHeaders[index] = { key: header.key, value, enabled: header.enabled };
  }
  updateNetwork({ headers: newHeaders });
};

// 切换启用状态
const toggleHeader = (index: number, enabled: boolean) => {
  const newHeaders = [...headers.value];
  const header = newHeaders[index];
  if (header) {
    newHeaders[index] = { key: header.key, value: header.value, enabled };
  }
  updateNetwork({ headers: newHeaders });
};
</script>

<template>
  <div class="space-y-4">
    <!-- 代理设置 -->
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

    <!-- 请求头设置 -->
    <Card>
      <CardHeader>
        <CardTitle class="text-base">请求头设置</CardTitle>
        <CardDescription>配置 HTTP 请求头（如 Referer、User-Agent、Cookie）</CardDescription>
      </CardHeader>
      <CardContent class="space-y-4">
        <div v-if="headers.length === 0" class="text-sm text-muted-foreground py-2">
          暂无自定义请求头，点击下方按钮添加
        </div>

        <div v-else class="space-y-2">
          <div
            v-for="(header, index) in headers"
            :key="index"
            class="flex items-center gap-2"
          >
            <!-- Key 输入 -->
            <input
              :value="header.key"
              type="text"
              placeholder="Header Name"
              class="w-36 h-9 px-3 text-sm rounded-md border border-input bg-transparent focus:outline-none focus:ring-2 focus:ring-ring"
              @input="updateHeaderKey(index, ($event.target as HTMLInputElement).value)"
            />
            <!-- Value 输入 -->
            <input
              :value="header.value"
              type="text"
              placeholder="Header Value"
              class="flex-1 h-9 px-3 text-sm rounded-md border border-input bg-transparent focus:outline-none focus:ring-2 focus:ring-ring"
              @input="updateHeaderValue(index, ($event.target as HTMLInputElement).value)"
            />
            <!-- 启用开关 -->
            <Switch
              :checked="header.enabled"
              @update:checked="toggleHeader(index, $event)"
            />
            <!-- 删除按钮 -->
            <Button
              variant="ghost"
              size="icon"
              class="h-9 w-9 text-destructive hover:text-destructive"
              @click="removeHeader(index)"
            >
              <AppIcon name="Trash2" :size="16" />
            </Button>
          </div>
        </div>

        <Button variant="outline" size="sm" @click="addHeader">
          <AppIcon name="Plus" :size="14" class="mr-1" />
          添加请求头
        </Button>
      </CardContent>
    </Card>
  </div>
</template>
