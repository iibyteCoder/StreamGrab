<script setup lang="ts">
/**
 * AddTaskDialog —— 三段式向导薄壳。
 * 流程编排在 useAddTaskWizard；本组件只做渲染 + 路由用户操作。
 */
import { computed, ref, watch, nextTick } from "vue";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { AppIcon, UrlDuplicateDialog } from "@/components/common";
import { useAddTaskWizard } from "@/composables";
import LinkConfigCard from "./LinkConfigCard.vue";

interface Props {
  open: boolean;
  /** 预填链接并自动推进到配置步（来自右键菜单「以此链接重新下载」） */
  initialUrl?: string | null;
}
const props = defineProps<Props>();
const emit = defineEmits<{ (e: "update:open", value: boolean): void }>();

const isOpen = computed({
  get: () => props.open,
  set: (v) => emit("update:open", v),
});

const {
  step,
  current,
  index,
  total,
  isSingle,
  isLast,
  showAddAll,
  isSubmitting,
  parseDone,
  parseTotal,
  parsingId,
  dirs,
  defaultDir,
  showDuplicate,
  duplicateTask,
  reset,
  submitPaste,
  retryParse,
  browseSaveDir,
  addCurrent,
  skip,
  addAll,
  confirmDuplicate,
  cancelDuplicate,
} = useAddTaskWizard();

const pasteText = ref("");
const isDragging = ref(false);
const textareaRef = ref<HTMLTextAreaElement | null>(null);

watch(isOpen, async (open) => {
  if (!open) return;
  reset();
  if (props.initialUrl) {
    // 右键菜单「以此链接重新下载」：预填并自动提交解析，
    // 复用既有 submitPaste 链路（resolveLinkToTask / 重复检测）直达配置步
    pasteText.value = props.initialUrl;
    void submitPaste(props.initialUrl);
  } else {
    pasteText.value = "";
  }
  await nextTick();
  textareaRef.value?.focus();
});

// 向导进入 done → 关闭弹窗
watch(step, (s) => {
  if (s === "done") isOpen.value = false;
});

function handleSubmitPaste() {
  if (pasteText.value.trim()) void submitPaste(pasteText.value);
}
function onPasteKeydown(e: KeyboardEvent) {
  if (e.key === "Enter" && !e.shiftKey) {
    e.preventDefault();
    handleSubmitPaste();
  }
}
function onDrop(e: DragEvent) {
  e.preventDefault();
  isDragging.value = false;
  const text = e.dataTransfer?.getData("text/plain");
  if (text) pasteText.value = text;
}
// 配置步 Enter = 添加/完成（避开 textarea，其由 onPasteKeydown 处理）
function onContentKeydown(e: KeyboardEvent) {
  if (e.key !== "Enter" || e.shiftKey) return;
  if ((e.target as HTMLElement)?.tagName === "TEXTAREA") return;
  if (step.value !== "config") return;
  e.preventDefault();
  void addCurrent();
}
</script>

<template>
  <Dialog v-model:open="isOpen">
    <DialogContent
      class="flex max-h-[85vh] max-w-[min(600px,calc(100vw-2rem))] flex-col"
    >
      <DialogHeader>
        <DialogTitle class="flex items-center gap-2">
          <AppIcon name="Plus" :size="20" />
          添加下载任务
          <span
            v-if="step === 'config' && !isSingle"
            class="text-sm font-normal text-muted-foreground"
          >
            {{ index + 1 }}/{{ total }}
          </span>
        </DialogTitle>
        <DialogDescription class="sr-only"
          >粘贴链接并配置下载任务</DialogDescription
        >
      </DialogHeader>

      <div
        class="-mx-2 flex-1 space-y-4 overflow-y-auto px-2"
        @keydown="onContentKeydown"
      >
        <!-- 步骤 1：粘贴 -->
        <div
          v-if="step === 'paste'"
          class="space-y-4"
          @dragover.prevent="isDragging = true"
          @dragleave="isDragging = false"
          @drop="onDrop"
        >
          <div class="relative">
            <div
              v-if="isDragging"
              class="absolute inset-0 z-10 flex items-center justify-center rounded-lg border-2 border-dashed border-primary bg-primary/10"
            >
              <span class="text-sm font-medium text-primary"
                >释放以粘贴链接</span
              >
            </div>
            <textarea
              ref="textareaRef"
              v-model="pasteText"
              placeholder="粘贴下载链接，每行一个（支持 M3U8 / DASH / MP4 直链）"
              class="h-40 w-full resize-none rounded-lg border bg-muted/50 px-3 py-2 text-sm transition-colors focus:border-primary focus:outline-none focus:ring-2 focus:ring-primary/50"
              @keydown="onPasteKeydown"
            />
          </div>
          <div class="flex justify-end">
            <Button :disabled="!pasteText.trim()" @click="handleSubmitPaste">
              <AppIcon name="ArrowRight" :size="16" class="mr-2" />
              解析并添加
            </Button>
          </div>
        </div>

        <!-- 步骤 2：解析中 -->
        <div
          v-else-if="step === 'parsing'"
          class="flex flex-col items-center justify-center gap-3 py-16"
        >
          <AppIcon
            name="Loader2"
            :size="32"
            class="animate-spin text-primary"
          />
          <span class="text-sm text-muted-foreground">
            正在解析 {{ parseTotal }} 个链接…（{{ parseDone }}/{{
              parseTotal
            }}）
          </span>
        </div>

        <!-- 步骤 3：逐条配置 -->
        <div v-else-if="step === 'config' && current" class="space-y-4">
          <LinkConfigCard
            :model-value="current"
            :recent-dirs="dirs"
            :default-dir="defaultDir"
            :parsing="parsingId === current.id"
            @parse="current && retryParse(current)"
            @browse-save-dir="browseSaveDir"
          />
          <div class="flex items-center justify-between border-t pt-3">
            <Button
              v-if="!isSingle"
              variant="ghost"
              size="sm"
              :disabled="isSubmitting"
              @click="skip"
              >跳过</Button
            >
            <span v-else />
            <div class="flex gap-2">
              <Button
                v-if="showAddAll"
                variant="outline"
                size="sm"
                :disabled="isSubmitting"
                @click="addAll"
              >
                全部添加
              </Button>
              <Button size="sm" :disabled="isSubmitting" @click="addCurrent">
                <AppIcon name="Download" :size="16" class="mr-1.5" />
                {{ isLast ? "完成" : "添加" }}
              </Button>
            </div>
          </div>
        </div>
      </div>

      <!-- URL 重复确认 -->
      <UrlDuplicateDialog
        v-model:open="showDuplicate"
        :existing-task="duplicateTask"
        @confirm="confirmDuplicate"
        @cancel="cancelDuplicate"
      />
    </DialogContent>
  </Dialog>
</template>
