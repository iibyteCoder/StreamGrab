<script setup lang="ts">
/**
 * NetworkSettings - 网络设置组件
 *
 * 数据源：Nm3u8dlConfig.network (NetworkConfig)
 * 更新：emit DeepPartial<Nm3u8dlConfig>
 */

import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import { AppIcon } from "@/components/common";
import { SettingSwitch, SettingInput, SettingsGroup } from "..";
import type { NetworkConfig, NetworkHeader, Nm3u8dlConfig } from "@/domain";
import type { DeepPartial } from "@/services";

interface Props {
  network: NetworkConfig;
}

const props = defineProps<Props>();
const { t } = useI18n();

const emit = defineEmits<{
  (e: "update", value: DeepPartial<Nm3u8dlConfig>): void;
}>();

// 请求头列表
const headers = computed(() => props.network.headers);

// 下一个 header id
const nextHeaderId = computed(() => {
  const maxId = headers.value.reduce((max, h) => Math.max(max, h.id), 0);
  return maxId + 1;
});

// 补丁网络配置
function patchNetwork(patch: Partial<NetworkConfig>) {
  emit("update", { network: patch });
}

// 添加请求头
function addHeader() {
  const newHeader: NetworkHeader = {
    id: nextHeaderId.value,
    name: "",
    value: "",
    enabled: true,
    sort_order: headers.value.length,
  };
  patchNetwork({ headers: [...headers.value, newHeader] });
}

// 删除请求头
function removeHeader(index: number) {
  const newHeaders = headers.value.filter((_, i) => i !== index);
  patchNetwork({ headers: newHeaders });
}

// 更新请求头 Name
function updateHeaderName(index: number, name: string) {
  const newHeaders = [...headers.value];
  const header = newHeaders[index];
  if (header) {
    newHeaders[index] = { ...header, name };
  }
  patchNetwork({ headers: newHeaders });
}

// 更新请求头 Value
function updateHeaderValue(index: number, value: string) {
  const newHeaders = [...headers.value];
  const header = newHeaders[index];
  if (header) {
    newHeaders[index] = { ...header, value };
  }
  patchNetwork({ headers: newHeaders });
}

// 切换启用状态
function toggleHeader(index: number, enabled: boolean) {
  const newHeaders = [...headers.value];
  const header = newHeaders[index];
  if (header) {
    newHeaders[index] = { ...header, enabled };
  }
  patchNetwork({ headers: newHeaders });
}
</script>

<template>
  <SettingsGroup
    :title="t('settings.network.proxy')"
    :description="t('settings.network.proxyDesc', '配置网络代理选项')"
  >
    <SettingSwitch
      :model-value="network.use_system_proxy"
      :label="t('settings.network.useSystemProxy')"
      :description="
        t('settings.network.useSystemProxyDesc', '自动使用系统配置的代理')
      "
      @update:model-value="patchNetwork({ use_system_proxy: $event })"
    />

    <SettingInput
      :model-value="network.custom_proxy || ''"
      :label="t('settings.network.customProxy')"
      placeholder="http://127.0.0.1:7890"
      @update:model-value="
        patchNetwork({ custom_proxy: String($event) || null })
      "
    />

    <SettingInput
      :model-value="network.base_url || ''"
      :label="t('settings.network.baseUrl')"
      placeholder="替换 URL 的基础部分"
      @update:model-value="patchNetwork({ base_url: String($event) || null })"
    />

    <SettingSwitch
      :model-value="network.append_url_params"
      :label="t('settings.network.appendUrlParams', '附加 URL 参数')"
      :description="
        t(
          'settings.network.appendUrlParamsDesc',
          '将原始 URL 的查询参数附加到所有请求',
        )
      "
      @update:model-value="patchNetwork({ append_url_params: $event })"
    />
  </SettingsGroup>

  <SettingsGroup
    :title="t('settings.network.headers')"
    :description="
      t(
        'settings.network.headersDesc',
        '配置 HTTP 请求头（如 Referer、User-Agent、Cookie）',
      )
    "
  >
    <div
      v-if="headers.length === 0"
      class="text-sm py-2"
      style="color: var(--text-secondary)"
    >
      {{
        t("settings.network.noHeaders", "暂无自定义请求头，点击下方按钮添加")
      }}
    </div>

    <div v-else class="space-y-2">
      <div
        v-for="(header, index) in headers"
        :key="header.id"
        class="flex items-center gap-2"
      >
        <!-- Name 输入 -->
        <input
          :value="header.name"
          type="text"
          placeholder="Header Name"
          class="w-36 h-9 px-3 text-sm rounded-md border bg-transparent focus:outline-none focus:ring-2 focus:ring-ring"
          style="border-color: rgba(255, 255, 255, 0.08)"
          @input="
            updateHeaderName(index, ($event.target as HTMLInputElement).value)
          "
        />
        <!-- Value 输入 -->
        <input
          :value="header.value"
          type="text"
          placeholder="Header Value"
          class="flex-1 h-9 px-3 text-sm rounded-md border bg-transparent focus:outline-none focus:ring-2 focus:ring-ring"
          style="border-color: rgba(255, 255, 255, 0.08)"
          @input="
            updateHeaderValue(index, ($event.target as HTMLInputElement).value)
          "
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
          class="h-9 w-9 cursor-pointer"
          style="color: var(--accent-error)"
          @click="removeHeader(index)"
        >
          <AppIcon name="Trash2" :size="16" />
        </Button>
      </div>
    </div>

    <Button
      variant="outline"
      size="sm"
      class="mt-2 cursor-pointer"
      @click="addHeader"
    >
      <AppIcon name="Plus" :size="14" class="mr-1" />
      {{ t("settings.network.addHeader", "添加请求头") }}
    </Button>
  </SettingsGroup>
</template>
