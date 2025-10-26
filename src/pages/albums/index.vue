<script setup lang="ts">
  import { useOrientation } from '@/composables/useOrientation'
  import AlbumsPageDesktop from '@/pages/desktop/AlbumsPage.vue'
  import AlbumsPageMobile from '@/pages/mobile/AlbumsPage.vue'

  const { isPortrait } = useOrientation()
</script>

<template>
  <component :is='isPortrait ? AlbumsPageMobile : AlbumsPageDesktop' v-bind='$attrs' />
</template>