<script setup lang="ts">
  import { useOrientation } from '@/composables/useOrientation'

  import PlaylistEditPageDesktop from '../desktop/PlaylistEditPage.vue'
  import PlaylistEditPageMobile from '../mobile/PlaylistEditPage.vue'

  const { isPortrait } = useOrientation()
</script>

<template>
  <component :is='isPortrait ? PlaylistEditPageMobile : PlaylistEditPageDesktop' />
</template>