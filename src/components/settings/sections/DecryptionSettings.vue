<script setup lang="ts">
/**
 * DecryptionSettings - 解密设置 UI 组件
 */

import { computed } from "vue";
import { Separator } from "@/components/ui/separator";
import { Button } from "@/components/ui/button";
import { AppIcon } from "@/components/common";
import { SettingSelect, SettingInput, SettingSwitch, SettingsGroup } from "..";
import type { LegacyDecryptionSettings, KeyConfig } from "@/types";

interface Props {
  settings: { decryption: LegacyDecryptionSettings };
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: "update:settings", value: Partial<LegacyDecryptionSettings>): void;
}>();

// 解密引擎选项
const decryptionEngineOptions = [
  { value: "FFMPEG", label: "FFmpeg" },
  { value: "MP4DECRYPT", label: "MP4Decrypt" },
  { value: "SHAKA_PACKAGER", label: "Shaka Packager" },
];

// HLS 加密方法选项
const hlsMethodOptions = [
  { value: "UNKNOWN", label: "自动检测" },
  { value: "AES_128", label: "AES-128 CBC" },
  { value: "AES_128_ECB", label: "AES-128 ECB" },
  { value: "CENC", label: "CENC" },
  { value: "CHACHA20", label: "ChaCha20" },
  { value: "SAMPLE_AES", label: "Sample AES" },
  { value: "SAMPLE_AES_CTR", label: "Sample AES CTR" },
  { value: "NONE", label: "无加密" },
];

// 密钥列表
const keys = computed(() => props.settings.decryption.keys);

// 自定义 HLS
const customHls = computed(() => props.settings.decryption.customHls);

// 更新设置
const updateDecryption = (value: Partial<LegacyDecryptionSettings>) => {
  emit("update:settings", value);
};

// 添加密钥
const addKey = () => {
  const newKeys: KeyConfig[] = [...keys.value, { key: "" }];
  updateDecryption({ keys: newKeys });
};

// 删除密钥
const removeKey = (index: number) => {
  const newKeys = keys.value.filter((_, i) => i !== index);
  updateDecryption({ keys: newKeys });
};

// 更新密钥 KID
const updateKeyId = (index: number, kid: string) => {
  const newKeys = [...keys.value];
  if (newKeys[index]) {
    newKeys[index] = { ...newKeys[index], kid };
  }
  updateDecryption({ keys: newKeys });
};

// 更新密钥值
const updateKeyValue = (index: number, key: string) => {
  const newKeys = [...keys.value];
  if (newKeys[index]) {
    newKeys[index] = { ...newKeys[index], key };
  }
  updateDecryption({ keys: newKeys });
};

// 更新自定义 HLS 设置
const updateCustomHls = (
  value: Partial<LegacyDecryptionSettings["customHls"]>,
) => {
  updateDecryption({
    customHls: { ...props.settings.decryption.customHls, ...value },
  });
};
</script>

<template>
  <div class="space-y-2">
    <SettingsGroup title="解密引擎" description="配置 DRM 解密相关选项">
      <SettingSelect
        :model-value="settings.decryption.engine"
        label="解密引擎"
        :options="decryptionEngineOptions"
        placeholder="选择引擎"
        @update:model-value="updateDecryption({ engine: $event as any })"
      />

      <SettingInput
        :model-value="settings.decryption.binPath"
        label="解密器路径"
        placeholder="留空则使用系统 PATH"
        @update:model-value="updateDecryption({ binPath: String($event) })"
      />

      <SettingInput
        :model-value="settings.decryption.keyTextFile"
        label="密钥文本文件"
        placeholder="包含密钥的文本文件路径"
        @update:model-value="updateDecryption({ keyTextFile: String($event) })"
      />

      <SettingSwitch
        :model-value="settings.decryption.realTimeDecryption"
        label="实时解密"
        description="下载时实时解密分片"
        @update:model-value="updateDecryption({ realTimeDecryption: $event })"
      />
    </SettingsGroup>

    <SettingsGroup
      title="密钥配置"
      description="手动添加 DRM 解密密钥 (KID:KEY 格式)"
    >
      <div v-if="keys.length === 0" class="text-sm text-muted-foreground py-2">
        暂无密钥配置，点击下方按钮添加
      </div>

      <div v-else class="space-y-2">
        <div
          v-for="(keyConfig, index) in keys"
          :key="index"
          class="flex items-center gap-2"
        >
          <input
            :value="keyConfig.kid || ''"
            type="text"
            placeholder="KID (可选)"
            class="w-40 h-9 px-3 text-sm rounded-md border border-input bg-transparent focus:outline-none focus:ring-2 focus:ring-ring"
            @input="
              updateKeyId(index, ($event.target as HTMLInputElement).value)
            "
          />
          <input
            :value="keyConfig.key"
            type="text"
            placeholder="Key (十六进制)"
            class="flex-1 h-9 px-3 text-sm rounded-md border border-input bg-transparent focus:outline-none focus:ring-2 focus:ring-ring"
            @input="
              updateKeyValue(index, ($event.target as HTMLInputElement).value)
            "
          />
          <Button
            variant="ghost"
            size="icon"
            class="h-9 w-9 text-destructive hover:text-destructive"
            @click="removeKey(index)"
          >
            <AppIcon name="Trash2" :size="16" />
          </Button>
        </div>
      </div>

      <Button variant="outline" size="sm" class="mt-2" @click="addKey">
        <AppIcon name="Plus" :size="14" class="mr-1" />
        添加密钥
      </Button>
    </SettingsGroup>

    <SettingsGroup
      title="HLS 自定义解密"
      description="用于特殊 HLS 加密流的解密配置"
    >
      <SettingSwitch
        :model-value="customHls.enabled"
        label="启用自定义 HLS 解密"
        description="使用自定义参数解密 HLS 流"
        @update:model-value="updateCustomHls({ enabled: $event })"
      />

      <template v-if="customHls.enabled">
        <Separator class="my-4" />

        <SettingSelect
          :model-value="customHls.method"
          label="加密方法"
          :options="hlsMethodOptions"
          @update:model-value="updateCustomHls({ method: $event as any })"
        />

        <SettingInput
          :model-value="customHls.key.value"
          label="密钥 (Key)"
          placeholder="十六进制或 Base64 格式"
          @update:model-value="
            updateCustomHls({
              key: { ...customHls.key, value: String($event) },
            })
          "
        />

        <SettingInput
          :model-value="customHls.iv?.value || ''"
          label="初始化向量 (IV)"
          placeholder="可选，十六进制或 Base64 格式"
          @update:model-value="
            updateCustomHls({ iv: { type: 'hex', value: String($event) } })
          "
        />
      </template>
    </SettingsGroup>
  </div>
</template>
