<script setup lang='ts'>
  import { ArrowLeft, ArrowRight, PanelLeft } from 'lucide-vue-next'
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-vue'
  import { computed, nextTick, ref, watch } from 'vue'
  import { useRoute } from 'vue-router'

  import Button from '@/components/ui/Button.vue'
  import { useOrientation } from '@/composables/useOrientation'
  import { isMobile } from '@/lib/platform'
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

  const { isLandscape, isPortrait } = useOrientation()

  const mainContentBgClass = computed(() => blurStore.selectedBlurMode.name === 'acrylic'
    ? 'bg-sidebar/60'
    : '')

  const rightPanelBgClass = computed(
    () => blurStore.selectedBlurMode.name !== 'none'
      ? 'bg-transparent'
      : 'bg-background-dark',
  )

  const isMobilePortraitMode = computed(() => isPortrait.value)
  const isMobileLandscapeMode = computed(() => isLandscape.value)

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
  <div :class="['h-screen flex flex-col', isMobilePortraitMode ? rightPanelBgClass : '']">
    <!-- Search results overlay -->
    <slot :is-sidebar-collapsed='isSidebarCollapsed' :on-result-click='() => {}' name='search-results' />

    <div
      :class="[
        mainContentBgClass,
        'flex flex-grow min-h-0',
        { 'pb-[88px]': hasPlayer }
      ]"
    >
      <div
        :class="[
          'flex flex-1 min-w-0',
          isMobilePortraitMode ? {} : {
            'ml-[64px]': (isSidebarCollapsed || isMobileLandscapeMode) &&
              !isQueueOpen && !isEqualizerOpen && !isLyricsOpen,
            'ml-[192px]': !isSidebarCollapsed && !isMobileLandscapeMode &&
              !isQueueOpen && !isEqualizerOpen && !isLyricsOpen,
            'ml-[64px] mr-[256px] lg:mr-[320px] xl:mr-[384px] 2xl:mr-[448px]':
              (isSidebarCollapsed || isMobileLandscapeMode) &&
              (isQueueOpen || isEqualizerOpen || isLyricsOpen),
            'ml-[192px] mr-[256px] lg:mr-[320px] xl:mr-[384px] 2xl:mr-[448px]':
              !isSidebarCollapsed && !isMobileLandscapeMode &&
              (isQueueOpen || isEqualizerOpen || isLyricsOpen),
          }
        ]"
      >
        <!-- Draggable top area -->
        <div
          :style='{ top: `calc(env(safe-area-inset-top))` }'
          class='absolute left-0 right-0 z-5 h-12'
          data-tauri-drag-region
        />
        <!-- Navigation buttons positioned relative to sidebar -->
        <div
          v-if='!isMobile()'
          :class="[
            'absolute z-10 flex items-center gap-2',
            isSidebarCollapsed ? 'left-[72px]' : 'left-[200px]'
          ]"
          :style='{ top: `calc(0.5rem + env(safe-area-inset-top))` }'
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
          :style='!isMobile()
            ? { paddingTop: `calc(3rem + env(safe-area-inset-top))` }
            : { paddingTop: `env(safe-area-inset-top)` }'
          class='flex-1 min-w-0 bg-background'
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

    <!-- Mobile bottom bar (portrait) or left bar (landscape) -->
    <Sidebar
      @global-search="$emit('global-search')"
      @navigate="(view: string) => emit('navigate', view)"
      v-if='isMobile()'
      :class="isMobilePortraitMode
        ? 'flex-shrink-0 h-16 border-t border-border/50'
        : 'absolute left-0 top-0 h-full w-16 z-30 border-l border-border/50'"
      :current-view='currentView'
      :is-collapsed='true'
      :is-mobile-portrait='isMobilePortraitMode'
      :style='isMobilePortraitMode ? { marginBottom: `env(safe-area-inset-bottom)` } : {}'
    />

    <!-- Desktop sidebar -->
    <Sidebar
      @global-search="$emit('global-search')"
      @navigate="(view: string) => emit('navigate', view)"
      v-if='!isMobile()'
      :class="[
        'absolute left-0 top-0 h-full z-30 border-r border-border/50'
      ]"
      :current-view='currentView'
      :is-collapsed='isSidebarCollapsed'
      :is-mobile-portrait='false'
      :style='{ paddingTop: `env(safe-area-inset-top)` }'
    />

    <!-- Queue/Equalizer/Lyrics positioned absolutely on the right -->
    <div
      :class="[
        'absolute right-0 top-0 h-full z-20',
        rightPanelBgClass,
        (isQueueOpen || isEqualizerOpen || isLyricsOpen) ? 'border-l border-border/50' : ''
      ]"
      :style='{ paddingTop: `env(safe-area-inset-top)` }'
    >
      <slot name='queue' />
    </div>

    <!-- Player positioned absolutely at bottom -->
    <div
      v-if='hasPlayer'
      :class="[
        'absolute z-30 border-t border-border/50 bg-sidebar',
        isMobilePortraitMode ? 'bottom-[4rem]' : 'bottom-0',
        {
          'left-0 right-0': isMobilePortraitMode,
          'left-[64px] right-0':
            (!isMobilePortraitMode && isSidebarCollapsed && !isQueueOpen && !isEqualizerOpen && !isLyricsOpen) ||
            (isMobileLandscapeMode && !isQueueOpen && !isEqualizerOpen && !isLyricsOpen),
          'left-[192px] right-0':
            !isMobilePortraitMode && !isSidebarCollapsed && !isMobileLandscapeMode &&
            !isQueueOpen && !isEqualizerOpen && !isLyricsOpen,
          'left-[64px] right-[256px] lg:right-[320px] xl:right-[384px] 2xl:right-[448px] border-r border-border/50':
            (!isMobilePortraitMode && isSidebarCollapsed && (isQueueOpen || isEqualizerOpen || isLyricsOpen)) ||
            (isMobileLandscapeMode && (isQueueOpen || isEqualizerOpen || isLyricsOpen)),
          'left-[192px] right-[256px] lg:right-[320px] xl:right-[384px] 2xl:right-[448px] border-r border-border/50':
            !isMobilePortraitMode && !isSidebarCollapsed && !isMobileLandscapeMode &&
            (isQueueOpen || isEqualizerOpen || isLyricsOpen),
        }
      ]"
      :style='isMobilePortraitMode ? { bottom: `calc(4rem + env(safe-area-inset-bottom))` } : {}'
    >
      <slot name='player' />
    </div>
  </div>
</template>
