<script setup lang="ts">
import type { HTMLAttributes, LabelHTMLAttributes } from 'vue'

interface Props extends /* @vue-ignore */ LabelHTMLAttributes {
  for?: string
  class?: HTMLAttributes['class']
}

defineProps<Props>()
</script>

<template>
  <label
    :for='for'
    :class='[
      "text-sm font-medium leading-none select-none",
      "peer-disabled:cursor-not-allowed peer-disabled:opacity-50",
      $props.class
    ]'
  >
    <slot />
  </label>
</template>
