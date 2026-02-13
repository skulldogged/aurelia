<script setup lang='ts'>
  import { ChevronDown, Mic2 } from 'lucide-vue-next'

  import WindowControls from '../shared/WindowControls.vue'
  import Button from '../ui/Button.vue'

  defineProps<{
    hasLyrics:          boolean
    isDesktop:          boolean
    showLyrics:         boolean
    showWindowControls: boolean
  }>()

  defineEmits<{
    (e: 'close'): void
    (e: 'toggle-lyrics'): void
  }>()
</script>

<template>
  <header class='relative z-30 flex items-center justify-between p-4'>
    <div
      @touchmove.stop
      @touchstart.stop
      class='flex items-center gap-2'
    >
      <Button
        @click="$emit('close')"
        class='fs-control-btn'
        size='icon'
        variant='ghost'
      >
        <ChevronDown class='size-5' />
      </Button>
      <Button
        @click="$emit('toggle-lyrics')"
        :class="['fs-control-btn', showLyrics && 'is-active']"
        :disabled='!hasLyrics'
        size='icon'
        variant='ghost'
      >
        <Mic2 class='size-5' />
      </Button>
    </div>

    <div
      v-if='isDesktop'
      class='absolute inset-0 -z-10'
      data-tauri-drag-region
    />

    <WindowControls
      v-if='showWindowControls'
      class='z-10'
    />
  </header>
</template>
