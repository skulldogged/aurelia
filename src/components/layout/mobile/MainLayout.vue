<script setup lang='ts'>
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-vue'
  import { computed } from 'vue'
  import { ref, watch } from 'vue'
  import { useRoute } from 'vue-router'

  import Sidebar from '@/components/layout/Sidebar.vue'
  import 'overlayscrollbars/overlayscrollbars.css'

  import { useMainLayout } from '@/composables/useMainLayout'
  import { useOrientation } from '@/composables/useOrientation'

  const props = defineProps<{
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

  const route = useRoute()
  const { isLandscape, isPortrait } = useOrientation()
  const { mainContentBgClass, rightPanelBgClass, scrollbarsRef } = useMainLayout()

  const isMobilePortraitMode = computed(() => isPortrait.value)
  const isMobileLandscapeMode = computed(() => isLandscape.value)

  const mobilePaddingTop = ref('0px')
  const pendingPaddingChange = ref<null | string>(null)

  // Update padding with delay to match transition midpoint
  watch(() => route.path, newPath => {
    // Don't update immediately - wait for transition event
    pendingPaddingChange.value = newPath === '/' ? '0px' : 'env(safe-area-inset-top)'
  }, { immediate: true })

  // Apply pending padding change when old page finishes leaving (invisible window)
  watch(() => props.transitionAfterLeave, () => {
    if (pendingPaddingChange.value) {
      mobilePaddingTop.value = pendingPaddingChange.value
      pendingPaddingChange.value = null
    }
  })
</script>

<template>
  <div :class="['h-screen flex flex-col', isMobilePortraitMode ? rightPanelBgClass : '']">
    <!-- Search results overlay -->
    <slot :on-result-click='() => {}' name='search-results' />

    <div
      :class="[
        mainContentBgClass,
        'flex grow min-h-0',
        { 'pb-[88px]': playerState.hasPlayer }
      ]"
    >
      <div
        :class="[
          'flex flex-1 min-w-0',
          {
            'ml-16': isMobileLandscapeMode &&
              !playerState.isQueueOpen && !playerState.isEqualizerOpen && !playerState.isLyricsOpen,
            'ml-16 mr-64 lg:mr-80 xl:mr-96 2xl:mr-[448px]':
              isMobileLandscapeMode &&
              (playerState.isQueueOpen || playerState.isEqualizerOpen || playerState.isLyricsOpen),
          }
        ]"
      >
        <!-- Draggable top area -->
        <div
          :style='{ top: `calc(env(safe-area-inset-top))` }'
          class='absolute left-0 right-0 z-5 h-12'
          data-tauri-drag-region
        />

        <main
          :style='{ paddingTop: mobilePaddingTop }'
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
      :class="[
        'mobile-nav transition-colors duration-200',
        isMobilePortraitMode
          ? 'shrink-0 h-14 border-t border-border/30 bg-sidebar'
          : 'absolute left-0 top-0 h-full w-16 z-30 border-r border-border/30 bg-sidebar'
      ]"
      :current-view='navigationState.currentView'
      :is-collapsed='true'
      :is-mobile-portrait='isMobilePortraitMode'
      :style='isMobilePortraitMode ? { paddingBottom: `env(safe-area-inset-bottom)` } : {}'
    />

    <!-- Queue/Equalizer/Lyrics positioned absolutely on the right -->
    <div
      :class="[
        'absolute right-0 top-0 h-full z-20',
        rightPanelBgClass,
        (playerState.isQueueOpen || playerState.isEqualizerOpen || playerState.isLyricsOpen)
          ? 'border-l border-border/50'
          : ''
      ]"
      :style='{ paddingTop: `calc(env(safe-area-inset-top))` }'
    >
      <slot name='queue' />
    </div>

    <!-- Player positioned absolutely at bottom -->
    <div
      v-if='playerState.hasPlayer'
      :class="[
        'absolute z-30 border-t border-border/30 bg-sidebar transition-all duration-200',
        isMobilePortraitMode ? 'bottom-14' : 'bottom-0',
        {
          'left-0 right-0': isMobilePortraitMode,
          'left-16 right-0':
            isMobileLandscapeMode
            && !playerState.isQueueOpen
            && !playerState.isEqualizerOpen
            && !playerState.isLyricsOpen,
          'left-16 right-64 lg:right-80 xl:right-96 2xl:right-[448px] border-r border-border/30':
            isMobileLandscapeMode
            && (playerState.isQueueOpen || playerState.isEqualizerOpen || playerState.isLyricsOpen),
        }
      ]"
      :style='isMobilePortraitMode ? { bottom: `calc(3.5rem + env(safe-area-inset-bottom))` } : {}'
    >
      <slot name='player' />
    </div>
  </div>
</template>
