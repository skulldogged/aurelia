<script setup lang='ts'>
  import { ChevronDown, Mic2 } from 'lucide-vue-next'

  import Button from '../ui/Button.vue'

  defineProps<{
    hasLyrics:  boolean
    isDesktop:  boolean
    showLyrics: boolean
  }>()

  defineEmits<{
    (e: 'close'): void
    (e: 'toggle-lyrics'): void
  }>()
</script>

<template>
  <header
    class='relative z-30 flex items-center py-4 pl-[max(1rem,var(--titlebar-area-left))] pr-[max(1rem,var(--titlebar-area-right))]'
  >
    <div
      @touchmove.stop
      @touchstart.stop
      class='flex items-center gap-2'
      data-no-drag
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
      class='min-w-0 flex-1 self-stretch'
      data-drag-region
    />
    <div v-else class='min-w-0 flex-1' />
  </header>
</template>
