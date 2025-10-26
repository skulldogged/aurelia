<script setup lang="ts">
  import { useOrientation } from '@/composables/useOrientation'

  import PlaylistDetailPageDesktop from '../desktop/PlaylistDetailPage.vue'
  import PlaylistDetailPageMobile from '../mobile/PlaylistDetailPage.vue'

  const { isPortrait } = useOrientation()
</script>

<template>
  <component :is='isPortrait ? PlaylistDetailPageMobile : PlaylistDetailPageDesktop' />
</template>
