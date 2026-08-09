<script setup lang="ts">
/**
 * MuxImportManager - 混流导入的外部媒体文件列表
 *
 * 混流时把外部音视频/字幕文件一起封装进输出（N_m3u8DL-RE `--mux-import`）。
 * 纯受控组件：数据源 props.imports，更新 emit 新数组，由父级 patch。
 */

import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import { AppIcon } from "@/components/common";
import { SettingsGroup } from ".";
import type { MuxImport } from "@/domain";

interface Props {
  imports: MuxImport[];
}

const props = defineProps<Props>();
const { t } = useI18n();

const emit = defineEmits<{
  (e: "update", value: MuxImport[]): void;
}>();

// 下一个文件 id
const nextId = computed(() => {
  const maxId = props.imports.reduce((max, i) => Math.max(max, i.id), 0);
  return maxId + 1;
});

// 添加导入文件
function addImport() {
  emit("update", [
    ...props.imports,
    {
      id: nextId.value,
      path: "",
      lang: null,
      name: null,
      enabled: true,
      sort_order: props.imports.length,
    },
  ]);
}

// 删除导入文件
function removeImport(index: number) {
  emit(
    "update",
    props.imports.filter((_, i) => i !== index),
  );
}

// 局部更新某个导入文件
function patchImport(index: number, patch: Partial<MuxImport>) {
  const next = [...props.imports];
  const item = next[index];
  if (item) {
    next[index] = { ...item, ...patch };
  }
  emit("update", next);
}
</script>

<template>
  <SettingsGroup
    :title="t('settings.download.muxImport', '混流导入')"
    :description="
      t('settings.download.muxImportDesc', '混流时导入外部音视频/字幕文件')
    "
  >
    <div
      v-if="imports.length === 0"
      class="px-5 py-4 text-sm text-muted-foreground"
    >
      {{
        t(
          "settings.download.noMuxImport",
          "暂无导入文件，点击下方按钮添加外部字幕/音轨",
        )
      }}
    </div>

    <div v-else class="space-y-2 px-5 py-4">
      <div
        v-for="(imp, index) in imports"
        :key="imp.id"
        class="flex items-center gap-2"
      >
        <input
          :value="imp.path"
          type="text"
          :placeholder="t('settings.download.muxImportPath', '文件路径')"
          class="h-9 w-44 rounded-md border border-border/60 bg-transparent px-3 text-sm focus:outline-none focus:ring-2 focus:ring-ring"
          @input="
            patchImport(index, {
              path: ($event.target as HTMLInputElement).value,
            })
          "
        />
        <input
          :value="imp.lang || ''"
          type="text"
          :placeholder="t('settings.download.muxImportLang', '语言代码 (chi)')"
          class="h-9 w-28 rounded-md border border-border/60 bg-transparent px-3 text-sm focus:outline-none focus:ring-2 focus:ring-ring"
          @input="
            patchImport(index, {
              lang: ($event.target as HTMLInputElement).value || null,
            })
          "
        />
        <input
          :value="imp.name || ''"
          type="text"
          :placeholder="t('settings.download.muxImportName', '描述')"
          class="h-9 flex-1 rounded-md border border-border/60 bg-transparent px-3 text-sm focus:outline-none focus:ring-2 focus:ring-ring"
          @input="
            patchImport(index, {
              name: ($event.target as HTMLInputElement).value || null,
            })
          "
        />
        <Switch
          :model-value="imp.enabled"
          @update:model-value="patchImport(index, { enabled: $event })"
        />
        <Button
          variant="ghost"
          size="icon"
          class="h-9 w-9 cursor-pointer text-destructive"
          @click="removeImport(index)"
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
        @click="addImport"
      >
        <AppIcon name="Plus" :size="14" class="mr-1" />
        {{ t("settings.download.addMuxImport", "添加导入文件") }}
      </Button>
    </div>
  </SettingsGroup>
</template>
