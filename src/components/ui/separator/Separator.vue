<script setup lang="ts">
import type { HTMLAttributes } from "vue";
import { Separator as RekaSeparator } from "reka-ui";
import { cn } from "@/lib/utils";

const props = defineProps<{
  orientation?: "horizontal" | "vertical";
  decorative?: boolean;
  class?: HTMLAttributes["class"];
}>();
</script>

<template>
  <RekaSeparator
    :orientation="orientation"
    :decorative="decorative"
    :class="
      cn(
        'shrink-0 bg-border',
        orientation === 'vertical' ? 'h-full w-[1px]' : 'h-[1px] w-full',
        props.class,
      )
    "
  />
</template>
