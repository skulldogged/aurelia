<script setup lang="ts">
  import {
    ArrowLeft,
    ArrowRight,
    PanelLeft,
    Search,
  } from 'lucide-vue-next'
  import { ref } from 'vue'

  import Button from '@/components/ui/Button.vue'
  import { Input } from '@/components/ui/input'

  defineProps<{
    canGoBack:    boolean
    canGoForward: boolean
  }>()

  const globalSearchQuery = ref('')

  const emit = defineEmits<{
    'global-search':    [query: string]
    'logout':           []
    'navigate-back':    []
    'navigate-forward': []
    'toggle-sidebar':   []
  }>()

  const handleGlobalSearch = (): void => {
    emit('global-search', globalSearchQuery.value)
  }

  const handleSearchFocus = (): void => {
    if (globalSearchQuery.value.trim())
      emit('global-search', globalSearchQuery.value)
  }

  const clearSearch = (): void => {
    globalSearchQuery.value = ''
    emit('global-search', '')
  }

  defineExpose({ clearSearch })
</script>

<template>
  <header
    class='relative bg-sidebar flex-shrink-0 h-12 z-50 flex justify-between items-center pr-[138px]'
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
    </div>

    <div class='absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2'>
      <div class='relative w-96'>
        <Search class='absolute left-2 top-1/2 transform -translate-y-1/2 h-4 w-4 text-foreground/60' />
        <Input
          @focus='handleSearchFocus'
          @input='handleGlobalSearch'
          v-model='globalSearchQuery'
          class='
            pl-8 h-9 bg-transparent border-0 text-foreground
            placeholder:text-muted-foreground focus-visible:ring-1
            focus-visible:ring-accent w-full
          '
          placeholder='Search music...'
        />
      </div>
    </div>

    <div class='flex items-center justify-end gap-2 h-full'>
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
  </header>
</template>
