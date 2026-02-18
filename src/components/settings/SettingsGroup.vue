<script setup lang="ts">
/**
 * SettingsGroup - 设置分组组件
 * 用于将相关设置项组织在一起
 */

interface Props {
  title: string;
  description?: string;
}

defineProps<Props>();
</script>

<template>
  <section class="mb-8 last:mb-0">
    <!-- 分组标题 -->
    <div class="mb-4">
      <h3 class="text-sm font-medium text-foreground">{{ title }}</h3>
      <p v-if="description" class="mt-1 text-xs text-muted-foreground">
        {{ description }}
      </p>
    </div>

    <!-- 分组内容 -->
    <div class="space-y-5 rounded-lg border bg-card/50 p-4">
      <slot />
    </div>
  </section>
</template>
