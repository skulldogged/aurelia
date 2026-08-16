<script setup lang='ts'>
  import { ArrowLeft, ArrowRight, PanelLeft } from 'lucide-vue-next'
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-vue'
  import { computed, ref, watch } from 'vue'

  import { useMainLayout } from '../../../composables/useMainLayout'
  import { getPlatform, Platform } from '../../../lib/platform'
  import Button from '../../ui/Button.vue'
  import Sidebar from '../Sidebar.vue'
  import 'overlayscrollbars/overlayscrollbars.css'

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
  }>()

  const emit = defineEmits<{
    'global-search':    []
    'logout':           []
    'navigate':         [view: string]
    'navigate-back':    []
    'navigate-forward': []
  }>()

  const { mainContentBgClass, rightPanelBgClass, scrollbarsRef, topBarBgClass } = useMainLayout()

  const storedState = localStorage.getItem('sidebarCollapsed')
  const isSidebarCollapsed = ref(storedState ? JSON.parse(storedState) : false)

  const isMacos = computed(() => getPlatform() === Platform.MacOS)

  watch(isSidebarCollapsed, newState => {
    localStorage.setItem('sidebarCollapsed', JSON.stringify(newState))
  })
</script>

<template>
  <div class='h-screen flex flex-col overflow-hidden relative'>
    <!-- Search results overlay -->
    <slot :is-sidebar-collapsed='isSidebarCollapsed' :on-result-click='() => {}' name='search-results' />

    <div
      :class="[
        mainContentBgClass,
        'flex grow min-h-0',
        { 'pb-[88px]': playerState.hasPlayer }
      ]"
    >
      <div
        :class="[
          'main-content flex flex-1 min-w-0',
          isSidebarCollapsed ? (isMacos ? 'ml-[81px]' : 'ml-[65px]') : 'ml-48',
          (
            playerState.isQueueOpen
            || playerState.isEqualizerOpen
            || playerState.isLyricsOpen
          )
            && 'mr-64 lg:mr-80 xl:mr-96 2xl:mr-[448px]',
        ]"
      >
        <!-- Draggable top area -->
        <div
          :style='{ top: `calc(env(safe-area-inset-top))` }'
          class='absolute left-0 right-0 z-5 h-12'
          data-drag-region
        />

        <!-- Navigation buttons positioned relative to sidebar -->
        <div
          :class="[
            'nav-buttons absolute z-10 flex items-center',
            isSidebarCollapsed ? (isMacos ? 'left-[89px]' : 'left-[73px]') : 'left-[200px]'
          ]"
          :style='{ top: `calc(0.5rem + env(safe-area-inset-top))` }'
        >
          <Button
            @click="emit('navigate-back')"
            :disabled='!navigationState.canGoBack'
            size='sm'
            variant='ghost'
          >
            <ArrowLeft class='h-4 w-4' />
          </Button>
          <Button
            @click="emit('navigate-forward')"
            :disabled='!navigationState.canGoForward'
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

        <!-- Top bar content slot - spans from sidebar to right edge -->
        <div
          :class="[
            'top-bar absolute z-5 flex items-center justify-center h-12 overflow-visible',
            topBarBgClass
          ]"
          :style='{
            top: `calc(env(safe-area-inset-top))`,
            left: isSidebarCollapsed ? (isMacos ? "81px" : "65px") : "192px",
            right: "0"
          }'
        >
          <div class='absolute z-10 left-0 right-0 pointer-events-none outer-shadow-bottom' />
          <div
            :class="[
              'top-bar-content relative w-full h-full pr-3',
              {
                'mr-[var(--titlebar-area-right)]': !playerState.isQueueOpen && !playerState.isEqualizerOpen && !playerState.isLyricsOpen,
                'mr-64 lg:mr-80 xl:mr-96 2xl:mr-[448px]':
                  playerState.isQueueOpen || playerState.isEqualizerOpen || playerState.isLyricsOpen,
              }
            ]"
            style='margin-left: 128px;'
          >
            <slot name='top-bar'>
              <!-- Default draggable area when no custom top bar content -->
              <div class='relative w-full h-full'>
                <div class='absolute inset-0 z-0' data-drag-region />
              </div>
            </slot>
          </div>
        </div>
        <main
          :style='{
            marginTop: `calc(3rem + env(safe-area-inset-top))`
          }'
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

    <!-- Desktop sidebar -->
    <Sidebar
      @global-search="$emit('global-search')"
      @logout="$emit('logout')"
      @navigate="(view: string) => emit('navigate', view)"
      :current-view='navigationState.currentView'
      :is-collapsed='isSidebarCollapsed'
      :is-mobile-portrait='false'
      :style='{ paddingTop: `calc(env(safe-area-inset-top))` }'
      class='absolute left-0 top-0 h-full z-30 border-r border-border/20'
    />

    <!-- Queue/Equalizer/Lyrics positioned absolutely on the right -->
    <div
      :class="[
        'right-panel absolute right-0 top-0 h-full z-20 overflow-hidden',
        'w-64 lg:w-80 xl:w-96 2xl:w-[448px]',
        rightPanelBgClass,
        (playerState.isQueueOpen || playerState.isEqualizerOpen || playerState.isLyricsOpen)
          ? 'panel-open'
          : 'panel-closed'
      ]"
      :style='{ paddingTop: `calc(env(safe-area-inset-top))` }'
    >
      <div
        :class="[
          'absolute z-10 pointer-events-none outer-shadow-left shadow-fade',
          (playerState.isQueueOpen || playerState.isEqualizerOpen || playerState.isLyricsOpen)
            ? 'opacity-100'
            : 'opacity-0'
        ]"
        :style='{
          top: `calc(3rem + env(safe-area-inset-top))`,
          bottom: playerState.hasPlayer ? "88px" : "0",
          width: "0.75rem",
          left: "-0.75rem"
        }'
      />
      <div class='h-full'>
        <slot name='queue' />
      </div>
    </div>

    <!-- Player positioned absolutely at bottom -->
    <div
      v-if='playerState.hasPlayer'
      :class="[
        'player-bar absolute z-50 bg-sidebar overflow-visible border-t border-border/20',
        'bottom-0',
        isSidebarCollapsed ? (isMacos ? 'left-[81px]' : 'left-[65px]') : 'left-48',
        (playerState.isQueueOpen || playerState.isEqualizerOpen || playerState.isLyricsOpen)
          ? 'right-64 lg:right-80 xl:right-96 2xl:right-[448px]'
          : 'right-0'
      ]"
    >
      <slot name='player' />
    </div>
  </div>
</template>

<style scoped>
/* Main content area smoothly adjusts with panels */
.main-content {
  transition: margin-left 0.2s ease, margin-right 0.25s cubic-bezier(0.32, 0.72, 0, 1);
  will-change: margin-left, margin-right;
}

/* Smooth slide animation for right panel using transform (GPU-accelerated) */
.right-panel {
  transition: transform 0.25s cubic-bezier(0.32, 0.72, 0, 1),
              visibility 0.25s;
  will-change: transform;
}

.right-panel.panel-closed {
  transform: translateX(100%);
  visibility: hidden;
}

.right-panel.panel-open {
  transform: translateX(0);
  visibility: visible;
}

/* Shadow fades with the same timing as panel */
.shadow-fade {
  transition: opacity 0.25s cubic-bezier(0.32, 0.72, 0, 1);
}

/* Player bar smoothly adjusts with panel */
.player-bar {
  transition: left 0.2s ease, right 0.25s cubic-bezier(0.32, 0.72, 0, 1);
  will-change: left, right;
}

/* Top bar smoothly adjusts with sidebar */
.top-bar {
  transition: left 0.2s ease;
}

/* Top bar content smoothly adjusts with right panel */
.top-bar-content {
  transition: margin-right 0.25s cubic-bezier(0.32, 0.72, 0, 1);
}

/* Navigation buttons smoothly adjust with sidebar */
.nav-buttons {
  transition: left 0.2s ease;
}
</style>
