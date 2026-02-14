<script setup lang="ts">
/**
 * SettingsView - 设置页面
 * 使用子组件组织不同类别的设置
 * 设置会自动保存，无需手动操作
 */

import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { ArrowLeft, Folder, Download, Video, Network, Key, Radio, Settings2, Palette, FileText } from 'lucide-vue-next';
import { Button } from '@/components/ui/button';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { useSettings, useToast } from '@/composables';
import {
  GeneralSettings,
  DownloadSettings,
  MuxSettings,
  NetworkSettings,
  DecryptionSettings,
  LiveSettings,
  AdvancedSettings,
  UISettings,
} from '@/components/settings/sections';
import { TemplateManager } from '@/components/template';

const router = useRouter();
const {
  settings,
  theme,
  isLoaded,
  resetSettings,
  setTheme,
  updateGeneral,
  updateDownload,
  updateMux,
  updateNetwork,
  updateDecryption,
  updateLive,
  updateAdvanced,
  updateUi,
  enableAutoSave,
} = useSettings();
const toast = useToast();

const activeTab = ref('general');

// 初始化 - 启用自动保存（设置已在 App.vue 中加载）
onMounted(() => {
  // 只有在设置已加载后才启用自动保存
  if (isLoaded.value) {
    enableAutoSave(500);
  }
});

// 重置设置
const handleReset = async () => {
  try {
    await resetSettings();
    toast.success('设置已恢复为默认值');
  } catch (error) {
    toast.error('恢复默认设置失败');
  }
};
</script>

<template>
  <div class="flex h-full flex-col bg-background">
    <!-- 头部 -->
    <header class="border-b p-4">
      <div class="flex items-center gap-3">
        <Button variant="ghost" size="icon" @click="router.push('/')">
          <ArrowLeft class="h-5 w-5" />
        </Button>
        <div>
          <h1 class="text-xl font-semibold">设置</h1>
          <p class="text-xs text-muted-foreground">配置应用程序选项，更改会自动保存</p>
        </div>
      </div>
    </header>

    <!-- 设置内容 -->
    <div class="flex-1 relative overflow-hidden">
      <Tabs v-model="activeTab" class="absolute inset-0 flex flex-col">
        <div class="border-b bg-card shrink-0">
          <TabsList class="h-auto w-full justify-start gap-1 bg-transparent p-2">
            <TabsTrigger value="general" class="gap-2 px-4 py-2 text-sm data-[state=active]:bg-primary/10 data-[state=active]:text-primary">
              <Folder class="h-4 w-4" />
              常规
            </TabsTrigger>
            <TabsTrigger value="templates" class="gap-2 px-4 py-2 text-sm data-[state=active]:bg-primary/10 data-[state=active]:text-primary">
              <FileText class="h-4 w-4" />
              模板
            </TabsTrigger>
            <TabsTrigger value="download" class="gap-2 px-4 py-2 text-sm data-[state=active]:bg-primary/10 data-[state=active]:text-primary">
              <Download class="h-4 w-4" />
              下载
            </TabsTrigger>
            <TabsTrigger value="mux" class="gap-2 px-4 py-2 text-sm data-[state=active]:bg-primary/10 data-[state=active]:text-primary">
              <Video class="h-4 w-4" />
              混流
            </TabsTrigger>
            <TabsTrigger value="network" class="gap-2 px-4 py-2 text-sm data-[state=active]:bg-primary/10 data-[state=active]:text-primary">
              <Network class="h-4 w-4" />
              网络
            </TabsTrigger>
            <TabsTrigger value="decryption" class="gap-2 px-4 py-2 text-sm data-[state=active]:bg-primary/10 data-[state=active]:text-primary">
              <Key class="h-4 w-4" />
              解密
            </TabsTrigger>
            <TabsTrigger value="live" class="gap-2 px-4 py-2 text-sm data-[state=active]:bg-primary/10 data-[state=active]:text-primary">
              <Radio class="h-4 w-4" />
              直播
            </TabsTrigger>
            <TabsTrigger value="advanced" class="gap-2 px-4 py-2 text-sm data-[state=active]:bg-primary/10 data-[state=active]:text-primary">
              <Settings2 class="h-4 w-4" />
              高级
            </TabsTrigger>
            <TabsTrigger value="ui" class="gap-2 px-4 py-2 text-sm data-[state=active]:bg-primary/10 data-[state=active]:text-primary">
              <Palette class="h-4 w-4" />
              界面
            </TabsTrigger>
          </TabsList>
        </div>

        <div class="flex-1 overflow-y-auto">
        <!-- 常规设置 -->
        <TabsContent value="general" class="p-4 mt-0">
          <GeneralSettings :settings="settings" @update:settings="updateGeneral($event)" />
        </TabsContent>

        <!-- 配置模板 -->
        <TabsContent value="templates" class="p-4 mt-0">
          <TemplateManager />
        </TabsContent>

        <!-- 下载设置 -->
        <TabsContent value="download" class="p-4 mt-0">
          <DownloadSettings :settings="settings" @update:settings="updateDownload($event)" />
        </TabsContent>

        <!-- 混流设置 -->
        <TabsContent value="mux" class="p-4 mt-0">
          <MuxSettings :settings="settings" @update:settings="updateMux($event)" />
        </TabsContent>

        <!-- 网络设置 -->
        <TabsContent value="network" class="p-4 mt-0">
          <NetworkSettings :settings="settings" @update:settings="updateNetwork($event)" />
        </TabsContent>

        <!-- 解密设置 -->
        <TabsContent value="decryption" class="p-4 mt-0">
          <DecryptionSettings :settings="settings" @update:settings="updateDecryption($event)" />
        </TabsContent>

        <!-- 直播设置 -->
        <TabsContent value="live" class="p-4 mt-0">
          <LiveSettings :settings="settings" @update:settings="updateLive($event)" />
        </TabsContent>

        <!-- 高级设置 -->
        <TabsContent value="advanced" class="p-4 mt-0">
          <AdvancedSettings :settings="settings" @update:settings="updateAdvanced($event)" @reset="handleReset" />
        </TabsContent>

        <!-- UI 设置 -->
        <TabsContent value="ui" class="p-4 mt-0">
          <UISettings :settings="settings" :theme="theme" @update:settings="updateUi($event)" @update:theme="setTheme($event)" />
        </TabsContent>
        </div>
      </Tabs>
    </div>
  </div>
</template>
