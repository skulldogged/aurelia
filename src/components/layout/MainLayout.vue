<script setup lang="ts">
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-vue'
  import { nextTick, ref, watch } from 'vue'
  import { useRoute } from 'vue-router'

  import Sidebar from './Sidebar.vue'
  import TopBar from './TopBar.vue'
  import 'overlayscrollbars/overlayscrollbars.css'

  defineProps<{
    canGoBack:       boolean
    canGoForward:    boolean
    currentView:     string
    hasPlayer:       boolean
    isEqualizerOpen: boolean
    isQueueOpen:     boolean
  }>()

  const route = useRoute()
  const topBar = ref<InstanceType<typeof TopBar> | null>(null)
  const scrollbarsRef = ref<InstanceType<typeof OverlayScrollbarsComponent> | null>(null)

  const emit = defineEmits<{
    'global-search':    [query: string]
    'logout':           []
    'navigate':         [view: string]
    'navigate-back':    []
    'navigate-forward': []
  }>()

  const storedState = localStorage.getItem('sidebarCollapsed')
  const isSidebarCollapsed = ref(storedState ? JSON.parse(storedState) : false)

  watch(isSidebarCollapsed, newState => {
    localStorage.setItem('sidebarCollapsed', JSON.stringify(newState))
  })

  // Scroll to top when route changes
  watch(() => route.path, async () => {
    await nextTick()
    // Wait for the page transition animation to complete (0.1s + small buffer)
    setTimeout(() => {
      const osInstance = scrollbarsRef.value?.osInstance?.()
      if (osInstance) {
        const elements = osInstance.elements()
        if (elements.scrollOffsetElement)
          elements.scrollOffsetElement.scrollTop = 0
      }
    }, 100) // 150ms to account for 0.1s transition + buffer
  })
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
    <slot :on-result-click='topBar?.clearSearch' name='search-results' />
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
          <OverlayScrollbarsComponent
            ref='scrollbarsRef'
            :options='{ scrollbars: { autoHide: "scroll" } }'
            class='h-full'
            defer
          >
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
