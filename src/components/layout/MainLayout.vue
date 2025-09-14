<script setup lang="ts">
  import { ref, watch } from 'vue'
  import Sidebar from './Sidebar.vue'
  import TopBar from './TopBar.vue'
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-vue'
  import 'overlayscrollbars/overlayscrollbars.css'

  defineProps<{
    currentView:     string
    canGoBack:       boolean
    canGoForward:    boolean
    hasPlayer:       boolean
    isQueueOpen:     boolean
    isEqualizerOpen: boolean
  }>()

  const topBar = ref<InstanceType<typeof TopBar> | null>(null)

  const emit = defineEmits<{
    'navigate':         [view: string]
    'navigate-back':    []
    'navigate-forward': []
    'logout':           []
    'global-search':    [query: string]
  }>()

  const storedState = localStorage.getItem('sidebarCollapsed')
  const isSidebarCollapsed = ref(storedState ? JSON.parse(storedState) : false)

  watch(isSidebarCollapsed, newState => {
    localStorage.setItem('sidebarCollapsed', JSON.stringify(newState))
  })

  const onResultClick = () => {
    topBar.value?.clearSearch()
  }
</script>

<template>
  <div class='h-screen flex flex-col relative'>
    <TopBar
      @global-search="(query) => emit('global-search', query)"
      @logout="emit('logout')"
      @navigate-back="emit('navigate-back')"
      @navigate-forward="emit('navigate-forward')"
      @toggle-sidebar='isSidebarCollapsed = !isSidebarCollapsed'
      ref='topBar'
      :can-go-back='canGoBack'
      :can-go-forward='canGoForward'
    />
    <slot :on-result-click='onResultClick' name='search-results' />
    <div class='flex flex-grow min-h-0 bg-sidebar'>
      <Sidebar
        @navigate="(view) => emit('navigate', view)"
        :current-view='currentView'
        :is-collapsed='isSidebarCollapsed'
      />
      <div class='flex flex-1 min-w-0'>
        <main
          :class="[
            'flex-1 min-w-0 bg-background border-l border-t border-black/50',
            hasPlayer ? 'rounded-l-xl border-b' : 'rounded-tl-xl',
            (isQueueOpen || isEqualizerOpen) ? 'border-r rounded-tr-xl rounded-br-xl' : ''
          ]"
        >
          <OverlayScrollbarsComponent :options='{ scrollbars: { autoHide: "scroll" } }' class='h-full' defer>
            <slot />
          </OverlayScrollbarsComponent>
        </main>
        <slot name='queue' />
      </div>
    </div>
    <div class='flex-shrink-0'>
      <slot name='player' />
    </div>
  </div>
</template>
