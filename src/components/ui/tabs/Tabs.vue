<script setup lang="ts">
import type { TabsRootEmits, TabsRootProps } from "reka-ui"
import type { HTMLAttributes } from "vue"
import { TabsRoot, useForwardPropsEmits } from "reka-ui"
import { cn } from "@/lib/utils"
import { computed } from "vue"

const props = defineProps<TabsRootProps & { class?: HTMLAttributes["class"] }>()
const emits = defineEmits<TabsRootEmits>()

const forwarded = useForwardPropsEmits(props, emits)

// 合并 class 属性
const mergedProps = computed(() => ({
  ...forwarded.value,
  class: props.class
}))
</script>

<template>
  <TabsRoot v-bind="mergedProps">
    <slot />
  </TabsRoot>
</template>
