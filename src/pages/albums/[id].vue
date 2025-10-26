<script setup lang="ts">
  import { useOrientation } from '@/composables/useOrientation'
  import AlbumDetailPageDesktop from '@/pages/desktop/AlbumDetailPage.vue'
  import AlbumDetailPageMobile from '@/pages/mobile/AlbumDetailPage.vue'

  const { isPortrait } = useOrientation()
</script>

<template>
  <component :is='isPortrait ? AlbumDetailPageMobile : AlbumDetailPageDesktop' />
</template>
