<script setup lang="ts">
/**
 * AdKeywordManager - 广告关键词过滤列表
 *
 * 过滤分片 URL 中包含指定正则的广告内容（N_m3u8DL-RE `--ad-keyword`）。
 * 纯受控组件：数据源 props.keywords，更新 emit 新数组，由父级 patch。
 */

import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import { AppIcon } from "@/components/common";
import { SettingsGroup } from ".";
import type { AdKeyword } from "@/domain";

interface Props {
  keywords: AdKeyword[];
}

const props = defineProps<Props>();
const { t } = useI18n();

const emit = defineEmits<{
  (e: "update", value: AdKeyword[]): void;
}>();

// 下一个关键词 id
const nextId = computed(() => {
  const maxId = props.keywords.reduce((max, k) => Math.max(max, k.id), 0);
  return maxId + 1;
});

// 添加关键词
function addKeyword() {
  emit("update", [
    ...props.keywords,
    {
      id: nextId.value,
      keyword: "",
      enabled: true,
      sort_order: props.keywords.length,
    },
  ]);
}

// 删除关键词
function removeKeyword(index: number) {
  emit(
    "update",
    props.keywords.filter((_, i) => i !== index),
  );
}

// 更新关键词文本
function updateKeyword(index: number, keyword: string) {
  const next = [...props.keywords];
  const item = next[index];
  if (item) {
    next[index] = { ...item, keyword };
  }
  emit("update", next);
}

// 切换启用状态
function toggleKeyword(index: number, enabled: boolean) {
  const next = [...props.keywords];
  const item = next[index];
  if (item) {
    next[index] = { ...item, enabled };
  }
  emit("update", next);
}
</script>

<template>
  <SettingsGroup
    :title="t('settings.download.adFilter', '广告过滤')"
    :description="
      t('settings.download.adFilterDesc', '过滤包含指定关键字的分片')
    "
  >
    <div
      v-if="keywords.length === 0"
      class="px-5 py-4 text-sm text-muted-foreground"
    >
      {{
        t("settings.download.noAdKeywords", "暂无广告关键词，点击下方按钮添加")
      }}
    </div>

    <div v-else class="space-y-2 px-5 py-4">
      <div
        v-for="(kw, index) in keywords"
        :key="kw.id"
        class="flex items-center gap-2"
      >
        <input
          :value="kw.keyword"
          type="text"
          :placeholder="
            t(
              'settings.download.adKeywordPlaceholder',
              '例如: ad_iframe|dummy\\.ts',
            )
          "
          class="h-9 flex-1 rounded-md border border-border/60 bg-transparent px-3 text-sm focus:outline-none focus:ring-2 focus:ring-ring"
          @input="
            updateKeyword(index, ($event.target as HTMLInputElement).value)
          "
        />
        <Switch
          :model-value="kw.enabled"
          @update:model-value="toggleKeyword(index, $event)"
        />
        <Button
          variant="ghost"
          size="icon"
          class="h-9 w-9 cursor-pointer text-destructive"
          @click="removeKeyword(index)"
        >
          <AppIcon name="Trash2" :size="16" />
        </Button>
      </div>
    </div>

    <div class="px-5 py-4">
      <Button
        variant="outline"
        size="sm"
        class="cursor-pointer"
        @click="addKeyword"
      >
        <AppIcon name="Plus" :size="14" class="mr-1" />
        {{ t("settings.download.addAdKeyword", "添加关键词") }}
      </Button>
    </div>
  </SettingsGroup>
</template>
