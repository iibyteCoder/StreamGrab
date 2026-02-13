<script setup lang="ts">
/**
 * SettingsView - 设置页面
 * 使用 Tabs 组织不同类别的设置
 */

import { ref, computed, onMounted, watch } from 'vue';
import { useRouter } from 'vue-router';
import {
  ArrowLeft,
  FolderOpen,
  Download,
  Video,
  Network,
  Key,
  Radio,
  Settings2,
  Palette,
  RotateCcw,
  Save,
  Folder,
  HardDrive,
} from 'lucide-vue-next';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import { Slider } from '@/components/ui/slider';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Separator } from '@/components/ui/separator';
import { useSettings, useToast } from '@/composables';

const router = useRouter();
const {
  settings,
  theme,
  loadSettings,
  saveSettings,
  resetSettings,
  setTheme,
  updateGeneral,
  updateDownload,
  updateMux,
  updateNetwork,
  updateDecryption,
  updateLive,
  updateAdvanced,
  enableAutoSave,
} = useSettings();
const toast = useToast();

const isSaving = ref(false);
const activeTab = ref('general');

// 初始化
onMounted(async () => {
  await loadSettings();
  enableAutoSave(500);
});

// 保存设置
const handleSave = async () => {
  if (isSaving.value) return;
  isSaving.value = true;
  try {
    await saveSettings();
    toast.success('设置已保存');
  } catch (error) {
    toast.error('保存设置失败');
  } finally {
    isSaving.value = false;
  }
};

// 重置设置
const handleReset = async () => {
  try {
    await resetSettings();
    toast.success('设置已重置');
  } catch (error) {
    toast.error('重置设置失败');
  }
};

// 选择目录
const selectDirectory = async (field: 'saveDir' | 'tmpDir') => {
  // TODO: 使用 Tauri 的 dialog API 选择目录
  toast.info('目录选择功能待实现');
};

// 主题选项
const themeOptions = [
  { value: 'light', label: '浅色' },
  { value: 'dark', label: '深色' },
  { value: 'system', label: '跟随系统' },
];

// 语言选项
const languageOptions = [
  { value: 'zh-CN', label: '简体中文' },
  { value: 'zh-TW', label: '繁体中文' },
  { value: 'en-US', label: 'English' },
];

// 混流格式选项
const muxFormatOptions = [
  { value: 'mp4', label: 'MP4' },
  { value: 'mkv', label: 'MKV' },
];

// 混流器选项
const muxerOptions = [
  { value: 'ffmpeg', label: 'FFmpeg' },
  { value: 'mkvmerge', label: 'MKVMerge' },
];

// 解密引擎选项
const decryptionEngineOptions = [
  { value: 'FFMPEG', label: 'FFmpeg' },
  { value: 'MP4DECRYPT', label: 'MP4Decrypt' },
  { value: 'SHAKA_PACKAGER', label: 'Shaka Packager' },
];

// 日志级别选项
const logLevelOptions = [
  { value: 'DEBUG', label: '调试' },
  { value: 'INFO', label: '信息' },
  { value: 'WARN', label: '警告' },
  { value: 'ERROR', label: '错误' },
  { value: 'OFF', label: '关闭' },
];

// 字幕格式选项
const subFormatOptions = [
  { value: 'SRT', label: 'SRT' },
  { value: 'VTT', label: 'WebVTT' },
];

// 格式化线程数显示
const threadCountDisplay = computed(() => `${settings.value.download.threadCount} 线程`);
</script>

<template>
  <div class="flex h-full flex-col bg-background">
    <!-- 头部 -->
    <header class="border-b p-4">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-3">
          <Button variant="ghost" size="icon" @click="router.push('/')">
            <ArrowLeft class="h-5 w-5" />
          </Button>
          <div>
            <h1 class="text-xl font-semibold">设置</h1>
            <p class="text-xs text-muted-foreground">配置应用程序选项</p>
          </div>
        </div>
        <div class="flex items-center gap-2">
          <Button variant="outline" size="sm" @click="handleReset">
            <RotateCcw class="mr-2 h-4 w-4" />
            重置
          </Button>
          <Button size="sm" :loading="isSaving" @click="handleSave">
            <Save class="mr-2 h-4 w-4" />
            保存
          </Button>
        </div>
      </div>
    </header>

    <!-- 设置内容 -->
    <Tabs v-model="activeTab" class="flex-1 flex flex-col">
      <div class="border-b px-4">
        <TabsList class="h-10">
          <TabsTrigger value="general" class="text-xs">
            <Folder class="mr-1.5 h-3.5 w-3.5" />
            常规
          </TabsTrigger>
          <TabsTrigger value="download" class="text-xs">
            <Download class="mr-1.5 h-3.5 w-3.5" />
            下载
          </TabsTrigger>
          <TabsTrigger value="mux" class="text-xs">
            <Video class="mr-1.5 h-3.5 w-3.5" />
            混流
          </TabsTrigger>
          <TabsTrigger value="network" class="text-xs">
            <Network class="mr-1.5 h-3.5 w-3.5" />
            网络
          </TabsTrigger>
          <TabsTrigger value="decryption" class="text-xs">
            <Key class="mr-1.5 h-3.5 w-3.5" />
            解密
          </TabsTrigger>
          <TabsTrigger value="live" class="text-xs">
            <Radio class="mr-1.5 h-3.5 w-3.5" />
            直播
          </TabsTrigger>
          <TabsTrigger value="advanced" class="text-xs">
            <Settings2 class="mr-1.5 h-3.5 w-3.5" />
            高级
          </TabsTrigger>
          <TabsTrigger value="ui" class="text-xs">
            <Palette class="mr-1.5 h-3.5 w-3.5" />
            界面
          </TabsTrigger>
        </TabsList>
      </div>

      <ScrollArea class="flex-1">
        <div class="p-4">
          <!-- 常规设置 -->
          <TabsContent value="general" class="mt-0 space-y-4">
            <Card>
              <CardHeader>
                <CardTitle class="text-base">存储位置</CardTitle>
                <CardDescription>设置下载和临时文件的保存位置</CardDescription>
              </CardHeader>
              <CardContent class="space-y-4">
                <div class="grid gap-2">
                  <Label>下载目录</Label>
                  <div class="flex gap-2">
                    <Input
                      :model-value="settings.general.saveDir"
                      @update:model-value="updateGeneral({ saveDir: $event })"
                      placeholder="./downloads"
                      class="flex-1"
                    />
                    <Button variant="outline" size="icon" @click="selectDirectory('saveDir')">
                      <FolderOpen class="h-4 w-4" />
                    </Button>
                  </div>
                </div>
                <div class="grid gap-2">
                  <Label>临时目录</Label>
                  <div class="flex gap-2">
                    <Input
                      :model-value="settings.general.tmpDir"
                      @update:model-value="updateGeneral({ tmpDir: $event })"
                      placeholder="./temp"
                      class="flex-1"
                    />
                    <Button variant="outline" size="icon" @click="selectDirectory('tmpDir')">
                      <FolderOpen class="h-4 w-4" />
                    </Button>
                  </div>
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle class="text-base">应用行为</CardTitle>
                <CardDescription>配置应用程序的默认行为</CardDescription>
              </CardHeader>
              <CardContent class="space-y-4">
                <div class="grid gap-2">
                  <Label>语言</Label>
                  <Select
                    :model-value="settings.general.language"
                    @update:model-value="updateGeneral({ language: $event as any })"
                  >
                    <SelectTrigger class="w-full">
                      <SelectValue placeholder="选择语言" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem
                        v-for="option in languageOptions"
                        :key="option.value"
                        :value="option.value"
                      >
                        {{ option.label }}
                      </SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                <div class="flex items-center justify-between">
                  <div class="space-y-0.5">
                    <Label>自动开始下载</Label>
                    <p class="text-xs text-muted-foreground">添加任务后自动开始下载</p>
                  </div>
                  <Switch
                    :checked="settings.general.autoStartDownload"
                    @update:checked="updateGeneral({ autoStartDownload: $event })"
                  />
                </div>

                <div class="flex items-center justify-between">
                  <div class="space-y-0.5">
                    <Label>最小化到托盘</Label>
                    <p class="text-xs text-muted-foreground">关闭窗口时最小化到系统托盘</p>
                  </div>
                  <Switch
                    :checked="settings.general.minimizeToTray"
                    @update:checked="updateGeneral({ minimizeToTray: $event })"
                  />
                </div>

                <div class="flex items-center justify-between">
                  <div class="space-y-0.5">
                    <Label>检查更新</Label>
                    <p class="text-xs text-muted-foreground">启动时自动检查新版本</p>
                  </div>
                  <Switch
                    :checked="settings.general.checkUpdate"
                    @update:checked="updateGeneral({ checkUpdate: $event })"
                  />
                </div>
              </CardContent>
            </Card>
          </TabsContent>

          <!-- 下载设置 -->
          <TabsContent value="download" class="mt-0 space-y-4">
            <Card>
              <CardHeader>
                <CardTitle class="text-base">下载参数</CardTitle>
                <CardDescription>配置下载相关的核心参数</CardDescription>
              </CardHeader>
              <CardContent class="space-y-4">
                <div class="grid gap-3">
                  <div class="flex items-center justify-between">
                    <Label>并发线程数</Label>
                    <span class="text-sm text-muted-foreground">{{ threadCountDisplay }}</span>
                  </div>
                  <Slider
                    :model-value="[settings.download.threadCount]"
                    @update:model-value="updateDownload({ threadCount: $event[0] })"
                    :min="1"
                    :max="32"
                    :step="1"
                  />
                </div>

                <div class="grid gap-2">
                  <Label>重试次数</Label>
                  <Input
                    type="number"
                    :model-value="settings.download.retryCount"
                    @update:model-value="updateDownload({ retryCount: parseInt($event) || 3 })"
                    :min="0"
                    :max="10"
                    class="w-24"
                  />
                </div>

                <div class="grid gap-2">
                  <Label>超时时间 (秒)</Label>
                  <Input
                    type="number"
                    :model-value="settings.download.timeout"
                    @update:model-value="updateDownload({ timeout: parseInt($event) || 30 })"
                    :min="5"
                    :max="300"
                    class="w-24"
                  />
                </div>

                <div class="grid gap-2">
                  <Label>最大下载速度</Label>
                  <Input
                    :model-value="settings.download.maxSpeed"
                    @update:model-value="updateDownload({ maxSpeed: $event })"
                    placeholder="0 = 不限制"
                    class="w-32"
                  />
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle class="text-base">流选择</CardTitle>
                <CardDescription>默认选择视频/音频/字幕流</CardDescription>
              </CardHeader>
              <CardContent class="space-y-4">
                <div class="flex items-center justify-between">
                  <div class="space-y-0.5">
                    <Label>自动选择最佳流</Label>
                    <p class="text-xs text-muted-foreground">自动选择最高质量的流</p>
                  </div>
                  <Switch
                    :checked="settings.download.autoSelect"
                    @update:checked="updateDownload({ autoSelect: $event })"
                  />
                </div>

                <Separator />

                <div class="grid gap-2">
                  <Label>视频流选择</Label>
                  <Input
                    :model-value="settings.download.selectVideo"
                    @update:model-value="updateDownload({ selectVideo: $event })"
                    placeholder="例如: res=1080"
                    class="flex-1"
                  />
                </div>

                <div class="grid gap-2">
                  <Label>音频流选择</Label>
                  <Input
                    :model-value="settings.download.selectAudio"
                    @update:model-value="updateDownload({ selectAudio: $event })"
                    placeholder="例如: lang=zh"
                    class="flex-1"
                  />
                </div>

                <div class="grid gap-2">
                  <Label>字幕流选择</Label>
                  <Input
                    :model-value="settings.download.selectSubtitle"
                    @update:model-value="updateDownload({ selectSubtitle: $event })"
                    placeholder="例如: lang=zh"
                    class="flex-1"
                  />
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle class="text-base">下载选项</CardTitle>
              </CardHeader>
              <CardContent class="space-y-3">
                <div class="flex items-center justify-between">
                  <div class="space-y-0.5">
                    <Label>检查分片数量</Label>
                  </div>
                  <Switch
                    :checked="settings.download.checkSegmentsCount"
                    @update:checked="updateDownload({ checkSegmentsCount: $event })"
                  />
                </div>

                <div class="flex items-center justify-between">
                  <div class="space-y-0.5">
                    <Label>完成后删除临时文件</Label>
                  </div>
                  <Switch
                    :checked="settings.download.delAfterDone"
                    @update:checked="updateDownload({ delAfterDone: $event })"
                  />
                </div>

                <div class="flex items-center justify-between">
                  <div class="space-y-0.5">
                    <Label>跳过合并</Label>
                  </div>
                  <Switch
                    :checked="settings.download.skipMerge"
                    @update:checked="updateDownload({ skipMerge: $event })"
                  />
                </div>

                <div class="flex items-center justify-between">
                  <div class="space-y-0.5">
                    <Label>写入元数据 JSON</Label>
                  </div>
                  <Switch
                    :checked="settings.download.writeMetaJson"
                    @update:checked="updateDownload({ writeMetaJson: $event })"
                  />
                </div>

                <div class="flex items-center justify-between">
                  <div class="space-y-0.5">
                    <Label>二进制合并</Label>
                  </div>
                  <Switch
                    :checked="settings.download.binaryMerge"
                    @update:checked="updateDownload({ binaryMerge: $event })"
                  />
                </div>

                <div class="flex items-center justify-between">
                  <div class="space-y-0.5">
                    <Label>并发下载</Label>
                  </div>
                  <Switch
                    :checked="settings.download.concurrentDownload"
                    @update:checked="updateDownload({ concurrentDownload: $event })"
                  />
                </div>
              </CardContent>
            </Card>
          </TabsContent>

          <!-- 混流设置 -->
          <TabsContent value="mux" class="mt-0 space-y-4">
            <Card>
              <CardHeader>
                <CardTitle class="text-base">混流配置</CardTitle>
                <CardDescription>配置视频混流相关选项</CardDescription>
              </CardHeader>
              <CardContent class="space-y-4">
                <div class="grid gap-2">
                  <Label>输出格式</Label>
                  <Select
                    :model-value="settings.mux.format"
                    @update:model-value="updateMux({ format: $event as any })"
                  >
                    <SelectTrigger class="w-full">
                      <SelectValue placeholder="选择格式" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem
                        v-for="option in muxFormatOptions"
                        :key="option.value"
                        :value="option.value"
                      >
                        {{ option.label }}
                      </SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                <div class="grid gap-2">
                  <Label>混流器</Label>
                  <Select
                    :model-value="settings.mux.muxer"
                    @update:model-value="updateMux({ muxer: $event as any })"
                  >
                    <SelectTrigger class="w-full">
                      <SelectValue placeholder="选择混流器" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem
                        v-for="option in muxerOptions"
                        :key="option.value"
                        :value="option.value"
                      >
                        {{ option.label }}
                      </SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                <div class="grid gap-2">
                  <Label>混流器路径</Label>
                  <div class="flex gap-2">
                    <Input
                      :model-value="settings.mux.binPath"
                      @update:model-value="updateMux({ binPath: $event })"
                      placeholder="留空则使用系统 PATH"
                      class="flex-1"
                    />
                  </div>
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle class="text-base">混流选项</CardTitle>
              </CardHeader>
              <CardContent class="space-y-3">
                <div class="flex items-center justify-between">
                  <div class="space-y-0.5">
                    <Label>保留原始文件</Label>
                    <p class="text-xs text-muted-foreground">混流后保留分离的音视频文件</p>
                  </div>
                  <Switch
                    :checked="settings.mux.keepOriginal"
                    @update:checked="updateMux({ keepOriginal: $event })"
                  />
                </div>

                <div class="flex items-center justify-between">
                  <div class="space-y-0.5">
                    <Label>跳过字幕</Label>
                  </div>
                  <Switch
                    :checked="settings.mux.skipSubtitles"
                    @update:checked="updateMux({ skipSubtitles: $event })"
                  />
                </div>

                <div class="flex items-center justify-between">
                  <div class="space-y-0.5">
                    <Label>不包含日期信息</Label>
                  </div>
                  <Switch
                    :checked="settings.mux.noDateInfo"
                    @update:checked="updateMux({ noDateInfo: $event })"
                  />
                </div>

                <div class="flex items-center justify-between">
                  <div class="space-y-0.5">
                    <Label>使用 Concat 解复用器</Label>
                  </div>
                  <Switch
                    :checked="settings.mux.useConcatDemuxer"
                    @update:checked="updateMux({ useConcatDemuxer: $event })"
                  />
                </div>
              </CardContent>
            </Card>
          </TabsContent>

          <!-- 网络设置 -->
          <TabsContent value="network" class="mt-0 space-y-4">
            <Card>
              <CardHeader>
                <CardTitle class="text-base">代理设置</CardTitle>
                <CardDescription>配置网络代理选项</CardDescription>
              </CardHeader>
              <CardContent class="space-y-4">
                <div class="flex items-center justify-between">
                  <div class="space-y-0.5">
                    <Label>使用系统代理</Label>
                  </div>
                  <Switch
                    :checked="settings.network.useSystemProxy"
                    @update:checked="updateNetwork({ useSystemProxy: $event })"
                  />
                </div>

                <div class="grid gap-2">
                  <Label>自定义代理</Label>
                  <Input
                    :model-value="settings.network.customProxy"
                    @update:model-value="updateNetwork({ customProxy: $event })"
                    placeholder="http://127.0.0.1:7890"
                    class="flex-1"
                  />
                </div>

                <div class="grid gap-2">
                  <Label>Base URL</Label>
                  <Input
                    :model-value="settings.network.baseUrl"
                    @update:model-value="updateNetwork({ baseUrl: $event })"
                    placeholder="替换 URL 的基础部分"
                    class="flex-1"
                  />
                </div>

                <div class="flex items-center justify-between">
                  <div class="space-y-0.5">
                    <Label>附加 URL 参数</Label>
                  </div>
                  <Switch
                    :checked="settings.network.appendUrlParams"
                    @update:checked="updateNetwork({ appendUrlParams: $event })"
                  />
                </div>
              </CardContent>
            </Card>
          </TabsContent>

          <!-- 解密设置 -->
          <TabsContent value="decryption" class="mt-0 space-y-4">
            <Card>
              <CardHeader>
                <CardTitle class="text-base">解密引擎</CardTitle>
                <CardDescription>配置 DRM 解密相关选项</CardDescription>
              </CardHeader>
              <CardContent class="space-y-4">
                <div class="grid gap-2">
                  <Label>解密引擎</Label>
                  <Select
                    :model-value="settings.decryption.engine"
                    @update:model-value="updateDecryption({ engine: $event as any })"
                  >
                    <SelectTrigger class="w-full">
                      <SelectValue placeholder="选择引擎" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem
                        v-for="option in decryptionEngineOptions"
                        :key="option.value"
                        :value="option.value"
                      >
                        {{ option.label }}
                      </SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                <div class="grid gap-2">
                  <Label>解密器路径</Label>
                  <Input
                    :model-value="settings.decryption.binPath"
                    @update:model-value="updateDecryption({ binPath: $event })"
                    placeholder="留空则使用系统 PATH"
                    class="flex-1"
                  />
                </div>

                <div class="grid gap-2">
                  <Label>密钥文本文件</Label>
                  <Input
                    :model-value="settings.decryption.keyTextFile"
                    @update:model-value="updateDecryption({ keyTextFile: $event })"
                    placeholder="包含密钥的文本文件路径"
                    class="flex-1"
                  />
                </div>

                <div class="flex items-center justify-between">
                  <div class="space-y-0.5">
                    <Label>实时解密</Label>
                    <p class="text-xs text-muted-foreground">下载时实时解密分片</p>
                  </div>
                  <Switch
                    :checked="settings.decryption.realTimeDecryption"
                    @update:checked="updateDecryption({ realTimeDecryption: $event })"
                  />
                </div>
              </CardContent>
            </Card>
          </TabsContent>

          <!-- 直播设置 -->
          <TabsContent value="live" class="mt-0 space-y-4">
            <Card>
              <CardHeader>
                <CardTitle class="text-base">直播录制</CardTitle>
                <CardDescription>配置直播流录制选项</CardDescription>
              </CardHeader>
              <CardContent class="space-y-4">
                <div class="flex items-center justify-between">
                  <div class="space-y-0.5">
                    <Label>作为 VOD 处理</Label>
                    <p class="text-xs text-muted-foreground">将直播流当作点播处理</p>
                  </div>
                  <Switch
                    :checked="settings.live.performAsVod"
                    @update:checked="updateLive({ performAsVod: $event })"
                  />
                </div>

                <div class="flex items-center justify-between">
                  <div class="space-y-0.5">
                    <Label>实时合并</Label>
                  </div>
                  <Switch
                    :checked="settings.live.realTimeMerge"
                    @update:checked="updateLive({ realTimeMerge: $event })"
                  />
                </div>

                <div class="flex items-center justify-between">
                  <div class="space-y-0.5">
                    <Label>保留分片</Label>
                  </div>
                  <Switch
                    :checked="settings.live.keepSegments"
                    @update:checked="updateLive({ keepSegments: $event })"
                  />
                </div>

                <div class="flex items-center justify-between">
                  <div class="space-y-0.5">
                    <Label>管道混流</Label>
                  </div>
                  <Switch
                    :checked="settings.live.pipeMux"
                    @update:checked="updateLive({ pipeMux: $event })"
                  />
                </div>

                <div class="flex items-center justify-between">
                  <div class="space-y-0.5">
                    <Label>通过音频修复 VTT</Label>
                  </div>
                  <Switch
                    :checked="settings.live.fixVttByAudio"
                    @update:checked="updateLive({ fixVttByAudio: $event })"
                  />
                </div>

                <Separator />

                <div class="grid gap-2">
                  <Label>录制限制</Label>
                  <Input
                    :model-value="settings.live.recordLimit"
                    @update:model-value="updateLive({ recordLimit: $event })"
                    placeholder="例如: 1:30:00 (1小时30分钟)"
                    class="flex-1"
                  />
                </div>

                <div class="grid gap-2">
                  <Label>等待时间 (秒)</Label>
                  <Input
                    type="number"
                    :model-value="settings.live.waitTime"
                    @update:model-value="updateLive({ waitTime: parseInt($event) || 0 })"
                    :min="0"
                    class="w-24"
                  />
                </div>

                <div class="grid gap-2">
                  <Label>获取分片数</Label>
                  <Input
                    type="number"
                    :model-value="settings.live.takeCount"
                    @update:model-value="updateLive({ takeCount: parseInt($event) || 0 })"
                    :min="0"
                    class="w-24"
                  />
                </div>
              </CardContent>
            </Card>
          </TabsContent>

          <!-- 高级设置 -->
          <TabsContent value="advanced" class="mt-0 space-y-4">
            <Card>
              <CardHeader>
                <CardTitle class="text-base">工具路径</CardTitle>
                <CardDescription>配置外部工具的路径</CardDescription>
              </CardHeader>
              <CardContent class="space-y-4">
                <div class="grid gap-2">
                  <Label>FFmpeg 路径</Label>
                  <Input
                    :model-value="settings.advanced.ffmpegPath"
                    @update:model-value="updateAdvanced({ ffmpegPath: $event })"
                    placeholder="留空则使用系统 PATH"
                    class="flex-1"
                  />
                </div>

                <div class="grid gap-2">
                  <Label>N_m3u8DL-RE 路径</Label>
                  <Input
                    :model-value="settings.advanced.n_m3u8dlPath"
                    @update:model-value="updateAdvanced({ n_m3u8dlPath: $event })"
                    placeholder="留空则使用系统 PATH"
                    class="flex-1"
                  />
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle class="text-base">日志设置</CardTitle>
              </CardHeader>
              <CardContent class="space-y-4">
                <div class="grid gap-2">
                  <Label>日志级别</Label>
                  <Select
                    :model-value="settings.advanced.logLevel"
                    @update:model-value="updateAdvanced({ logLevel: $event as any })"
                  >
                    <SelectTrigger class="w-full">
                      <SelectValue placeholder="选择级别" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem
                        v-for="option in logLevelOptions"
                        :key="option.value"
                        :value="option.value"
                      >
                        {{ option.label }}
                      </SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                <div class="grid gap-2">
                  <Label>日志文件路径</Label>
                  <Input
                    :model-value="settings.advanced.logFilePath"
                    @update:model-value="updateAdvanced({ logFilePath: $event })"
                    placeholder="留空则不写入文件"
                    class="flex-1"
                  />
                </div>

                <div class="flex items-center justify-between">
                  <div class="space-y-0.5">
                    <Label>禁用日志</Label>
                  </div>
                  <Switch
                    :checked="settings.advanced.noLog"
                    @update:checked="updateAdvanced({ noLog: $event })"
                  />
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle class="text-base">实验性功能</CardTitle>
              </CardHeader>
              <CardContent class="space-y-3">
                <div class="flex items-center justify-between">
                  <div class="space-y-0.5">
                    <Label>允许多 EXT-X-MAP</Label>
                    <p class="text-xs text-muted-foreground">允许 HLS 多个 EXT-X-MAP 标签</p>
                  </div>
                  <Switch
                    :checked="settings.advanced.allowHlsMultiExtMap"
                    @update:checked="updateAdvanced({ allowHlsMultiExtMap: $event })"
                  />
                </div>

                <div class="flex items-center justify-between">
                  <div class="space-y-0.5">
                    <Label>禁用更新检查</Label>
                  </div>
                  <Switch
                    :checked="settings.advanced.disableUpdateCheck"
                    @update:checked="updateAdvanced({ disableUpdateCheck: $event })"
                  />
                </div>

                <div class="grid gap-2">
                  <Label>URL 处理器参数</Label>
                  <Input
                    :model-value="settings.advanced.urlProcessorArgs"
                    @update:model-value="updateAdvanced({ urlProcessorArgs: $event })"
                    placeholder="传递给 URL 处理器的额外参数"
                    class="flex-1"
                  />
                </div>
              </CardContent>
            </Card>
          </TabsContent>

          <!-- UI 设置 -->
          <TabsContent value="ui" class="mt-0 space-y-4">
            <Card>
              <CardHeader>
                <CardTitle class="text-base">外观</CardTitle>
                <CardDescription>自定义应用程序外观</CardDescription>
              </CardHeader>
              <CardContent class="space-y-4">
                <div class="grid gap-2">
                  <Label>主题</Label>
                  <Select :model-value="theme" @update:model-value="setTheme($event as any)">
                    <SelectTrigger class="w-full">
                      <SelectValue placeholder="选择主题" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem
                        v-for="option in themeOptions"
                        :key="option.value"
                        :value="option.value"
                      >
                        {{ option.label }}
                      </SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                <div class="flex items-center justify-between">
                  <div class="space-y-0.5">
                    <Label>显示通知</Label>
                    <p class="text-xs text-muted-foreground">下载完成时显示系统通知</p>
                  </div>
                  <Switch
                    :checked="settings.ui.showNotification"
                    @update:checked="
                      updateAdvanced({} as any);
                      settings.ui.showNotification = $event;
                    "
                  />
                </div>

                <div class="flex items-center justify-between">
                  <div class="space-y-0.5">
                    <Label>剪贴板监视</Label>
                    <p class="text-xs text-muted-foreground">自动检测剪贴板中的 M3U8 链接</p>
                  </div>
                  <Switch
                    :checked="settings.ui.clipboardWatch"
                    @update:checked="
                      updateAdvanced({} as any);
                      settings.ui.clipboardWatch = $event;
                    "
                  />
                </div>
              </CardContent>
            </Card>
          </TabsContent>
        </div>
      </ScrollArea>
    </Tabs>
  </div>
</template>
