<script setup lang='ts'>
  import { ArrowLeft, ArrowRight, PanelLeft } from 'lucide-vue-next'
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-vue'
  import { computed, nextTick, ref, watch } from 'vue'
  import { useRoute } from 'vue-router'

  import Button from '@/components/ui/Button.vue'
  import { useBlurStore } from '@/stores'

  import Sidebar from './Sidebar.vue'
  import 'overlayscrollbars/overlayscrollbars.css'

  defineProps<{
    canGoBack:       boolean
    canGoForward:    boolean
    currentView:     string
    hasPlayer:       boolean
    isEqualizerOpen: boolean
    isLyricsOpen:    boolean
    isQueueOpen:     boolean
  }>()

  const route = useRoute()
  const scrollbarsRef = ref<InstanceType<typeof OverlayScrollbarsComponent> | null>(null)
  const blurStore = useBlurStore()

  const mainContentBgClass = computed(() => blurStore.selectedBlurMode.name === 'acrylic'
    ? 'bg-sidebar/60'
    : '')

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
    <!-- Sidebar positioned absolutely to extend full height -->
    <Sidebar
      @global-search="(query: string) => emit('global-search', query)"
      @navigate="(view: string) => emit('navigate', view)"
      :current-view='currentView'
      :is-collapsed='isSidebarCollapsed'
      class='absolute left-0 top-0 h-full z-30 border-r border-border/50'
    />

    <!-- Search results overlay -->
    <slot :is-sidebar-collapsed='isSidebarCollapsed' :on-result-click='() => {}' name='search-results' />

    <div :class="[mainContentBgClass, { 'pb-[88px]': hasPlayer }]" class='flex flex-grow min-h-0'>
      <div
        :class="[
          'flex flex-1 min-w-0',
          {
            'ml-[64px]': isSidebarCollapsed && !isQueueOpen && !isEqualizerOpen && !isLyricsOpen,
            'ml-[192px]': !isSidebarCollapsed && !isQueueOpen && !isEqualizerOpen && !isLyricsOpen,
            'ml-[64px] mr-[256px] lg:mr-[320px] xl:mr-[384px] 2xl:mr-[448px]':
              isSidebarCollapsed && (isQueueOpen || isEqualizerOpen || isLyricsOpen),
            'ml-[192px] mr-[256px] lg:mr-[320px] xl:mr-[384px] 2xl:mr-[448px]':
              !isSidebarCollapsed && (isQueueOpen || isEqualizerOpen || isLyricsOpen),
          }
        ]"
      >
        <!-- Draggable top area -->
        <div
          class='absolute top-0 left-0 right-0 z-5 h-12'
          data-tauri-drag-region
        />
        <!-- Navigation buttons positioned relative to sidebar -->
        <div
          :class="[
            'absolute top-2 z-10 flex items-center gap-2',
            isSidebarCollapsed ? 'left-[72px]' : 'left-[200px]'
          ]"
        >
          <Button
            @click="emit('navigate-back')"
            :disabled='!canGoBack'
            size='sm'
            variant='ghost'
          >
            <ArrowLeft class='h-4 w-4' />
          </Button>
          <Button
            @click="emit('navigate-forward')"
            :disabled='!canGoForward'
            size='sm'
            variant='ghost'
          >
            <ArrowRight class='h-4 w-4' />
          </Button>
          <Button
            @click='isSidebarCollapsed = !isSidebarCollapsed'
            size='sm'
            variant='ghost'
          >
            <PanelLeft class='h-4 w-4' />
          </Button>
        </div>
        <main
          class='flex-1 min-w-0 bg-background pt-12'
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
      </div>
    </div>

    <!-- Queue/Equalizer/Lyrics positioned absolutely on the right -->
    <div
      :class="[
        'absolute right-0 top-0 h-full z-20',
        (isQueueOpen || isEqualizerOpen || isLyricsOpen) ? 'border-l border-border/50' : ''
      ]"
    >
      <slot name='queue' />
    </div>

    <!-- Player positioned absolutely at bottom -->
    <div
      v-if='hasPlayer'
      :class="[
        'absolute bottom-0 z-30 border-t border-border/50 bg-sidebar',
        {
          'left-[64px] right-0': isSidebarCollapsed && !isQueueOpen && !isEqualizerOpen && !isLyricsOpen,
          'left-[192px] right-0': !isSidebarCollapsed && !isQueueOpen && !isEqualizerOpen && !isLyricsOpen,
          'left-[64px] right-[256px] lg:right-[320px] xl:right-[384px] 2xl:right-[448px]':
            isSidebarCollapsed && (isQueueOpen || isEqualizerOpen || isLyricsOpen),
          'left-[192px] right-[256px] lg:right-[320px] xl:right-[384px] 2xl:right-[448px]':
            !isSidebarCollapsed && (isQueueOpen || isEqualizerOpen || isLyricsOpen),
        }
      ]"
    >
      <slot name='player' />
    </div>
  </div>
</template>
