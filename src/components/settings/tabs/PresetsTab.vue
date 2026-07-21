<script setup lang="ts">
/**
 * PresetsTab - 任务预设标签页
 *
 * 取代旧 TemplateManager，管理 TaskPreset（命名的 TaskOverrides 组合）。
 * 业务逻辑：usePresetManager composable
 */

import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import { AppIcon } from "@/components/common";
import { usePresetManager } from "@/composables";
import type { TaskOverrides, MuxFormat, SubtitleFormat } from "@/domain";

const { t } = useI18n();
const manager = usePresetManager();

// ========================================
// 预设列表
// ========================================

const presets = computed(() => manager.presets.value);

// 常用图标名列表供选择
const iconOptions = [
  { value: "Star", label: "Star" },
  { value: "Heart", label: "Heart" },
  { value: "Zap", label: "Zap" },
  { value: "Crown", label: "Crown" },
  { value: "Flame", label: "Flame" },
  { value: "Rocket", label: "Rocket" },
  { value: "Gem", label: "Gem" },
  { value: "Shield", label: "Shield" },
  { value: "Target", label: "Target" },
  { value: "Award", label: "Award" },
  { value: "Bookmark", label: "Bookmark" },
  { value: "Film", label: "Film" },
  { value: "Music", label: "Music" },
  { value: "Radio", label: "Radio" },
  { value: "Tv", label: "TV" },
  { value: "Globe", label: "Globe" },
];

// 混流格式选项
const muxFormatOptions = [
  { value: "mp4", label: "MP4" },
  { value: "mkv", label: "MKV" },
];

// 字幕格式选项
const subFormatOptions = [
  { value: "SRT", label: "SRT" },
  { value: "VTT", label: "WebVTT" },
];

// ========================================
// 覆盖项摘要
// ========================================

function getOverridesSummary(overrides: TaskOverrides): string {
  const parts: string[] = [];
  if (overrides.saveDir)
    parts.push(
      `${t("settings.preset.summaryDir", "目录")}: ${overrides.saveDir}`,
    );
  if (overrides.saveName)
    parts.push(
      `${t("settings.preset.summaryName", "名称")}: ${overrides.saveName}`,
    );
  if (overrides.muxFormat)
    parts.push(
      `${t("settings.preset.summaryFormat", "格式")}: ${overrides.muxFormat}`,
    );
  if (overrides.maxSpeed)
    parts.push(
      `${t("settings.preset.summarySpeed", "限速")}: ${overrides.maxSpeed}`,
    );
  if (overrides.subtitleFormat)
    parts.push(
      `${t("settings.preset.summarySub", "字幕")}: ${overrides.subtitleFormat}`,
    );
  if (overrides.subtitlesOnly)
    parts.push(t("settings.preset.summarySubOnly", "仅字幕"));
  if (overrides.key) parts.push(t("settings.preset.summaryKey", "含密钥"));
  return parts.length > 0
    ? parts.join(" · ")
    : t("settings.preset.noOverrides", "无覆盖项");
}

// ========================================
// 新建预设
// ========================================

function handleCreate() {
  manager.createFromOverrides({});
}

// ========================================
// 对话框表单辅助
// ========================================

function updateOverride<K extends keyof TaskOverrides>(
  key: K,
  value: TaskOverrides[K],
) {
  manager.editForm.value.overrides = {
    ...manager.editForm.value.overrides,
    [key]: value,
  };
}

function handleSave() {
  manager.savePreset();
}
</script>

<template>
  <div class="space-y-6">
    <!-- 标题和操作按钮 -->
    <div class="flex items-center justify-between">
      <div>
        <h3 class="text-sm font-medium" style="color: var(--text-primary)">
          {{ t("settings.preset.title", "任务预设") }}
        </h3>
        <p class="text-xs mt-1" style="color: var(--text-secondary)">
          {{
            t(
              "settings.preset.subtitle",
              "保存常用的下载配置组合，快速应用到新任务",
            )
          }}
        </p>
      </div>
      <Button
        variant="default"
        size="sm"
        class="cursor-pointer"
        @click="handleCreate"
      >
        <AppIcon name="Plus" :size="14" class="mr-1.5" />
        {{ t("settings.preset.create", "新建预设") }}
      </Button>
    </div>

    <!-- 预设列表 -->
    <div
      v-if="presets.length === 0"
      class="text-center py-12"
      style="color: var(--text-secondary)"
    >
      <AppIcon name="FileBox" :size="40" class="mx-auto mb-3 opacity-50" />
      <p>{{ t("settings.preset.noPresets", "暂无预设") }}</p>
      <p class="text-xs mt-1">
        {{ t("settings.preset.noPresetsHint", "点击上方按钮创建第一个预设") }}
      </p>
    </div>

    <div v-else class="grid gap-3 sm:grid-cols-2">
      <div
        v-for="preset in presets"
        :key="preset.id"
        class="group relative rounded-xl p-4"
        style="
          background: var(--bg-surface);
          border: 1px solid rgba(255, 255, 255, 0.08);
          transition: border-color 150ms ease-out;
        "
        @mouseenter="
          ($event.currentTarget as HTMLElement).style.borderColor =
            'rgba(255, 255, 255, 0.16)'
        "
        @mouseleave="
          ($event.currentTarget as HTMLElement).style.borderColor =
            'rgba(255, 255, 255, 0.08)'
        "
      >
        <div class="flex items-start justify-between mb-2">
          <div class="flex items-center gap-2">
            <AppIcon
              :name="
                (preset.icon as keyof typeof import('lucide-vue-next')) ||
                'Bookmark'
              "
              :size="16"
              style="color: var(--accent-primary)"
            />
            <span
              class="text-sm font-medium"
              style="color: var(--text-primary)"
            >
              {{ preset.name }}
            </span>
          </div>
          <div class="flex items-center gap-1">
            <Button
              variant="ghost"
              size="icon"
              class="h-7 w-7 cursor-pointer opacity-0 group-hover:opacity-100"
              style="transition: opacity 150ms ease-out"
              :title="t('common.edit', '编辑')"
              @click="manager.editPreset(preset)"
            >
              <AppIcon name="Pencil" :size="14" />
            </Button>
            <Button
              variant="ghost"
              size="icon"
              class="h-7 w-7 cursor-pointer opacity-0 group-hover:opacity-100"
              style="transition: opacity 150ms ease-out"
              :title="t('common.copy', '复制')"
              @click="manager.duplicatePreset(preset)"
            >
              <AppIcon name="Copy" :size="14" />
            </Button>
            <Button
              variant="ghost"
              size="icon"
              class="h-7 w-7 cursor-pointer opacity-0 group-hover:opacity-100"
              style="
                transition: opacity 150ms ease-out;
                color: var(--accent-error);
              "
              :title="t('common.delete', '删除')"
              @click="manager.confirmDelete(preset)"
            >
              <AppIcon name="Trash2" :size="14" />
            </Button>
          </div>
        </div>

        <p
          v-if="preset.description"
          class="text-xs mb-2 line-clamp-2"
          style="color: var(--text-secondary)"
        >
          {{ preset.description }}
        </p>

        <p class="text-xs" style="color: var(--text-secondary); opacity: 0.7">
          {{ getOverridesSummary(preset.overrides) }}
        </p>
      </div>
    </div>

    <!-- 编辑/新建对话框 -->
    <Dialog
      :open="manager.showEditDialog.value"
      @update:open="manager.closeEditDialog()"
    >
      <DialogContent class="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>
            {{
              manager.editingPreset.value
                ? t("settings.preset.editPreset", "编辑预设")
                : t("settings.preset.createPreset", "新建预设")
            }}
          </DialogTitle>
        </DialogHeader>

        <div class="space-y-4 py-4">
          <!-- 基本信息 -->
          <div class="grid grid-cols-3 gap-3">
            <div class="col-span-2 space-y-1.5">
              <Label>{{ t("settings.preset.name", "预设名称") }}</Label>
              <Input
                v-model="manager.editForm.value.name"
                :placeholder="
                  t('settings.preset.namePlaceholder', '例如：B站 1080P')
                "
              />
            </div>
            <div class="space-y-1.5">
              <Label>{{ t("settings.preset.icon", "图标") }}</Label>
              <Select v-model="manager.editForm.value.icon">
                <SelectTrigger>
                  <SelectValue
                    :placeholder="t('settings.preset.selectIcon', '选择图标')"
                  />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem
                    v-for="icon in iconOptions"
                    :key="icon.value"
                    :value="icon.value"
                  >
                    <div class="flex items-center gap-2">
                      <AppIcon :name="icon.value as any" :size="14" />
                      <span>{{ icon.label }}</span>
                    </div>
                  </SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>

          <div class="space-y-1.5">
            <Label>{{ t("settings.preset.description", "描述") }}</Label>
            <Input
              v-model="manager.editForm.value.description"
              :placeholder="
                t(
                  'settings.preset.descriptionPlaceholder',
                  '可选，说明预设用途',
                )
              "
            />
          </div>

          <Separator />

          <!-- 覆盖项 -->
          <div class="space-y-3">
            <Label class="text-sm font-medium">{{
              t("settings.preset.overrides", "覆盖项（留空则沿用全局默认）")
            }}</Label>

            <div class="grid grid-cols-2 gap-3">
              <div class="space-y-1.5">
                <Label class="text-xs">{{
                  t("settings.general.saveDir")
                }}</Label>
                <Input
                  :model-value="manager.editForm.value.overrides.saveDir || ''"
                  :placeholder="
                    t('settings.preset.leaveEmptyDefault', '留空沿用默认')
                  "
                  @update:model-value="
                    updateOverride('saveDir', String($event) || null)
                  "
                />
              </div>
              <div class="space-y-1.5">
                <Label class="text-xs">{{
                  t("settings.preset.saveName", "保存名称")
                }}</Label>
                <Input
                  :model-value="manager.editForm.value.overrides.saveName || ''"
                  :placeholder="
                    t('settings.preset.leaveEmptyDefault', '留空使用默认')
                  "
                  @update:model-value="
                    updateOverride('saveName', String($event) || null)
                  "
                />
              </div>
            </div>

            <div class="grid grid-cols-2 gap-3">
              <div class="space-y-1.5">
                <Label class="text-xs">{{ t("settings.mux.format") }}</Label>
                <Select
                  :model-value="
                    manager.editForm.value.overrides.muxFormat || ''
                  "
                  @update:model-value="
                    updateOverride('muxFormat', ($event as MuxFormat) || null)
                  "
                >
                  <SelectTrigger>
                    <SelectValue
                      :placeholder="
                        t('settings.preset.followDefault', '沿用默认')
                      "
                    />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="">{{
                      t("settings.preset.followDefault", "沿用默认")
                    }}</SelectItem>
                    <SelectItem
                      v-for="opt in muxFormatOptions"
                      :key="opt.value"
                      :value="opt.value"
                    >
                      {{ opt.label }}
                    </SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div class="space-y-1.5">
                <Label class="text-xs">{{
                  t("settings.download.subtitleFormat")
                }}</Label>
                <Select
                  :model-value="
                    manager.editForm.value.overrides.subtitleFormat || ''
                  "
                  @update:model-value="
                    updateOverride(
                      'subtitleFormat',
                      ($event as SubtitleFormat) || null,
                    )
                  "
                >
                  <SelectTrigger>
                    <SelectValue
                      :placeholder="
                        t('settings.preset.followDefault', '沿用默认')
                      "
                    />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="">{{
                      t("settings.preset.followDefault", "沿用默认")
                    }}</SelectItem>
                    <SelectItem
                      v-for="opt in subFormatOptions"
                      :key="opt.value"
                      :value="opt.value"
                    >
                      {{ opt.label }}
                    </SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>

            <div class="grid grid-cols-2 gap-3">
              <div class="space-y-1.5">
                <Label class="text-xs">{{
                  t("settings.download.maxSpeed")
                }}</Label>
                <Input
                  :model-value="manager.editForm.value.overrides.maxSpeed || ''"
                  :placeholder="
                    t('settings.preset.leaveEmptyUnlimited', '留空不限')
                  "
                  @update:model-value="
                    updateOverride('maxSpeed', String($event) || null)
                  "
                />
              </div>
              <div class="space-y-1.5">
                <Label class="text-xs">{{
                  t("settings.preset.customRange", "自定义范围")
                }}</Label>
                <Input
                  :model-value="
                    manager.editForm.value.overrides.customRange || ''
                  "
                  :placeholder="
                    t(
                      'settings.preset.rangePlaceholder',
                      '例如: 0:00:00-0:05:00',
                    )
                  "
                  @update:model-value="
                    updateOverride('customRange', String($event) || null)
                  "
                />
              </div>
            </div>

            <div class="grid grid-cols-2 gap-3">
              <div class="space-y-1.5">
                <Label class="text-xs">{{
                  t("settings.preset.key", "解密密钥")
                }}</Label>
                <Input
                  :model-value="manager.editForm.value.overrides.key || ''"
                  placeholder="KID:KEY 格式"
                  @update:model-value="
                    updateOverride('key', String($event) || null)
                  "
                />
              </div>
              <div class="flex items-end gap-3 pb-1">
                <div class="flex items-center gap-2">
                  <Switch
                    :checked="
                      manager.editForm.value.overrides.subtitlesOnly || false
                    "
                    @update:checked="updateOverride('subtitlesOnly', $event)"
                  />
                  <Label class="text-xs">{{
                    t("settings.download.downloadSubtitleOnly")
                  }}</Label>
                </div>
              </div>
            </div>
          </div>
        </div>

        <DialogFooter>
          <Button
            variant="outline"
            class="cursor-pointer"
            @click="manager.closeEditDialog()"
          >
            {{ t("common.cancel") }}
          </Button>
          <Button class="cursor-pointer" @click="handleSave">
            {{
              manager.editingPreset.value
                ? t("common.save")
                : t("settings.preset.create")
            }}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

    <!-- 删除确认对话框 -->
    <Dialog
      :open="manager.showDeleteDialog.value"
      @update:open="manager.closeDeleteDialog()"
    >
      <DialogContent class="sm:max-w-sm">
        <DialogHeader>
          <DialogTitle>{{ t("common.confirm", "确认") }}</DialogTitle>
        </DialogHeader>
        <p class="text-sm" style="color: var(--text-secondary)">
          {{
            t(
              "settings.preset.deleteConfirm",
              "确定要删除预设「{name}」吗？此操作不可恢复。",
            ).replace("{name}", manager.deletingPreset.value?.name || "")
          }}
        </p>
        <DialogFooter>
          <Button
            variant="outline"
            class="cursor-pointer"
            @click="manager.closeDeleteDialog()"
          >
            {{ t("common.cancel") }}
          </Button>
          <Button
            variant="destructive"
            class="cursor-pointer"
            @click="manager.deletePreset()"
          >
            {{ t("common.delete", "删除") }}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </div>
</template>
