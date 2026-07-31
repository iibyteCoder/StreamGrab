<script setup lang="ts">
/**
 * AddTaskDialog —— 主从详情式暂存层编排外壳（重写）。
 *
 * L1 总览（TaskStagingList）：粘贴 + 批次默认 + 行清单。
 * L2 聚焦（LinkConfigPanel）：单条引擎类型驱动配置 + 内联流选择。
 * 单链接（len==1）：直接进 L2，零跳转。
 * 提交：resolveLinkToTask 三层合并 → addAndStartTask / taskStore.addTask。
 * 后端契约零改动。
 */
import { ref, computed, watch, nextTick } from "vue";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { AppIcon, UrlDuplicateDialog } from "@/components/common";
import { useToast, useDownloader, usePresetManager } from "@/composables";
import { useSettingsStore, useTaskStore } from "@/stores";
import { systemService } from "@/services";
import { detectUrlType, isStreamingType } from "@/domain/url";
import { extractFileName } from "@/utils/format";
import { generateId } from "@/utils/id";
import type { DownloadTask } from "@/domain";
import TaskStagingList from "./TaskStagingList.vue";
import LinkConfigPanel from "./LinkConfigPanel.vue";
import { resolveLinkToTask, seedPresetOverrides } from "./resolveLinkToTask";
import type { BatchDefaults, StagedLink } from "./staging-types";

interface Props {
  open: boolean;
}
const props = defineProps<Props>();
const emit = defineEmits<{ (e: "update:open", value: boolean): void }>();

const toast = useToast();
const settingsStore = useSettingsStore();
const taskStore = useTaskStore();
const { addAndStartTask, parseUrl } = useDownloader();
const { applyPreset } = usePresetManager();

const isOpen = computed({
  get: () => props.open,
  set: (v) => emit("update:open", v),
});

// ===== 状态 =====
const staged = ref<StagedLink[]>([]);
const view = ref<"list" | "focus">("list");
const selectedId = ref<string | null>(null);
const isSubmitting = ref(false);

const batch = ref<BatchDefaults>({ saveDir: "", autoStart: false });
const batchPresetId = ref<string>("__none__");

// URL 重复
const showUrlDuplicateDialog = ref(false);
const duplicateTask = ref<DownloadTask | null>(null);
const pendingResume = ref<(() => Promise<void>) | null>(null);

const isSingle = computed(() => staged.value.length === 1);
const selectedLink = computed(
  () => staged.value.find((l) => l.id === selectedId.value) ?? null,
);
const canCommit = computed(
  () => staged.value.some((l) => l.status !== "invalid") && !isSubmitting.value,
);
const globalSaveDir = computed(() => settingsStore.defaultSaveDir);
const saveDirPlaceholder = computed(() => {
  const b = batch.value.saveDir.trim();
  const g = globalSaveDir.value;
  if (b) return `将使用批次默认：${b}`;
  if (g) return `将使用全局默认：${g}`;
  return "使用全局默认";
});

// ===== 生命周期 =====
watch(isOpen, async (open) => {
  if (open) {
    batch.value = {
      saveDir: "",
      autoStart: settingsStore.autoStartDownload,
    };
    batchPresetId.value = "__none__";
    staged.value = [];
    view.value = "list";
    selectedId.value = null;
    await nextTick();
  }
});

const reset = () => {
  staged.value = [];
  view.value = "list";
  selectedId.value = null;
  isSubmitting.value = false;
  batch.value = { saveDir: "", autoStart: false };
  batchPresetId.value = "__none__";
  showUrlDuplicateDialog.value = false;
  duplicateTask.value = null;
  pendingResume.value = null;
};

// ===== 粘贴 → 构造 StagedLink[]（编排者持有构造逻辑） =====
function buildLinks(text: string): StagedLink[] {
  const presetOv =
    batchPresetId.value !== "__none__"
      ? applyPreset(batchPresetId.value)
      : null;
  const lines = text
    .split("\n")
    .map((l) => l.trim())
    .filter((l) => l.startsWith("http://") || l.startsWith("https://"));
  return lines.map((url) => {
    const detectedType = detectUrlType(url);
    const streaming = isStreamingType(detectedType);
    return {
      id: generateId(),
      url,
      detectedType,
      fileName: extractFileName(url),
      saveDir: "",
      overrides: seedPresetOverrides(presetOv, detectedType),
      status: streaming ? ("pending" as const) : ("ready" as const),
    };
  });
}

function handlePaste(text: string) {
  const links = buildLinks(text);
  if (links.length === 0) return;
  staged.value = [...staged.value, ...links];
  if (isSingle.value) {
    selectedId.value = staged.value[0]!.id;
    view.value = "focus";
    void maybeAutoParse();
  } else {
    view.value = "list";
  }
}

// 单链接流媒体：进入聚焦即自动解析一次
async function maybeAutoParse() {
  if (!isSingle.value || !selectedLink.value) return;
  const link = selectedLink.value;
  if (
    link.detectedType &&
    isStreamingType(link.detectedType) &&
    !link.streamInfo
  ) {
    const info = await parseUrl(link.url);
    if (info) {
      link.streamInfo = info;
      link.status = "parsed";
    }
  }
}

// ===== 批次预设变更：重播种未触碰的流媒体行 =====
function handlePresetChange(presetId: string) {
  batchPresetId.value = presetId;
  const presetOv = presetId !== "__none__" ? applyPreset(presetId) : null;
  for (const link of staged.value) {
    if (
      link.status === "pending" &&
      link.detectedType &&
      isStreamingType(link.detectedType)
    ) {
      link.overrides = seedPresetOverrides(presetOv, link.detectedType);
    }
  }
}

// ===== 选择/导航 =====
function handleSelect(id: string) {
  selectedId.value = id;
  view.value = "focus";
}
function handleFocusDone() {
  if (selectedLink.value && selectedLink.value.status !== "invalid") {
    selectedLink.value.status = "ready";
  }
  view.value = isSingle.value ? "focus" : "list";
}
function handleRemove(id: string) {
  staged.value = staged.value.filter((l) => l.id !== id);
  if (selectedId.value === id) selectedId.value = null;
  if (isSingle.value && staged.value[0]) {
    selectedId.value = staged.value[0]!.id;
  }
}

// 保存目录浏览（经 systemService，仅编排者知道 service）
async function handleBrowseSaveDir() {
  const dir = await systemService.selectDirectory();
  if (dir) batch.value.saveDir = dir;
}

// ===== 提交 =====
async function handleCommit() {
  if (isSubmitting.value || !canCommit.value) return;
  isSubmitting.value = true;
  const links = staged.value.filter((l) => l.status !== "invalid");
  await runSubmit(links, 0);
}

async function runSubmit(links: StagedLink[], from: number) {
  let success = 0;
  for (let i = from; i < links.length; i++) {
    const link = links[i]!;
    // URL 重复检测
    const existing = taskStore.checkUrlExists(link.url);
    if (existing) {
      duplicateTask.value = existing;
      showUrlDuplicateDialog.value = true;
      // 暂停：用户确认后从下一条继续（强制跳过检查）
      pendingResume.value = async () => {
        try {
          await addOne(link, true);
          success++;
        } catch {
          // 逐条失败不阻塞
        }
        await runSubmit(links, i + 1);
      };
      return; // 暂停
    }
    try {
      await addOne(link, false);
      success++;
    } catch {
      // 逐条失败不阻塞
    }
  }
  if (success > 0) {
    toast.success(`已添加 ${success} 个任务`);
  }
  isSubmitting.value = false;
  handleClose();
}

async function addOne(link: StagedLink, skipUrlCheck: boolean) {
  const resolved = resolveLinkToTask(link, batch.value, globalSaveDir.value);
  if (batch.value.autoStart && !resolved.hasSchedule) {
    await addAndStartTask(
      resolved.url,
      resolved.fileName,
      resolved.saveDir,
      resolved.overrides,
    );
  } else {
    await taskStore.addTask({
      url: resolved.url,
      fileName: resolved.fileName,
      saveDir: resolved.saveDir,
      overrides: resolved.overrides,
      skipUrlCheck,
    });
  }
}

async function handleUrlDuplicateConfirm() {
  showUrlDuplicateDialog.value = false;
  const resume = pendingResume.value;
  pendingResume.value = null;
  if (resume) await resume();
}

function handleUrlDuplicateCancel() {
  showUrlDuplicateDialog.value = false;
  pendingResume.value = null;
  isSubmitting.value = false;
  toast.warning("已取消，部分任务未添加");
}

// ===== 关闭 =====
function handleClose() {
  reset();
  isOpen.value = false;
}
</script>

<template>
  <Dialog v-model:open="isOpen">
    <DialogContent
      class="flex max-h-[85vh] max-w-[min(640px,calc(100vw-2rem))] flex-col"
      @close-auto-focus="reset"
    >
      <DialogHeader>
        <DialogTitle class="flex items-center gap-2">
          <AppIcon name="Plus" :size="20" />
          添加下载任务
        </DialogTitle>
        <DialogDescription class="sr-only">
          粘贴链接，逐条配置后添加下载任务
        </DialogDescription>
      </DialogHeader>

      <div class="-mx-2 flex-1 space-y-4 overflow-y-auto px-2">
        <!-- L1 总览：单链接时也显示（便于继续粘贴追加） -->
        <TaskStagingList
          v-if="view === 'list' || isSingle"
          :links="staged"
          :batch="batch"
          :batch-preset-id="batchPresetId"
          :global-save-dir="globalSaveDir"
          @update:batch="(b: BatchDefaults) => (batch = b)"
          @update:preset="handlePresetChange"
          @paste="handlePaste"
          @select="handleSelect"
          @remove="handleRemove"
          @commit="handleCommit"
        >
          <template #saveDirBrowse>
            <Button
              variant="outline"
              size="sm"
              class="h-9 px-3"
              @click="handleBrowseSaveDir"
            >
              <AppIcon name="FolderOpen" :size="14" />
            </Button>
          </template>
        </TaskStagingList>

        <!-- L2 聚焦 -->
        <div v-if="view === 'focus' && selectedLink" class="space-y-3">
          <button
            v-if="!isSingle"
            class="flex cursor-pointer items-center gap-1.5 text-sm text-muted-foreground transition-colors hover:text-foreground"
            @click="view = 'list'"
          >
            <AppIcon name="ChevronLeft" :size="14" />
            返回列表
          </button>
          <LinkConfigPanel
            :model-value="selectedLink"
            :save-dir-placeholder="saveDirPlaceholder"
            @done="handleFocusDone"
          />
        </div>
      </div>

      <!-- URL 重复确认 -->
      <UrlDuplicateDialog
        v-model:open="showUrlDuplicateDialog"
        :existing-task="duplicateTask"
        @confirm="handleUrlDuplicateConfirm"
        @cancel="handleUrlDuplicateCancel"
      />
    </DialogContent>
  </Dialog>
</template>
