<script setup lang="ts">
/**
 * SettingsGroup - 设置分组卡片
 *
 * 单卡片模型：标题位于卡片内顶部，内容以发丝线分隔（divide-y）。
 * 行组件自带垂直节奏（padded 模式 px-5 py-4），组间距由父级 space-y-* 控制。
 */

interface Props {
  title: string;
  description?: string;
}

defineProps<Props>();
</script>

<template>
  <section
    class="overflow-hidden rounded-xl border border-border/60 bg-card/60"
  >
    <!-- 分组标题（卡片内顶部） -->
    <div class="border-b border-border/40 px-5 py-3.5">
      <h3 class="text-sm font-medium text-foreground">{{ title }}</h3>
      <p
        v-if="description"
        class="mt-0.5 text-xs leading-relaxed text-muted-foreground"
      >
        {{ description }}
      </p>
    </div>

    <!-- 分组内容（行间发丝分隔） -->
    <div class="divide-y divide-border/50">
      <slot />
    </div>
  </section>
</template>
