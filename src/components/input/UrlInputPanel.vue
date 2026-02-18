<script setup lang="ts">
/**
 * UrlInputPanel - URL 输入面板组件
 * 支持多行输入、批量添加、文件导入、定时开始
 */

import { ref, computed } from "vue";
import { Textarea } from "@/components/ui/textarea";
import { Button } from "@/components/ui/button";
import { AppIcon } from "@/components/common";
import { useToast } from "@/composables";
import { open } from "@tauri-apps/plugin-dialog";
import { readTextFile } from "@tauri-apps/plugin-fs";

interface Props {
  modelValue: string;
  isSubmitting?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  isSubmitting: false,
});

const emit = defineEmits<{
  (e: "update:modelValue", value: string): void;
  (e: "download", options?: { startAt?: Date }): void;
}>();

const toast = useToast();

// 高级选项展开状态
const showAdvanced = ref(false);

// 定时开始时间
const scheduledDate = ref<string>("");
const scheduledTime = ref<string>("");

// 计算有效的 URL 数量
const urlCount = computed(() => {
  if (!props.modelValue.trim()) return 0;
  return parseUrls(props.modelValue).length;
});

// 计算定时开始时间
const scheduledStartAt = computed((): Date | undefined => {
  if (!scheduledDate.value) return undefined;
  const dateTime = scheduledTime.value || "00:00";
  const dateStr = `${scheduledDate.value}T${dateTime}`;
  const date = new Date(dateStr);
  return isNaN(date.getTime()) ? undefined : date;
});

/**
 * 从文本中解析 URL 列表
 */
const parseUrls = (text: string): string[] => {
  return text
    .split("\n")
    .map((line) => line.trim())
    .filter(
      (line) =>
        line.length > 0 &&
        (line.startsWith("http://") || line.startsWith("https://")),
    );
};

/**
 * 从文件导入 URL
 */
const handleImportFile = async () => {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: "文本文件", extensions: ["txt", "text"] }],
      title: "选择 URL 列表文件",
    });

    if (!selected) return;

    const filePath = selected;
    if (!filePath) return;

    const content = await readTextFile(filePath);
    if (!content.trim()) {
      toast.warning("文件内容为空");
      return;
    }

    const urls = parseUrls(content);
    if (urls.length === 0) {
      toast.warning("文件中未找到有效的链接");
      return;
    }

    let newValue = props.modelValue;
    if (newValue.trim()) {
      newValue += "\n";
    }
    emit("update:modelValue", newValue + urls.join("\n"));

    toast.success(`已导入 ${urls.length} 个链接`);
  } catch (error) {
    console.error("Import file error:", error);
    toast.error(
      `导入失败: ${error instanceof Error ? error.message : "未知错误"}`,
    );
  }
};

/**
 * 处理下载
 */
const handleDownload = () => {
  const options: { startAt?: Date } = {};
  if (scheduledStartAt.value) {
    options.startAt = scheduledStartAt.value;
  }
  emit("download", options);
};

/**
 * 清除定时设置
 */
const clearSchedule = () => {
  scheduledDate.value = "";
  scheduledTime.value = "";
};
</script>

<template>
  <div class="space-y-2">
    <Textarea
      :model-value="modelValue"
      @update:model-value="
        (val: string | number) => emit('update:modelValue', String(val))
      "
      placeholder="输入下载链接，每行一个&#10;例如:&#10;https://example.com/video1.m3u8&#10;https://example.com/video2.m3u8"
      class="min-h-[80px] resize-none"
      @keydown.ctrl.enter="handleDownload"
    />

    <!-- 高级选项 -->
    <div class="space-y-2">
      <button
        class="flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground transition-colors"
        @click="showAdvanced = !showAdvanced"
      >
        <AppIcon
          :name="showAdvanced ? 'ChevronDown' : 'ChevronRight'"
          :size="12"
        />
        高级选项
      </button>

      <div
        v-if="showAdvanced"
        class="flex items-center gap-3 p-2 rounded-md bg-muted/50"
      >
        <span class="text-xs text-muted-foreground whitespace-nowrap"
          >定时开始:</span
        >
        <input
          v-model="scheduledDate"
          type="date"
          class="h-7 px-2 text-xs rounded border border-input bg-background focus:outline-none focus:ring-1 focus:ring-ring"
        />
        <input
          v-model="scheduledTime"
          type="time"
          class="h-7 px-2 text-xs rounded border border-input bg-background focus:outline-none focus:ring-1 focus:ring-ring"
        />
        <button
          v-if="scheduledDate"
          class="text-xs text-muted-foreground hover:text-foreground"
          @click="clearSchedule"
        >
          清除
        </button>
      </div>
    </div>

    <!-- 操作栏 -->
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-2">
        <span class="text-xs text-muted-foreground">
          <template v-if="urlCount > 0">
            已输入 {{ urlCount }} 个链接
            <span v-if="scheduledStartAt" class="text-primary ml-1">
              · 定时
              {{
                scheduledStartAt.toLocaleString("zh-CN", {
                  month: "short",
                  day: "numeric",
                  hour: "2-digit",
                  minute: "2-digit",
                })
              }}
            </span>
          </template>
          <template v-else>Ctrl + Enter 快速添加</template>
        </span>
        <Button
          variant="ghost"
          size="sm"
          class="h-6 px-2 text-xs"
          @click="handleImportFile"
        >
          <AppIcon name="FileUp" :size="12" class="mr-1" />
          导入
        </Button>
      </div>
      <Button
        :loading="isSubmitting"
        :disabled="urlCount === 0"
        @click="handleDownload"
      >
        <template v-if="urlCount > 1"> 下载 ({{ urlCount }}) </template>
        <template v-else> 下载 </template>
      </Button>
    </div>
  </div>
</template>
