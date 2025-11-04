<script setup lang='ts'>
  import { ArrowLeft, ArrowRight, PanelLeft } from 'lucide-vue-next'
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-vue'
  import { ref, watch } from 'vue'

  import Sidebar from '@/components/layout/Sidebar.vue'
  import Button from '@/components/ui/Button.vue'
  import { useMainLayout } from '@/composables/useMainLayout'
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

  const { mainContentBgClass, rightPanelBgClass, scrollbarsRef } = useMainLayout()

  const storedState = localStorage.getItem('sidebarCollapsed')
  const isSidebarCollapsed = ref(storedState ? JSON.parse(storedState) : false)

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
          {
            'ml-16': isSidebarCollapsed &&
              !playerState.isQueueOpen && !playerState.isEqualizerOpen && !playerState.isLyricsOpen,
            'ml-48': !isSidebarCollapsed &&
              !playerState.isQueueOpen && !playerState.isEqualizerOpen && !playerState.isLyricsOpen,
            'ml-16 mr-64 lg:mr-80 xl:mr-96 2xl:mr-[448px]':
              isSidebarCollapsed &&
              (playerState.isQueueOpen || playerState.isEqualizerOpen || playerState.isLyricsOpen),
            'ml-48 mr-64 lg:mr-80 xl:mr-96 2xl:mr-[448px]':
              !isSidebarCollapsed &&
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

        <!-- Navigation buttons positioned relative to sidebar -->
        <div
          :class="[
            'absolute z-10 flex items-center',
            isSidebarCollapsed ? 'left-[72px]' : 'left-[200px]'
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
          :style='{
            top: `calc(env(safe-area-inset-top))`,
            left: isSidebarCollapsed ? "64px" : "192px",
            right: "0",
            boxShadow: "0 1px 0 0 var(--border)"
          }'
          class='
            absolute z-5 flex items-center justify-center h-12
            bg-background-dark
          '
        >
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
            paddingTop: `calc(3rem + env(safe-area-inset-top))`
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
      class='absolute left-0 top-0 h-full z-30 border-r border-border'
    />

    <!-- Queue/Equalizer/Lyrics positioned absolutely on the right -->
    <div
      :class="[
        'absolute right-0 top-0 h-full z-20',
        rightPanelBgClass,
        (playerState.isQueueOpen || playerState.isEqualizerOpen || playerState.isLyricsOpen)
          ? 'border-l border-border'
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
        'absolute z-30 bg-background-dark border-t border-border',
        'bottom-0',
        {
          'left-16 right-0':
            isSidebarCollapsed
            && !playerState.isQueueOpen
            && !playerState.isEqualizerOpen
            && !playerState.isLyricsOpen,
          'left-48 right-0':
            !isSidebarCollapsed
            && !playerState.isQueueOpen
            && !playerState.isEqualizerOpen
            && !playerState.isLyricsOpen,
          'left-16 right-64 lg:right-80 xl:right-96 2xl:right-[448px] border-r border-border':
            isSidebarCollapsed
            && (playerState.isQueueOpen || playerState.isEqualizerOpen || playerState.isLyricsOpen),
          'left-48 right-64 lg:right-80 xl:right-96 2xl:right-[448px] border-r border-border':
            !isSidebarCollapsed
            && (playerState.isQueueOpen || playerState.isEqualizerOpen || playerState.isLyricsOpen),
        }
      ]"
    >
      <slot name='player' />
    </div>
  </div>
</template>