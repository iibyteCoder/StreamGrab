<script setup lang="ts">
import type { AlertDialogActionProps } from "reka-ui"
import { AlertDialogAction } from "reka-ui"
import { cn } from '@/lib/utils'
import { buttonVariants } from '@/components/ui/button'

const props = defineProps<AlertDialogActionProps>()
</script>

<template>
  <AlertDialogAction
    v-bind="props"
    :class="cn(buttonVariants(), props.class)"
  >
    <slot />
  </AlertDialogAction>
</template>
