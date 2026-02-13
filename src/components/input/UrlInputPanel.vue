<script setup lang="ts">
/**
 * UrlInputPanel - URL 输入面板组件
 * 支持多行输入、批量添加、文件导入
 */

import { ref, computed } from 'vue';
import { FileUp } from 'lucide-vue-next';
import { Textarea } from '@/components/ui/textarea';
import { Button } from '@/components/ui/button';
import { AppIcon } from '@/components/common';
import { useToast } from '@/composables';
import { open } from '@tauri-apps/plugin-dialog';
import { readTextFile } from '@tauri-apps/plugin-fs';

interface Props {
  modelValue: string;
  isSubmitting?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  isSubmitting: false,
});

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void;
  (e: 'download'): void;
}>();

const toast = useToast();

// 计算有效的 URL 数量
const urlCount = computed(() => {
  if (!props.modelValue.trim()) return 0;
  return parseUrls(props.modelValue).length;
});

/**
 * 从文本中解析 URL 列表
 */
const parseUrls = (text: string): string[] => {
  return text
    .split('\n')
    .map(line => line.trim())
    .filter(line => line.length > 0 && (line.startsWith('http://') || line.startsWith('https://')));
};

/**
 * 从文件导入 URL
 */
const handleImportFile = async () => {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: '文本文件', extensions: ['txt', 'text'] }],
      title: '选择 URL 列表文件',
    });

    if (!selected) return;

    // open with multiple: false returns string | null
    const filePath = selected;
    if (!filePath) return;

    const content = await readTextFile(filePath);
    if (!content.trim()) {
      toast.warning('文件内容为空');
      return;
    }

    // 解析 URL
    const urls = parseUrls(content);
    if (urls.length === 0) {
      toast.warning('文件中未找到有效的链接');
      return;
    }

    // 追加到输入框
    let newValue = props.modelValue;
    if (newValue.trim()) {
      newValue += '\n';
    }
    emit('update:modelValue', newValue + urls.join('\n'));

    toast.success(`已导入 ${urls.length} 个链接`);
  } catch (error) {
    console.error('Import file error:', error);
    toast.error(`导入失败: ${error instanceof Error ? error.message : '未知错误'}`);
  }
};

/**
 * 处理下载
 */
const handleDownload = () => {
  emit('download');
};
</script>

<template>
  <div class="space-y-2">
    <Textarea
      :model-value="modelValue"
      @update:model-value="emit('update:modelValue', $event)"
      placeholder="输入下载链接，每行一个&#10;例如:&#10;https://example.com/video1.m3u8&#10;https://example.com/video2.m3u8"
      class="min-h-[80px] resize-none"
      @keydown.ctrl.enter="handleDownload"
    />
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-2">
        <span class="text-xs text-muted-foreground">
          <template v-if="urlCount > 0">已输入 {{ urlCount }} 个链接</template>
          <template v-else>Ctrl + Enter 快速添加</template>
        </span>
        <Button variant="ghost" size="sm" class="h-6 px-2 text-xs" @click="handleImportFile">
          <AppIcon :name="FileUp" :size="12" class="mr-1" />
          导入
        </Button>
      </div>
      <Button
        :loading="isSubmitting"
        :disabled="urlCount === 0"
        @click="handleDownload"
      >
        <template v-if="urlCount > 1">
          下载 ({{ urlCount }})
        </template>
        <template v-else>
          下载
        </template>
      </Button>
    </div>
  </div>
</template>
