<script setup lang="ts">
import Sidebar from './Sidebar.vue'
import TopBar from './TopBar.vue'

defineProps<{
  currentView: string
  canGoBack: boolean
  canGoForward: boolean
}>()

const emit = defineEmits<{
  'navigate': [view: string]
  'navigate-back': []
  'navigate-forward': []
  'logout': []
  'global-search': [query: string]
}>()
</script>

<template>
  <div class="h-screen flex flex-col">
    <div class="flex flex-grow min-h-0">
      <Sidebar :current-view="currentView" :can-go-back="canGoBack" :can-go-forward="canGoForward"
        @navigate="(view) => emit('navigate', view)" @navigate-back="emit('navigate-back')"
        @navigate-forward="emit('navigate-forward')" @logout="emit('logout')" />
      <main class="flex-1 flex flex-col min-w-0">
        <TopBar @global-search="(query) => emit('global-search', query)" />
        <div class="flex-grow overflow-y-auto">
          <slot></slot>
        </div>
      </main>
    </div>
    <div class="flex-shrink-0">
      <slot name="player"></slot>
    </div>
  </div>
</template>
