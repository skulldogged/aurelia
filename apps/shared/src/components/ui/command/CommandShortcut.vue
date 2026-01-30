<script setup lang="ts">
import type { HTMLAttributes } from "vue"
import { cn } from "../../../lib/utils"

const props = defineProps<{
  class?: HTMLAttributes["class"]
}>()
</script>

<template>
  <kbd :class="cn('ml-auto inline-flex h-5 min-w-5 items-center justify-center rounded bg-muted/50 px-1.5 font-mono text-[10px] font-medium text-muted-foreground', props.class)">
    <slot />
  </kbd>
</template>
