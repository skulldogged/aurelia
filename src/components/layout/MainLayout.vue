<script setup lang='ts'>
  import { useMediaQuery } from '@vueuse/core'
  import { ArrowLeft, ArrowRight, PanelLeft } from 'lucide-vue-next'
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-vue'
  import { computed, nextTick, ref, watch } from 'vue'
  import { useRoute } from 'vue-router'

  import Button from '@/components/ui/Button.vue'
  import { useBlurStore } from '@/stores'

  import Sidebar from './Sidebar.vue'
  import 'overlayscrollbars/overlayscrollbars.css'

  const props = defineProps<{
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
    'global-search':    []
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

  const isMobile = useMediaQuery('(max-width: 768px)')
  const isSidebarOpen = ref(false)

  const queueActive = computed(() =>
    props.isQueueOpen || props.isEqualizerOpen || props.isLyricsOpen,
  )

  watch(isMobile, value => {
    if (!value)
      isSidebarOpen.value = false
  })

  const sidebarClasses = computed(() => {
    if (isMobile.value) {
      return [
        'fixed left-0 top-0 h-full z-40 border-r border-border/50 transition-transform duration-200',
        'bg-sidebar/95 backdrop-blur-xl',
        isSidebarOpen.value ? 'translate-x-0 pointer-events-auto' : '-translate-x-full pointer-events-none',
      ]
    }

    return 'absolute left-0 top-0 h-full z-30 border-r border-border/50'
  })

  const mainSpacingClasses = computed(() => {
    if (isMobile.value)
      return 'ml-0 mr-0'

    if (queueActive.value) {
      return isSidebarCollapsed.value
        ? 'ml-[64px] mr-[256px] lg:mr-[320px] xl:mr-[384px] 2xl:mr-[448px]'
        : 'ml-[192px] mr-[256px] lg:mr-[320px] xl:mr-[384px] 2xl:mr-[448px]'
    }

    return isSidebarCollapsed.value ? 'ml-[64px]' : 'ml-[192px]'
  })

  const navOffsetClass = computed(() => {
    if (isMobile.value)
      return 'left-4'

    return isSidebarCollapsed.value ? 'left-[72px]' : 'left-[200px]'
  })

  const playerSpacingClasses = computed(() => {
    if (isMobile.value)
      return 'left-0 right-0'

    if (queueActive.value) {
      return isSidebarCollapsed.value
        ? 'left-[64px] right-[256px] lg:right-[320px] xl:right-[384px] 2xl:right-[448px]'
        : 'left-[192px] right-[256px] lg:right-[320px] xl:right-[384px] 2xl:right-[448px]'
    }

    return isSidebarCollapsed.value ? 'left-[64px] right-0' : 'left-[192px] right-0'
  })

  const queueContainerClasses = computed(() => {
    if (isMobile.value) {
      return queueActive.value
        ? [
          'fixed inset-0 z-40 overflow-y-auto bg-background/95 backdrop-blur-xl',
          'pt-4 pb-[96px] px-4',
        ]
        : ['hidden']
    }

    return [
      'absolute right-0 top-0 h-full z-20',
      queueActive.value ? 'border-l border-border/50' : '',
    ]
  })

  const handleNavigate = (view: string): void => {
    emit('navigate', view)
    if (isMobile.value)
      isSidebarOpen.value = false
  }

  const handleGlobalSearch = (): void => {
    emit('global-search')
    if (isMobile.value)
      isSidebarOpen.value = false
  }

  const toggleSidebar = (): void => {
    if (isMobile.value)
      isSidebarOpen.value = !isSidebarOpen.value
    else
      isSidebarCollapsed.value = !isSidebarCollapsed.value
  }

  // Scroll to top when route changes and close mobile sidebar
  watch(() => route.path, async () => {
    if (isMobile.value)
      isSidebarOpen.value = false

    await nextTick()
    // Wait for the page transition animation to complete (0.1s + small buffer)
    setTimeout(() => {
      const osInstance = scrollbarsRef.value?.osInstance?.()
      if (osInstance) {
        const elements = osInstance.elements()
        if (elements.scrollOffsetElement)
          elements.scrollOffsetElement.scrollTop = 0
      }
    }, 100)
  })
</script>

<template>
  <div class='h-screen flex flex-col relative'>
    <Sidebar
      @global-search='handleGlobalSearch'
      @navigate='handleNavigate'
      :class='sidebarClasses'
      :current-view='currentView'
      :is-collapsed='isMobile ? false : isSidebarCollapsed'
    />

    <div
      @click='isSidebarOpen = false'
      v-if='isMobile && isSidebarOpen'
      class='fixed inset-0 z-30 bg-background/60 backdrop-blur-sm transition-opacity'
    />

    <!-- Search results overlay -->
    <slot :is-sidebar-collapsed='isSidebarCollapsed' :on-result-click='() => {}' name='search-results' />

    <div
      :class="[
        mainContentBgClass,
        { 'pb-[88px]': hasPlayer },
      ]"
      class='flex flex-grow min-h-0 transition-[padding] duration-200'
    >
      <div
        :class="[
          'flex flex-1 min-w-0 transition-all duration-200',
          mainSpacingClasses,
        ]"
      >
        <!-- Draggable top area -->
        <div
          class='absolute top-0 left-0 right-0 z-5 h-12'
          data-tauri-drag-region
        />
        <!-- Navigation buttons positioned relative to sidebar -->
        <div
          :class="['absolute top-2 z-10 flex items-center gap-2', navOffsetClass]"
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
            @click='toggleSidebar'
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
    <div :class='queueContainerClasses'>
      <slot name='queue' />
    </div>

    <!-- Player positioned absolutely at bottom -->
    <div
      v-if='hasPlayer'
      :class="[
        'absolute bottom-0 z-30 border-t border-border/50 bg-sidebar transition-all duration-200',
        playerSpacingClasses,
      ]"
    >
      <slot name='player' />
    </div>
  </div>
</template>
