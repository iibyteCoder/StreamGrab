<script setup lang="ts">
/**
 * UrlInput URL 输入组件
 * 用于输入和验证 M3U8/MPD 链接
 */

import { ref, computed } from "vue";
import { AppInput, AppButton } from "@/components/common";
import { validateUrl, type UrlValidationResult } from "@/utils/validate";
import { useToast } from "@/composables";

interface Props {
  loading?: boolean;
  placeholder?: string;
}

withDefaults(defineProps<Props>(), {
  loading: false,
  placeholder: "输入 M3U8 / MPD / MSS 链接...",
});

const emit = defineEmits<{
  (e: "submit", url: string): void;
  (e: "parse", url: string): void;
}>();

const toast = useToast();

const url = ref("");
const validation = ref<UrlValidationResult | null>(null);

// 输入状态
const hasError = computed(() => validation.value && !validation.value.valid);
const hasWarning = computed(() => validation.value?.type === "unknown");

// 验证 URL
const validateInput = () => {
  if (!url.value.trim()) {
    validation.value = null;
    return;
  }
  validation.value = validateUrl(url.value);
};

// 处理输入
const handleInput = () => {
  validateInput();
};

// 处理粘贴
const handlePaste = async () => {
  await new Promise((resolve) => setTimeout(resolve, 0));
  validateInput();

  // 如果 URL 有效，自动触发解析
  if (validation.value?.valid) {
    emit("parse", url.value.trim());
  }
};

// 提交 URL
const handleSubmit = () => {
  const trimmedUrl = url.value.trim();

  if (!trimmedUrl) {
    toast.warning("请输入下载链接");
    return;
  }

  validateInput();

  if (!validation.value?.valid) {
    toast.error(validation.value?.error || "无效的链接");
    return;
  }

  emit("submit", trimmedUrl);

  // 清空输入
  url.value = "";
  validation.value = null;
};

// 清除输入
const handleClear = () => {
  url.value = "";
  validation.value = null;
};

// 状态提示
const statusText = computed(() => {
  if (!validation.value) return "";
  if (!validation.value.valid) return validation.value.error;
  if (validation.value.type === null) return "未识别的流类型";
  if (validation.value.type === "unknown") return "未识别的流类型";
  return `检测到 ${validation.value.type.toUpperCase()} 流`;
});

const statusClass = computed(() => {
  if (hasError.value) return "text-accent-error";
  if (hasWarning.value) return "text-yellow-500";
  if (validation.value?.valid) return "text-accent-success";
  return "";
});
</script>

<template>
  <div class="w-full">
    <div class="flex gap-2">
      <!-- URL 输入框 -->
      <div class="flex-1 relative">
        <AppInput
          v-model="url"
          :placeholder="placeholder"
          :error="hasError ?? undefined"
          clearable
          @input="handleInput"
          @paste="handlePaste"
          @enter="handleSubmit"
          @clear="handleClear"
        >
          <template #prefix>
            <svg
              class="w-4 h-4"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1"
              />
            </svg>
          </template>
        </AppInput>

        <!-- 状态提示 -->
        <Transition
          enter-active-class="transition-all duration-200"
          leave-active-class="transition-all duration-200"
        >
          <p
            v-if="statusText"
            class="absolute left-0 -bottom-5 text-xs"
            :class="statusClass"
          >
            {{ statusText }}
          </p>
        </Transition>
      </div>

      <!-- 下载按钮 -->
      <AppButton variant="primary" :loading="loading" @click="handleSubmit">
        <template #icon-left>
          <svg
            class="w-4 h-4"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"
            />
          </svg>
        </template>
        下载
      </AppButton>
    </div>
  </div>
</template>
