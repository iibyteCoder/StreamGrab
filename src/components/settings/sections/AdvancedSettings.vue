<script setup lang="ts">
/**
 * AdvancedSettings - 高级设置组件
 */

import { RotateCcw, AlertTriangle } from 'lucide-vue-next';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from '@/components/ui/alert-dialog';
import { SettingSelect, SettingInput, SettingSwitch } from '..';

interface Settings {
  advanced: {
    ffmpegPath: string;
    n_m3u8dlPath: string;
    logLevel: string;
    logFilePath: string;
    noLog: boolean;
    allowHlsMultiExtMap: boolean;
    disableUpdateCheck: boolean;
    urlProcessorArgs: string;
  };
}

interface Props {
  settings: Settings;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: 'update:settings', value: any): void;
  (e: 'reset'): void;
}>();

// 日志级别选项
const logLevelOptions = [
  { value: 'DEBUG', label: '调试' },
  { value: 'INFO', label: '信息' },
  { value: 'WARN', label: '警告' },
  { value: 'ERROR', label: '错误' },
  { value: 'OFF', label: '关闭' },
];

// 更新设置
const updateAdvanced = (value: any) => {
  emit('update:settings', value);
};

// 重置设置
const handleReset = () => {
  emit('reset');
};
</script>

<template>
  <div class="space-y-4">
    <Card>
      <CardHeader>
        <CardTitle class="text-base">工具路径</CardTitle>
        <CardDescription>配置外部工具的路径</CardDescription>
      </CardHeader>
      <CardContent class="space-y-4">
        <SettingInput
          :model-value="settings.advanced.ffmpegPath"
          label="FFmpeg 路径"
          placeholder="留空则使用系统 PATH"
          class="flex-1"
          @update:model-value="updateAdvanced({ ffmpegPath: $event })"
        />

        <SettingInput
          :model-value="settings.advanced.n_m3u8dlPath"
          label="N_m3u8DL-RE 路径"
          placeholder="留空则使用系统 PATH"
          class="flex-1"
          @update:model-value="updateAdvanced({ n_m3u8dlPath: $event })"
        />
      </CardContent>
    </Card>

    <Card>
      <CardHeader>
        <CardTitle class="text-base">日志设置</CardTitle>
      </CardHeader>
      <CardContent class="space-y-4">
        <SettingSelect
          :model-value="settings.advanced.logLevel"
          label="日志级别"
          :options="logLevelOptions"
          placeholder="选择级别"
          @update:model-value="updateAdvanced({ logLevel: $event })"
        />

        <SettingInput
          :model-value="settings.advanced.logFilePath"
          label="日志文件路径"
          placeholder="留空则不写入文件"
          class="flex-1"
          @update:model-value="updateAdvanced({ logFilePath: $event })"
        />

        <SettingSwitch
          :model-value="settings.advanced.noLog"
          label="禁用日志"
          @update:model-value="updateAdvanced({ noLog: $event })"
        />
      </CardContent>
    </Card>

    <Card>
      <CardHeader>
        <CardTitle class="text-base">实验性功能</CardTitle>
      </CardHeader>
      <CardContent class="space-y-3">
        <SettingSwitch
          :model-value="settings.advanced.allowHlsMultiExtMap"
          label="允许多 EXT-X-MAP"
          description="允许 HLS 多个 EXT-X-MAP 标签"
          @update:model-value="updateAdvanced({ allowHlsMultiExtMap: $event })"
        />

        <SettingSwitch
          :model-value="settings.advanced.disableUpdateCheck"
          label="禁用更新检查"
          @update:model-value="updateAdvanced({ disableUpdateCheck: $event })"
        />

        <SettingInput
          :model-value="settings.advanced.urlProcessorArgs"
          label="URL 处理器参数"
          placeholder="传递给 URL 处理器的额外参数"
          class="flex-1"
          @update:model-value="updateAdvanced({ urlProcessorArgs: $event })"
        />
      </CardContent>
    </Card>

    <!-- 恢复默认配置 -->
    <Card class="border-destructive/20">
      <CardHeader>
        <CardTitle class="flex items-center gap-2 text-base text-destructive">
          <AlertTriangle class="h-4 w-4" />
          危险操作
        </CardTitle>
        <CardDescription>以下操作不可撤销，请谨慎使用</CardDescription>
      </CardHeader>
      <CardContent>
        <div class="flex items-center justify-between">
          <div class="space-y-0.5">
            <p class="text-sm font-medium">恢复默认配置</p>
            <p class="text-xs text-muted-foreground">将所有设置恢复为默认值</p>
          </div>
          <AlertDialog>
            <AlertDialogTrigger as-child>
              <Button variant="outline" size="sm" class="text-destructive border-destructive/30 hover:bg-destructive/10">
                <RotateCcw class="mr-2 h-4 w-4" />
                恢复默认
              </Button>
            </AlertDialogTrigger>
            <AlertDialogContent>
              <AlertDialogHeader>
                <AlertDialogTitle>确认恢复默认配置？</AlertDialogTitle>
                <AlertDialogDescription>
                  此操作将把所有设置恢复为默认值，当前配置将被覆盖。此操作无法撤销。
                </AlertDialogDescription>
              </AlertDialogHeader>
              <AlertDialogFooter>
                <AlertDialogCancel>取消</AlertDialogCancel>
                <AlertDialogAction @click="handleReset">
                  确认恢复
                </AlertDialogAction>
              </AlertDialogFooter>
            </AlertDialogContent>
          </AlertDialog>
        </div>
      </CardContent>
    </Card>
  </div>
</template>
