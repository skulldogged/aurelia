<script setup lang='ts'>
  import { ArrowLeft, ArrowRight, PanelLeft } from 'lucide-vue-next'
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-vue'
  import { computed, ref, watch } from 'vue'

  import Sidebar from '@/components/layout/Sidebar.vue'
  import Button from '@/components/ui/Button.vue'
  import { useMainLayout } from '@/composables/useMainLayout'
  import { getPlatform, Platform } from '@/lib/platform'
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
  <div class='h-screen flex flex-col'>
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
          'flex flex-1 min-w-0',
          isSidebarCollapsed ? (isMacos ? 'ml-20' : 'ml-16') : 'ml-48',
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
          data-tauri-drag-region
        />

        <!-- Navigation buttons positioned relative to sidebar -->
        <div
          :class="[
            'absolute z-10 flex items-center',
            isSidebarCollapsed ? (isMacos ? 'left-[88px]' : 'left-[72px]') : 'left-[200px]'
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
            'absolute z-5 flex items-center justify-center h-12 overflow-visible',
            topBarBgClass
          ]"
          :style='{
            top: `calc(env(safe-area-inset-top))`,
            left: isSidebarCollapsed ? (isMacos ? "80px" : "64px") : "192px",
            right: "0"
          }'
        >
          <div class='absolute z-10 left-0 right-0 pointer-events-none outer-shadow-bottom' />
          <div
            :class="[
              'relative w-full h-full pr-3',
              {
                'mr-[138px]': !playerState.isQueueOpen && !playerState.isEqualizerOpen && !playerState.isLyricsOpen,
                'mr-64 lg:mr-80 xl:mr-96 2xl:mr-[448px]':
                  playerState.isQueueOpen || playerState.isEqualizerOpen || playerState.isLyricsOpen,
              }
            ]"
            style='margin-left: 128px;'
          >
            <slot name='top-bar'>
              <!-- Default draggable area when no custom top bar content -->
              <div class='relative w-full h-full'>
                <div class='absolute inset-0 z-0' data-tauri-drag-region />
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
      @navigate="(view: string) => emit('navigate', view)"
      :current-view='navigationState.currentView'
      :is-collapsed='isSidebarCollapsed'
      :is-mobile-portrait='false'
      :style='{ paddingTop: `calc(env(safe-area-inset-top))` }'
      class='absolute left-0 top-0 h-full z-30'
    />

    <!-- Queue/Equalizer/Lyrics positioned absolutely on the right -->
    <div
      :class="[
        'absolute right-0 top-0 h-full z-20 overflow-visible',
        rightPanelBgClass,
        (playerState.isQueueOpen || playerState.isEqualizerOpen || playerState.isLyricsOpen)
          ? ''
          : 'hidden'
      ]"
      :style='{ paddingTop: `calc(env(safe-area-inset-top))` }'
    >
      <div
        v-if='playerState.isQueueOpen || playerState.isEqualizerOpen || playerState.isLyricsOpen'
        :style='{
          top: `calc(3rem + env(safe-area-inset-top))`,
          bottom: playerState.hasPlayer ? "88px" : "0",
          width: "0.75rem",
          left: "-0.75rem"
        }'
        class='absolute z-10 pointer-events-none outer-shadow-left'
      />
      <slot name='queue' />
    </div>

    <!-- Player positioned absolutely at bottom -->
    <div
      v-if='playerState.hasPlayer'
      :class="[
        'absolute z-30 bg-background-dark overflow-visible',
        'bottom-0',
        isSidebarCollapsed ? (isMacos ? 'left-20' : 'left-16') : 'left-48',
        (playerState.isQueueOpen || playerState.isEqualizerOpen || playerState.isLyricsOpen)
          ? 'right-64 lg:right-80 xl:right-96 2xl:right-[448px]'
          : 'right-0'
      ]"
    >
      <!-- Shadow on top of player -->
      <div class='absolute z-10 left-0 right-0 pointer-events-none outer-shadow-top' />
      <slot name='player' />
    </div>
  </div>
</template>