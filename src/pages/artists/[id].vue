<script setup lang="ts">
  import { useOrientation } from '@/composables/useOrientation'
  import ArtistDetailPageDesktop from '@/pages/desktop/ArtistDetailPage.vue'
  import ArtistDetailPageMobile from '@/pages/mobile/ArtistDetailPage.vue'

  const { isPortrait } = useOrientation()
</script>

<template>
  <component :is='isPortrait ? ArtistDetailPageMobile : ArtistDetailPageDesktop' />
</template>