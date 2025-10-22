<script setup lang="ts">
  import { ArrowLeft, ArrowRight, PanelLeft, Search } from 'lucide-vue-next'

  import Button from '@/components/ui/Button.vue'

  defineProps<{
    canGoBack:    boolean
    canGoForward: boolean
  }>()

  const emit = defineEmits<{
    'global-search':    []
    'navigate-back':    []
    'navigate-forward': []
    'toggle-sidebar':   []
  }>()
</script>

<template>
  <header
    class='relative bg-sidebar shrink-0 h-12 z-50 flex justify-between items-center pr-[138px]'
    data-tauri-drag-region
  >
    <div class='flex items-center gap-2 pl-2 h-full'>
      <Button
        @click="emit('toggle-sidebar')"
        class='w-12 h-10'
        size='icon'
        variant='ghost'
      >
        <PanelLeft class='h-4 w-4' />
      </Button>
      <Button
        @click="emit('navigate-back')"
        :disabled='!canGoBack'
        size='icon'
        variant='ghost'
      >
        <ArrowLeft class='h-4 w-4' />
      </Button>
      <Button
        @click="emit('navigate-forward')"
        :disabled='!canGoForward'
        size='icon'
        variant='ghost'
      >
        <ArrowRight class='h-4 w-4' />
      </Button>
    </div>

    <div class='flex items-center justify-end gap-2 h-full'>
      <Button
        @click="emit('global-search')"
        class='w-12 h-10'
        size='icon'
        variant='ghost'
      >
        <Search class='h-4 w-4' />
      </Button>
    </div>
  </header>
</template>
