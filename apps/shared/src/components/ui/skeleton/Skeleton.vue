<script setup lang="ts">
import type { HTMLAttributes } from "vue"
import { cn } from "../../../lib/utils"

interface SkeletonProps {
  class?: HTMLAttributes["class"]
  shimmer?: boolean
}

const props = withDefaults(defineProps<SkeletonProps>(), {
  shimmer: true
})
</script>

<template>
  <div
    data-slot="skeleton"
    :class="cn(
      'rounded-md bg-muted/60',
      props.shimmer ? 'skeleton-shimmer' : 'animate-pulse',
      props.class
    )"
  />
</template>

<style scoped>
.skeleton-shimmer {
  background: linear-gradient(
    90deg,
    var(--muted) 0%,
    var(--muted) 40%,
    color-mix(in srgb, var(--muted) 60%, var(--foreground) 8%) 50%,
    var(--muted) 60%,
    var(--muted) 100%
  );
  background-size: 200% 100%;
  animation: shimmer 1.8s ease-in-out infinite;
}

@keyframes shimmer {
  0% {
    background-position: 200% 0;
  }
  100% {
    background-position: -200% 0;
  }
}
</style>
