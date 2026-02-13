<script setup lang="ts">
/**
 * Button 按钮组件
 */

import { computed } from 'vue';

type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'danger';
type ButtonSize = 'sm' | 'md' | 'lg';

interface Props {
  variant?: ButtonVariant;
  size?: ButtonSize;
  disabled?: boolean;
  loading?: boolean;
  block?: boolean;
  type?: 'button' | 'submit' | 'reset';
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'primary',
  size: 'md',
  disabled: false,
  loading: false,
  block: false,
  type: 'button',
});

const emit = defineEmits<{
  (e: 'click', event: MouseEvent): void;
}>();

const buttonClasses = computed(() => {
  const base = [
    'inline-flex items-center justify-center font-medium rounded-lg transition-all duration-150',
    'focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-bg-base',
    'disabled:opacity-50 disabled:cursor-not-allowed',
  ];

  // 尺寸
  const sizeClasses: Record<ButtonSize, string> = {
    sm: 'px-3 py-1.5 text-sm gap-1.5',
    md: 'px-4 py-2 text-sm gap-2',
    lg: 'px-6 py-3 text-base gap-2',
  };

  // 变体
  const variantClasses: Record<ButtonVariant, string> = {
    primary: [
      'bg-accent-primary text-white',
      'hover:bg-accent-primary/90',
      'focus:ring-accent-primary',
    ].join(' '),
    secondary: [
      'bg-bg-surface text-text-primary border border-border-default',
      'hover:bg-bg-elevated hover:border-border-hover',
      'focus:ring-accent-primary',
    ].join(' '),
    ghost: [
      'text-text-secondary',
      'hover:text-text-primary hover:bg-bg-surface',
      'focus:ring-accent-primary',
    ].join(' '),
    danger: [
      'bg-accent-error text-white',
      'hover:bg-accent-error/90',
      'focus:ring-accent-error',
    ].join(' '),
  };

  const classes = [...base, sizeClasses[props.size], variantClasses[props.variant]];

  if (props.block) {
    classes.push('w-full');
  }

  return classes;
});

const handleClick = (event: MouseEvent) => {
  if (!props.disabled && !props.loading) {
    emit('click', event);
  }
};
</script>

<template>
  <button
    :type="type"
    :class="buttonClasses"
    :disabled="disabled || loading"
    @click="handleClick"
  >
    <!-- Loading spinner -->
    <svg
      v-if="loading"
      class="animate-spin h-4 w-4"
      xmlns="http://www.w3.org/2000/svg"
      fill="none"
      viewBox="0 0 24 24"
    >
      <circle
        class="opacity-25"
        cx="12"
        cy="12"
        r="10"
        stroke="currentColor"
        stroke-width="4"
      />
      <path
        class="opacity-75"
        fill="currentColor"
        d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
      />
    </svg>

    <!-- Icon slot -->
    <slot v-if="!loading" name="icon-left" />

    <!-- Content -->
    <span><slot /></span>

    <!-- Icon slot -->
    <slot v-if="!loading" name="icon-right" />
  </button>
</template>
