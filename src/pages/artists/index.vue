<script setup lang="ts">
  import { useOrientation } from '@/composables/useOrientation'
  import ArtistsPageDesktop from '@/pages/desktop/ArtistsPage.vue'
  import ArtistsPageMobile from '@/pages/mobile/ArtistsPage.vue'

  const { isPortrait } = useOrientation()
</script>

<template>
  <component :is='isPortrait ? ArtistsPageMobile : ArtistsPageDesktop' v-bind='$attrs' />
</template>