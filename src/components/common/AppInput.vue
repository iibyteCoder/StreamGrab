<script setup lang="ts">
/**
 * Input 输入框组件
 */

import { computed, ref } from 'vue';

interface Props {
  modelValue?: string;
  type?: 'text' | 'password' | 'url' | 'number';
  placeholder?: string;
  disabled?: boolean;
  readonly?: boolean;
  error?: boolean;
  errorMessage?: string;
  clearable?: boolean;
  maxlength?: number;
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: '',
  type: 'text',
  placeholder: '',
  disabled: false,
  readonly: false,
  error: false,
  clearable: false,
});

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void;
  (e: 'input', event: Event): void;
  (e: 'change', event: Event): void;
  (e: 'focus', event: FocusEvent): void;
  (e: 'blur', event: FocusEvent): void;
  (e: 'clear'): void;
  (e: 'enter', event: KeyboardEvent): void;
}>();

const inputRef = ref<HTMLInputElement>();

const inputClasses = computed(() => {
  const base = [
    'w-full px-3 py-2 rounded-lg border transition-all duration-150',
    'bg-bg-surface text-text-primary placeholder:text-text-muted',
    'focus:outline-none focus:ring-2 focus:ring-accent-primary/50 focus:border-accent-primary',
  ];

  if (props.error) {
    base.push('border-accent-error focus:ring-accent-error/50 focus:border-accent-error');
  } else {
    base.push('border-border-default hover:border-border-hover');
  }

  if (props.disabled) {
    base.push('opacity-50 cursor-not-allowed');
  }

  if (props.clearable && props.modelValue) {
    base.push('pr-10');
  }

  return base;
});

const handleInput = (event: Event) => {
  const value = (event.target as HTMLInputElement).value;
  emit('update:modelValue', value);
  emit('input', event);
};

const handleChange = (event: Event) => {
  emit('change', event);
};

const handleFocus = (event: FocusEvent) => {
  emit('focus', event);
};

const handleBlur = (event: FocusEvent) => {
  emit('blur', event);
};

const handleKeydown = (event: KeyboardEvent) => {
  if (event.key === 'Enter') {
    emit('enter', event);
  }
};

const handleClear = () => {
  emit('update:modelValue', '');
  emit('clear');
  inputRef.value?.focus();
};

const focus = () => {
  inputRef.value?.focus();
};

const blur = () => {
  inputRef.value?.blur();
};

defineExpose({ focus, blur });
</script>

<template>
  <div class="w-full">
    <div class="relative">
      <!-- Prefix slot -->
      <span
        v-if="$slots.prefix"
        class="absolute left-3 top-1/2 -translate-y-1/2 text-text-muted"
      >
        <slot name="prefix" />
      </span>

      <input
        ref="inputRef"
        :type="type"
        :value="modelValue"
        :placeholder="placeholder"
        :disabled="disabled"
        :readonly="readonly"
        :maxlength="maxlength"
        :class="inputClasses"
        @input="handleInput"
        @change="handleChange"
        @focus="handleFocus"
        @blur="handleBlur"
        @keydown="handleKeydown"
      />

      <!-- Clear button -->
      <button
        v-if="clearable && modelValue && !disabled"
        type="button"
        class="absolute right-3 top-1/2 -translate-y-1/2 text-text-muted hover:text-text-primary transition-colors"
        @click="handleClear"
      >
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
            d="M6 18L18 6M6 6l12 12"
          />
        </svg>
      </button>

      <!-- Suffix slot -->
      <span
        v-if="$slots.suffix && !clearable"
        class="absolute right-3 top-1/2 -translate-y-1/2 text-text-muted"
      >
        <slot name="suffix" />
      </span>
    </div>

    <!-- Error message -->
    <p v-if="error && errorMessage" class="mt-1 text-sm text-accent-error">
      {{ errorMessage }}
    </p>
  </div>
</template>
