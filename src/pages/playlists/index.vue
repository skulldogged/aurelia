<script setup lang="ts">
  import { useOrientation } from '@/composables/useOrientation'

  import PlaylistsPageDesktop from '../desktop/PlaylistsPage.vue'
  import PlaylistsPageMobile from '../mobile/PlaylistsPage.vue'

  const { isPortrait } = useOrientation()
</script>

<template>
  <component :is='isPortrait ? PlaylistsPageMobile : PlaylistsPageDesktop' />
</template>
