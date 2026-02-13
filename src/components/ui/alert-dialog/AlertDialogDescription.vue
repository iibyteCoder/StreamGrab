<script setup lang="ts">
import type { AlertDialogDescriptionProps } from "reka-ui"
import { AlertDialogDescription } from "reka-ui"
import { cn } from '@/lib/utils'

const props = defineProps<AlertDialogDescriptionProps & { class?: string }>()
</script>

<template>
  <AlertDialogDescription
    v-bind="props"
    :class="cn('text-sm text-muted-foreground', props.class)"
  >
    <slot />
  </AlertDialogDescription>
</template>
