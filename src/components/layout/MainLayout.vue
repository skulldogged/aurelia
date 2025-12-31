<script setup lang="ts">
  import { isMobile } from '@/lib/platform'

  import MainLayoutDesktop from './desktop/MainLayout.vue'
  import MainLayoutMobile from './mobile/MainLayout.vue'

  defineProps<{
    navigationState: {
      canGoBack:    boolean
      canGoForward: boolean
      currentView:  string
    }
    playerState: {
      hasPlayer:       boolean
      isEqualizerOpen: boolean
      isLyricsOpen:    boolean
      isQueueOpen:     boolean
    }
    transitionAfterLeave?:  boolean
    transitionBeforeEnter?: boolean
  }>()

  const emit = defineEmits<{
    'global-search':    []
    'logout':           []
    'navigate':         [view: string]
    'navigate-back':    []
    'navigate-forward': []
  }>()
</script>

<template>
  <component
    :is='isMobile() ? MainLayoutMobile : MainLayoutDesktop'
    @global-search="emit('global-search')"
    @logout="emit('logout')"
    @navigate="(view: string) => emit('navigate', view)"
    @navigate-back="emit('navigate-back')"
    @navigate-forward="emit('navigate-forward')"
    :navigation-state='navigationState'
    :player-state='playerState'
    :transition-after-leave='transitionAfterLeave'
    :transition-before-enter='transitionBeforeEnter'
  >
    <template #default>
      <slot />
    </template>
    <template #search-results>
      <slot name='search-results' />
    </template>
    <template #top-bar>
      <slot name='top-bar' />
    </template>
    <template #queue>
      <slot name='queue' />
    </template>
    <template #player>
      <slot name='player' />
    </template>
  </component>
</template>