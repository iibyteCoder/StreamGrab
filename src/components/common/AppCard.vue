<script setup lang="ts">
/**
 * Card 卡片容器组件
 */

import { computed } from "vue";

interface Props {
  title?: string;
  subtitle?: string;
  padding?: "none" | "sm" | "md" | "lg";
  hoverable?: boolean;
  clickable?: boolean;
  bordered?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  padding: "md",
  hoverable: false,
  clickable: false,
  bordered: true,
});

const emit = defineEmits<{
  (e: "click", event: MouseEvent): void;
}>();

const cardClasses = computed(() => {
  const classes = ["bg-bg-surface rounded-lg overflow-hidden"];

  // 边框
  if (props.bordered) {
    classes.push("border border-border-default");
  }

  // 悬停效果
  if (props.hoverable) {
    classes.push(
      "transition-shadow duration-200 hover:shadow-lg hover:shadow-black/10",
    );
  }

  // 可点击效果
  if (props.clickable) {
    classes.push("cursor-pointer transition-all duration-200");
    classes.push("hover:border-accent-primary/50 hover:bg-bg-elevated");
  }

  return classes;
});

const bodyClasses = computed(() => {
  const paddingClasses = {
    none: "",
    sm: "p-3",
    md: "p-4",
    lg: "p-6",
  };
  return paddingClasses[props.padding];
});

const handleClick = (event: MouseEvent) => {
  if (props.clickable) {
    emit("click", event);
  }
};
</script>

<template>
  <div :class="cardClasses" @click="handleClick">
    <!-- Header slot -->
    <div
      v-if="title || subtitle || $slots.header"
      class="border-b border-border-default px-4 py-3"
    >
      <slot name="header">
        <h3 v-if="title" class="text-base font-medium text-text-primary">
          {{ title }}
        </h3>
        <p v-if="subtitle" class="mt-1 text-sm text-text-secondary">
          {{ subtitle }}
        </p>
      </slot>
    </div>

    <!-- Body -->
    <div :class="bodyClasses">
      <slot />
    </div>

    <!-- Footer slot -->
    <div v-if="$slots.footer" class="border-t border-border-default px-4 py-3">
      <slot name="footer" />
    </div>

    <!-- Actions slot (absolute positioned) -->
    <div v-if="$slots.actions" class="absolute top-3 right-3">
      <slot name="actions" />
    </div>
  </div>
</template>
