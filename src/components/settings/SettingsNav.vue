<script setup lang="ts">
/**
 * SettingsNav - 设置页侧边导航组件
 */

import * as Icons from "lucide-vue-next";
import { AppIcon } from "@/components/common";

type IconName = keyof typeof Icons;

interface NavItem {
  value: string;
  label: string;
  icon: IconName;
}

const navItems: NavItem[] = [
  { value: "general", label: "常规", icon: "Settings" },
  { value: "templates", label: "模板", icon: "FileText" },
  { value: "download", label: "下载", icon: "Download" },
  { value: "mux", label: "混流", icon: "Video" },
  { value: "network", label: "网络", icon: "Globe" },
  { value: "decryption", label: "解密", icon: "Key" },
  { value: "live", label: "直播", icon: "Radio" },
  { value: "advanced", label: "高级", icon: "Settings2" },
  { value: "ui", label: "界面", icon: "Palette" },
];

interface Props {
  modelValue: string;
}

defineProps<Props>();

const emit = defineEmits<{
  (e: "update:modelValue", value: string): void;
}>();

const handleSelect = (value: string) => {
  emit("update:modelValue", value);
};
</script>

<template>
  <nav class="w-48 shrink-0 border-r bg-muted/30">
    <div class="p-3">
      <ul class="space-y-0.5">
        <li v-for="item in navItems" :key="item.value">
          <button
            :class="[
              'flex w-full items-center gap-3 rounded-md px-3 py-2 text-sm transition-all duration-150',
              modelValue === item.value
                ? 'bg-primary/10 text-primary font-medium'
                : 'text-muted-foreground hover:bg-muted hover:text-foreground',
            ]"
            @click="handleSelect(item.value)"
          >
            <AppIcon :name="item.icon" :size="16" />
            <span>{{ item.label }}</span>
          </button>
        </li>
      </ul>
    </div>
  </nav>
</template>
