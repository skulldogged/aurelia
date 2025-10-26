<script setup lang="ts">
  import type { Credentials, Song } from '@/bindings'

  import { useOrientation } from '@/composables/useOrientation'
  import SongsPageDesktop from '@/pages/desktop/SongsPage.vue'
  import SongsPageMobile from '@/pages/mobile/SongsPage.vue'

  const props = defineProps<{
    credentials: Credentials
  }>()

  const emit = defineEmits<{
    'play-instant-mix': [song: Song]
    'play-song':        [song: Song]
    'toggle-favorite':  [song: Song]
  }>()

  const { isPortrait } = useOrientation()
</script>

<template>
  <component
    :is='isPortrait ? SongsPageMobile : SongsPageDesktop'
    @play-instant-mix='song => emit("play-instant-mix", song)'
    @play-song='song => emit("play-song", song)'
    @toggle-favorite='song => emit("toggle-favorite", song)'
    v-bind='$attrs'
    :credentials='props.credentials'
  />
</template>