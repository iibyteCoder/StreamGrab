<script setup lang="ts">
/**
 * DecryptionSettings - 解密设置 UI 组件
 *
 * 数据源：Nm3u8dlConfig.decryption (DecryptionConfig)
 * 更新：emit DeepPartial<Nm3u8dlConfig>
 */

import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { Separator } from "@/components/ui/separator";
import { Button } from "@/components/ui/button";
import { AppIcon } from "@/components/common";
import { SettingSelect, SettingInput, SettingSwitch, SettingsGroup } from "..";
import type {
  DecryptionConfig,
  DecryptionKey,
  DecryptionEngine,
  HlsEncryptionMethod,
  CustomHlsConfig,
  Nm3u8dlConfig,
} from "@/domain";
import type { DeepPartial } from "@/services";

interface Props {
  decryption: DecryptionConfig;
}

const props = defineProps<Props>();
const { t } = useI18n();

const emit = defineEmits<{
  (e: "update", value: DeepPartial<Nm3u8dlConfig>): void;
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
const keys = computed(() => props.decryption.keys);

// 自定义 HLS
const customHls = computed(() => props.decryption.custom_hls);

// 下一个 key id
const nextKeyId = computed(() => {
  const maxId = keys.value.reduce((max, k) => Math.max(max, k.id), 0);
  return maxId + 1;
});

// 补丁解密配置
function patchDecryption(patch: Partial<DecryptionConfig>) {
  emit("update", { decryption: patch });
}

// 添加密钥
function addKey() {
  const newKey: DecryptionKey = {
    id: nextKeyId.value,
    kid: null,
    key: "",
    sort_order: keys.value.length,
  };
  patchDecryption({ keys: [...keys.value, newKey] });
}

// 删除密钥
function removeKey(index: number) {
  const newKeys = keys.value.filter((_, i) => i !== index);
  patchDecryption({ keys: newKeys });
}

// 更新密钥 KID
function updateKeyKid(index: number, kid: string) {
  const newKeys = [...keys.value];
  if (newKeys[index]) {
    newKeys[index] = { ...newKeys[index], kid: kid || null };
  }
  patchDecryption({ keys: newKeys });
}

// 更新密钥值
function updateKeyValue(index: number, key: string) {
  const newKeys = [...keys.value];
  if (newKeys[index]) {
    newKeys[index] = { ...newKeys[index], key };
  }
  patchDecryption({ keys: newKeys });
}

// 更新自定义 HLS 设置
function updateCustomHls(patch: Partial<CustomHlsConfig>) {
  patchDecryption({
    custom_hls: { ...props.decryption.custom_hls, ...patch },
  });
}
</script>

<template>
  <SettingsGroup
    :title="t('settings.decryption.engine', '解密引擎')"
    :description="t('settings.decryption.engineDesc', '配置 DRM 解密相关选项')"
  >
    <SettingSelect
      :model-value="decryption.engine"
      :label="t('settings.decryption.engine', '解密引擎')"
      :options="decryptionEngineOptions"
      placeholder="选择引擎"
      @update:model-value="
        patchDecryption({ engine: $event as DecryptionEngine })
      "
    />

    <SettingInput
      :model-value="decryption.bin_path || ''"
      :label="t('settings.decryption.binPath', '解密器路径')"
      placeholder="留空则使用系统 PATH"
      @update:model-value="
        patchDecryption({ bin_path: String($event) || null })
      "
    />

    <SettingInput
      :model-value="decryption.key_text_file || ''"
      :label="t('settings.decryption.keyFile')"
      placeholder="包含密钥的文本文件路径"
      @update:model-value="
        patchDecryption({ key_text_file: String($event) || null })
      "
    />

    <SettingSwitch
      :model-value="decryption.real_time_decryption"
      :label="t('settings.decryption.realTimeDecryption')"
      :description="
        t('settings.decryption.realTimeDecryptionDesc', '下载时实时解密分片')
      "
      @update:model-value="patchDecryption({ real_time_decryption: $event })"
    />
  </SettingsGroup>

  <SettingsGroup
    :title="t('settings.decryption.keys')"
    :description="
      t('settings.decryption.keysDesc', '手动添加 DRM 解密密钥 (KID:KEY 格式)')
    "
  >
    <div
      v-if="keys.length === 0"
      class="text-sm py-2"
      style="color: var(--text-secondary)"
    >
      {{ t("settings.decryption.noKeys", "暂无密钥配置，点击下方按钮添加") }}
    </div>

    <div v-else class="space-y-2">
      <div
        v-for="(keyConfig, index) in keys"
        :key="keyConfig.id"
        class="flex items-center gap-2"
      >
        <input
          :value="keyConfig.kid || ''"
          type="text"
          placeholder="KID (可选)"
          class="w-40 h-9 px-3 text-sm rounded-md border bg-transparent focus:outline-none focus:ring-2 focus:ring-ring"
          style="border-color: rgba(255, 255, 255, 0.08)"
          @input="
            updateKeyKid(index, ($event.target as HTMLInputElement).value)
          "
        />
        <input
          :value="keyConfig.key"
          type="text"
          placeholder="Key (十六进制)"
          class="flex-1 h-9 px-3 text-sm rounded-md border bg-transparent focus:outline-none focus:ring-2 focus:ring-ring"
          style="border-color: rgba(255, 255, 255, 0.08)"
          @input="
            updateKeyValue(index, ($event.target as HTMLInputElement).value)
          "
        />
        <Button
          variant="ghost"
          size="icon"
          class="h-9 w-9 cursor-pointer"
          style="color: var(--accent-error)"
          @click="removeKey(index)"
        >
          <AppIcon name="Trash2" :size="16" />
        </Button>
      </div>
    </div>

    <Button
      variant="outline"
      size="sm"
      class="mt-2 cursor-pointer"
      @click="addKey"
    >
      <AppIcon name="Plus" :size="14" class="mr-1" />
      {{ t("settings.decryption.addKey", "添加密钥") }}
    </Button>
  </SettingsGroup>

  <SettingsGroup
    :title="t('settings.decryption.hlsCustomMethod')"
    :description="
      t(
        'settings.decryption.hlsCustomMethodDesc',
        '用于特殊 HLS 加密流的解密配置',
      )
    "
  >
    <SettingSwitch
      :model-value="customHls.enabled"
      :label="
        t('settings.decryption.hlsCustomMethodEnabled', '启用自定义 HLS 解密')
      "
      :description="
        t(
          'settings.decryption.hlsCustomMethodEnabledDesc',
          '使用自定义参数解密 HLS 流',
        )
      "
      @update:model-value="updateCustomHls({ enabled: $event })"
    />

    <template v-if="customHls.enabled">
      <Separator class="my-4" />

      <SettingSelect
        :model-value="customHls.method"
        label="加密方法"
        :options="hlsMethodOptions"
        @update:model-value="
          updateCustomHls({ method: $event as HlsEncryptionMethod })
        "
      />

      <SettingInput
        :model-value="customHls.key_value || ''"
        label="密钥 (Key)"
        placeholder="十六进制或 Base64 格式"
        @update:model-value="
          updateCustomHls({ key_value: String($event) || null })
        "
      />

      <SettingInput
        :model-value="customHls.iv_value || ''"
        label="初始化向量 (IV)"
        placeholder="可选，十六进制或 Base64 格式"
        @update:model-value="
          updateCustomHls({ iv_value: String($event) || null })
        "
      />
    </template>
  </SettingsGroup>
</template>
