<script setup lang="ts">
  interface Props {
    class?: string
  }

  defineProps<Props>()
</script>

<template>
  <div
    :class='[$props.class, "absolute inset-0 z-0"]'
    data-tauri-drag-region
  />
</template>