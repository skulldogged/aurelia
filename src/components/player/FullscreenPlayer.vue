<script setup lang="ts">
  import { useMediaQuery } from '@vueuse/core'
  import {
    ChevronDown,
    Heart,
    ListMusic,
    Mic2,
    Music2,
    Pause,
    Play,
    Repeat,
    Repeat1,
    Shuffle,
    SkipBack,
    SkipForward,
    Sliders,
    Volume1,
    Volume2,
    VolumeX,
  } from 'lucide-vue-next'
  import { storeToRefs } from 'pinia'
  import { computed, onUnmounted, ref, watch } from 'vue'
  import { PropType } from 'vue'

  import { Song } from '@/bindings'
  import AudioVisualizer from '@/components/player/AudioVisualizer.vue'
  import FullscreenEqualizer from '@/components/player/FullscreenEqualizer.vue'
  import FullscreenQueue from '@/components/player/FullscreenQueue.vue'
  import ImageLoader from '@/components/shared/ImageLoader.vue'
  import LyricsView from '@/components/shared/LyricsView.vue'
  import WindowControls from '@/components/shared/WindowControls.vue'
  import Button from '@/components/ui/Button.vue'
  import { Slider } from '@/components/ui/slider'
  import { useImageLoader } from '@/composables/useImageLoader'
  import { useSwipe } from '@/composables/useSwipe'
  import { logger } from '@/lib/logger'
  import { getPlatform, isMobile, isMobilePortrait, Platform } from '@/lib/platform'
  import { formatDuration, getSongFormatInfo } from '@/lib/utils'
  import { PlayerState, usePlayerStore } from '@/stores'

  const props = defineProps({
    frequencyData: {
      default:  () => new Uint8Array(0),
      required: false,
      type:     Object as PropType<Uint8Array>,
    },
    isEqualizerOpen: {
      default: false,
      type:    Boolean,
    },
    isLyricsOpen: {
      default: false,
      type:    Boolean,
    },
    isQueueOpen: {
      default: false,
      type:    Boolean,
    },
    playerState: {
      required: true,
      type:     Object as PropType<PlayerState>,
    },
    previewProgress: {
      default: null,
      type:    Object as PropType<null | {
        deltaY:    number
        direction: 'down' | 'left' | 'right' | 'up' | null
        startY:    number
      }>,
    },
    serverUrl: {
      default: '',
      type:    String,
    },
    show: {
      required: true,
      type:     Boolean,
    },
    timeDomainData: {
      default:  () => new Uint8Array(0),
      required: false,
      type:     Object as PropType<Uint8Array>,
    },
    token: {
      default: '',
      type:    String,
    },
  })

  const emit = defineEmits<{
    (e: 'close'): void
    (e: 'next-song'): void
    (e: 'previous-song'): void
    (e: 'remove-song', song: Song): void
    (e: 'seek', value: number): void
    (e: 'toggle-equalizer'): void
    (e: 'toggle-favorite', song: Song): void
    (e: 'toggle-lyrics'): void
    (e: 'toggle-mute'): void
    (e: 'toggle-play-pause'): void
    (e: 'toggle-queue'): void
    (e: 'toggle-repeat'): void
    (e: 'toggle-shuffle'): void
    (e: 'update:playlist', playlist: Song[]): void
    (e: 'volume-change', value: number): void
  }>()

  // Reactive state
  const volumePopupRef = ref<HTMLDivElement | null>(null)
  const isVolumePopupVisible = ref(false)
  const isAnimating = ref(false)

  // Composables
  const { getImageUrl } = useImageLoader()
  const playerStore = usePlayerStore()
  const { hasLyrics, visualizerEnabled, visualizerStyle } = storeToRefs(playerStore)
  const { startTracking, stopTracking, swipeProgress, updateTracking } = useSwipe({ maxTime: 300 })

  // Media queries for responsive behavior
  const isLargeScreen = useMediaQuery('(min-width: 1024px)')
  const isMediumScreen = useMediaQuery('(min-width: 768px)')
  const isNarrowScreen = useMediaQuery('(max-width: 480px)')

  // Platform detection
  const isDesktop = computed(() => {
    const current = getPlatform()
    return current !== Platform.Android && current !== Platform.IOS
  })
  const isMobileDevice = computed(() => isMobile())
  const isMobilePortraitMode = computed(() => isMobilePortrait())
  const isMobileLandscapeMode = computed(() => {
    const platform = getPlatform()
    return (platform === Platform.Android || platform === Platform.IOS) &&
      !isMobilePortrait() &&
      window.innerWidth > window.innerHeight
  })

  // Background image
  const backgroundImageData = ref<null | string>(null)

  watch(() => props.playerState.currentSong?.id, async (newId, oldId) => {
    if (newId === oldId) return
    const newSong = props.playerState.currentSong
    if (newSong && props.serverUrl && props.token) {
      try {
        const imageId = newSong.albumId
        if (imageId) {
          const imageData = await getImageUrl(imageId, props.serverUrl, props.token, 'Primary')
          backgroundImageData.value = imageData
        }
      } catch (error) {
        logger.error('Failed to load background image:', error)
        backgroundImageData.value = null
      }
    } else {
      backgroundImageData.value = null
    }
  }, { immediate: true })

  // Computed values
  const formatTime = formatDuration
  const songFormatInfo = computed(() => getSongFormatInfo(props.playerState.currentSong))
  const effectiveVolume = computed(() => props.playerState.isMuted ? 0 : props.playerState.volume)
  const progress = computed(() =>
    props.playerState.duration > 0
      ? (props.playerState.currentTime / props.playerState.duration) * 100
      : 0,
  )

  // Visibility - simplified logic
  const isVisible = computed(() =>
    props.show || (props.previewProgress && props.previewProgress.direction === 'up'),
  )

  // Animation state
  watch(() => props.show, (newVal, oldVal) => {
    if (newVal !== oldVal) {
      isAnimating.value = true
      setTimeout(() => {
        isAnimating.value = false
      }, 400)
    }
  })

  // Swipe handling - simplified
  const swipeOffset = computed(() => {
    // Preview from mini player (swiping up)
    if (props.previewProgress?.direction === 'up') {
      return Math.max(0, props.previewProgress.startY + props.previewProgress.deltaY)
    }
    // Swiping down to close
    if (swipeProgress.value?.direction === 'down') {
      return Math.max(0, swipeProgress.value.deltaY)
    }
    return 0
  })

  const swipeOpacity = computed(() => {
    if (props.previewProgress?.direction === 'up') {
      return Math.min(Math.abs(props.previewProgress.deltaY) / 150, 1)
    }
    if (swipeProgress.value?.direction === 'down') {
      return Math.max(1 - swipeProgress.value.deltaY / 300, 0.2)
    }
    return 1
  })

  const isDragging = computed(() =>
    !!swipeProgress.value?.direction || !!props.previewProgress?.direction,
  )

  const handleSwipeStart = (event: TouchEvent): void => {
    startTracking(event)
  }

  const handleSwipeMove = (event: TouchEvent): void => {
    updateTracking(event)
  }

  const handleSwipeEnd = (event: TouchEvent): void => {
    const result = stopTracking(event)
    if (result?.direction === 'down' && result.isIntentional) {
      emit('close')
    }
  }

  // Lyrics seek handler
  const handleLyricsSeek = (time: number): void => {
    if (props.playerState.duration > 0) {
      const percentage = (time / props.playerState.duration) * 100
      emit('seek', percentage)
    }
  }

  const onLyricsLoaded = (lyricsFound: boolean): void => {
    playerStore.setHasLyrics(lyricsFound)
  }

  // Volume popup handling
  const toggleVolumePopup = (): void => {
    isVolumePopupVisible.value = !isVolumePopupVisible.value
  }

  const handleClickOutside = (event: Event): void => {
    const target = event.target as Element
    if (target.closest('[data-volume-button]')) return
    if (volumePopupRef.value?.contains(target)) return
    isVolumePopupVisible.value = false
  }

  watch(isVolumePopupVisible, visible => {
    if (visible) {
      document.addEventListener('click', handleClickOutside)
    } else {
      document.removeEventListener('click', handleClickOutside)
    }
  })

  onUnmounted(() => {
    document.removeEventListener('click', handleClickOutside)
  })

  // Panel visibility (synced with props, with delayed unmount for animations)
  const showLyrics = computed(() => props.isLyricsOpen)

  // Track last active side panel for delayed unmount
  const lastSidePanel = ref<'equalizer' | 'queue' | null>(null)
  const isSidePanelAnimating = ref(false)

  watch([() => props.isEqualizerOpen, () => props.isQueueOpen], ([eq, queue], [prevEq, prevQueue]) => {
    if (eq && !prevEq) lastSidePanel.value = 'equalizer'
    else if (queue && !prevQueue) lastSidePanel.value = 'queue'

    // Panel just closed
    if (!eq && !queue && (prevEq || prevQueue)) {
      isSidePanelAnimating.value = true
      setTimeout(() => {
        isSidePanelAnimating.value = false
        lastSidePanel.value = null
      }, 250) // Match slide animation duration
    }
  })

  const showEqualizer = computed(
    () =>
      props.isEqualizerOpen
      || (isSidePanelAnimating.value && lastSidePanel.value === 'equalizer'),
  )
  const showQueue = computed(
    () =>
      props.isQueueOpen
      || (isSidePanelAnimating.value && lastSidePanel.value === 'queue'),
  )
  const hasActivePanel = computed(() => props.isEqualizerOpen || props.isQueueOpen)

  // Show secondary controls only on wider screens (not narrow and not mobile portrait)
  const showSecondaryControls = computed(() => !isMobilePortraitMode.value && !isNarrowScreen.value)
</script>

<template>
  <Transition name='fullscreen-player'>
    <div
      @touchend='handleSwipeEnd'
      @touchmove='handleSwipeMove'
      @touchstart='handleSwipeStart'
      v-if='isVisible'
      :class="[
        'fullscreen-player fixed inset-0 z-50 flex flex-col overflow-hidden',
        { 'is-mobile': !isDesktop }
      ]"
      :style="{
        '--swipe-offset': `${swipeOffset}px`,
        '--swipe-opacity': swipeOpacity,
        transform: isDragging ? `translateY(var(--swipe-offset))` : undefined,
        opacity: isDragging ? 'var(--swipe-opacity)' : undefined,
      }"
    >
      <!-- Background layers -->
      <div class='absolute inset-0 z-0'>
        <!-- Album art background -->
        <Transition mode='out-in' name='fade'>
          <div
            v-if='backgroundImageData'
            :key='backgroundImageData'
            :style='{ backgroundImage: `url(${backgroundImageData})` }'
            class='absolute inset-0 bg-cover bg-center scale-110'
          />
        </Transition>
        <!-- Overlay -->
        <div class='absolute inset-0 bg-black/75' />
        <!-- Visualizer -->
        <Transition name='fade'>
          <div
            v-if='visualizerEnabled && frequencyData.length > 0 && playerState.isPlaying'
            :class="[
              'absolute bottom-0 left-0 right-0',
              isMobileDevice ? 'h-64 opacity-50' : 'h-40 opacity-40'
            ]"
          >
            <AudioVisualizer
              :boost='isMobileDevice ? 2.0 : 1.0'
              :frequency-data='frequencyData'
              :is-playing='playerState.isPlaying'
              :style='visualizerStyle'
              :time-domain-data='timeDomainData'
            />
          </div>
        </Transition>
      </div>

      <!-- Top bar with controls -->
      <header
        :style='isMobilePortraitMode ? { paddingTop: `calc(1rem + env(safe-area-inset-top))` } : {}'
        class='relative z-30 flex items-center justify-between p-4'
      >
        <!-- Left controls -->
        <div @touchmove.stop @touchstart.stop class='flex items-center gap-2'>
          <Button
            @click="$emit('close')"
            class='fs-control-btn'
            size='icon'
            variant='ghost'
          >
            <ChevronDown class='size-5' />
          </Button>
          <Button
            @click="$emit('toggle-lyrics')"
            :class="['fs-control-btn', showLyrics && 'is-active']"
            :disabled='!hasLyrics'
            size='icon'
            variant='ghost'
          >
            <Mic2 class='size-5' />
          </Button>
        </div>

        <!-- Draggable region for desktop (between controls) -->
        <div
          v-if='isDesktop'
          class='absolute inset-0 -z-10'
          data-tauri-drag-region
        />

        <!-- Window controls (Windows/Linux) -->
        <WindowControls
          v-if='isDesktop && getPlatform() !== Platform.MacOS'
          class='z-10'
        />
      </header>

      <!-- Main content area -->
      <main class='relative z-10 flex-1 flex overflow-hidden'>
        <!-- Side panel (EQ/Queue) - Desktop only -->
        <aside
          :class="[
            'side-panel shrink-0 overflow-hidden',
            hasActivePanel && !isMobilePortraitMode && !isMobileLandscapeMode
              ? 'side-panel-open'
              : 'w-0'
          ]"
        >
          <div
            class='
              side-panel-content h-full
              bg-background/95 rounded-2xl
              border border-white/10 overflow-hidden shadow-2xl
            '
          >
            <FullscreenEqualizer v-if='showEqualizer' class='h-full' />
            <FullscreenQueue
              @remove-song='$emit("remove-song", $event)'
              @update:playlist='$emit("update:playlist", $event)'
              v-if='showQueue'
              class='h-full'
            />
          </div>
        </aside>

        <!-- Center content -->
        <div
          :class="[
            'center-content flex-1 flex flex-col items-center justify-center',
            'px-4 sm:px-6 md:px-8 lg:px-10',
            (isMobilePortraitMode || isNarrowScreen) ? 'pb-24 sm:pb-28 pt-2 sm:pt-4' : 'py-4 sm:py-6 md:py-8',
          ]"
        >
          <!-- Inner wrapper for album art and controls -->
          <div class='center-items flex flex-col items-center w-full'>
            <!-- Album art -->
            <Transition mode='out-in' name='scale-fade'>
              <div
                v-if='(!showLyrics || isLargeScreen) && !isMobileLandscapeMode'
                :key='playerState.currentSong?.albumId || undefined'
                :class="[
                  'album-art-wrapper mb-4 sm:mb-5',
                  showLyrics && isLargeScreen ? 'album-art-small' : ''
                ]"
              >
                <ImageLoader
                  v-if='playerState.currentSong'
                  :item-id='playerState.currentSong.albumId || undefined'
                  :server-url='serverUrl'
                  :token='token'
                  alt='Album art'
                  class='album-art'
                >
                  <template #fallback>
                    <div class='album-art bg-muted/50 flex items-center justify-center'>
                      <Music2 class='size-20 text-muted-foreground/50' />
                    </div>
                  </template>
                </ImageLoader>
              </div>
            </Transition>

            <!-- Lyrics for mobile portrait (replaces album art) -->
            <div
              @touchmove.stop
              @touchstart.stop
              v-if='showLyrics && !isLargeScreen && isMobilePortraitMode'
              class='w-full h-64 mb-6'
            >
              <LyricsView
                @lyrics-loaded='onLyricsLoaded'
                @seek='handleLyricsSeek'
                :current-time='playerState.currentTime'
                :duration='playerState.duration'
                :is-in-sidebar='false'
                :song='playerState.currentSong'
                :visible='showLyrics'
                class='h-full'
              />
            </div>

            <!-- Song info -->
            <div class='song-info w-full text-center mb-3 sm:mb-4'>
              <h1 class='text-lg sm:text-xl md:text-2xl font-bold text-white truncate'>
                {{ playerState.currentSong?.name || 'Unknown Song' }}
              </h1>
              <p class='text-sm sm:text-base text-white/70 truncate mt-1'>
                {{ playerState.currentSong?.artists?.join(', ') || 'Unknown Artist' }}
              </p>
              <p class='text-xs sm:text-sm text-white/50 truncate'>
                {{ playerState.currentSong?.album || 'Unknown Album' }}
              </p>
              <p v-if='songFormatInfo' class='text-[10px] sm:text-xs text-white/40 mt-1'>
                {{ songFormatInfo }}
              </p>
            </div>

            <!-- Progress bar -->
            <div @touchmove.stop @touchstart.stop class='w-full'>
              <Slider
                @update:model-value='$event && $emit("seek", $event[0])'
                :max='100'
                :model-value='[progress]'
                :step='0.1'
                class='w-full'
              />
              <div class='flex justify-between text-[10px] sm:text-xs text-white/60 mt-1.5 sm:mt-2 font-mono'>
                <span>{{ formatTime(playerState.currentTime) }}</span>
                <span>{{ formatTime(playerState.duration) }}</span>
              </div>
            </div>

            <!-- Playback controls -->
            <div @touchmove.stop @touchstart.stop class='flex items-center gap-1 sm:gap-2 mt-3'>
              <!-- Secondary controls (wider screens only) -->
              <template v-if='showSecondaryControls'>
                <Button
                  @click="playerState.currentSong && $emit('toggle-favorite', playerState.currentSong)"
                  :class="['fs-control-btn mr-2', playerState.currentSong?.isFavorite && 'is-active']"
                  size='icon'
                  variant='ghost'
                >
                  <Heart :class="['size-5', playerState.currentSong?.isFavorite && 'fill-current']" />
                </Button>
                <div class='relative mr-2'>
                  <Button
                    @click='toggleVolumePopup'
                    :class="['fs-control-btn', isVolumePopupVisible && 'is-active']"
                    size='icon'
                    variant='ghost'
                    data-volume-button
                  >
                    <Volume2 v-if='effectiveVolume > 50' class='size-5' />
                    <Volume1 v-else-if='effectiveVolume > 0' class='size-5' />
                    <VolumeX v-else class='size-5' />
                  </Button>
                  <Transition name='pop'>
                    <div
                      v-if='isVolumePopupVisible'
                      ref='volumePopupRef'
                      class='volume-popup'
                    >
                      <span class='text-xs text-white/70 font-medium tabular-nums'>
                        {{ Math.round(effectiveVolume) }}%
                      </span>
                      <Slider
                        @update:model-value="$event && $emit('volume-change', $event[0])"
                        :max='100'
                        :model-value='[effectiveVolume]'
                        :step='1'
                        class='h-20 w-1.5'
                        orientation='vertical'
                      />
                      <button
                        @click.stop="$emit('toggle-mute')"
                        class='p-1 text-white/60 hover:text-white transition-colors'
                      >
                        <VolumeX v-if='effectiveVolume === 0' class='size-4' />
                        <Volume2 v-else class='size-4' />
                      </button>
                    </div>
                  </Transition>
                </div>
              </template>

              <!-- Primary controls -->
              <Button
                @click="$emit('toggle-shuffle')"
                :class="['fs-control-btn', playerState.isShuffled && 'is-active']"
                size='icon'
                variant='ghost'
              >
                <Shuffle class='size-5' />
              </Button>

              <Button
                @click="$emit('previous-song')"
                :disabled='!playerState.hasPrevious'
                class='fs-control-btn'
                size='icon'
                variant='ghost'
              >
                <SkipBack class='size-5 sm:size-6' />
              </Button>

              <Button
                @click="$emit('toggle-play-pause')"
                :class="['play-btn', isMobilePortraitMode ? 'size-14 sm:size-16' : 'size-12 sm:size-14']"
              >
                <Pause v-if='playerState.isPlaying' class='size-5 sm:size-6 md:size-7' />
                <Play v-else class='size-5 sm:size-6 md:size-7 ml-0.5' />
              </Button>

              <Button
                @click="$emit('next-song')"
                :disabled='!playerState.hasNext'
                class='fs-control-btn'
                size='icon'
                variant='ghost'
              >
                <SkipForward class='size-5 sm:size-6' />
              </Button>

              <Button
                @click="$emit('toggle-repeat')"
                :class="['fs-control-btn', playerState.repeatMode !== 'none' && 'is-active']"
                size='icon'
                variant='ghost'
              >
                <Repeat1 v-if="playerState.repeatMode === 'one'" class='size-5' />
                <Repeat v-else class='size-5' />
              </Button>

              <!-- Secondary controls (wider screens only) -->
              <template v-if='showSecondaryControls'>
                <Button
                  @click="$emit('toggle-queue')"
                  :class="['fs-control-btn ml-2', showQueue && 'is-active']"
                  size='icon'
                  variant='ghost'
                >
                  <ListMusic class='size-5' />
                </Button>
                <Button
                  @click="$emit('toggle-equalizer')"
                  :class="['fs-control-btn', showEqualizer && 'is-active']"
                  size='icon'
                  variant='ghost'
                >
                  <Sliders class='size-5' />
                </Button>
              </template>
            </div>
          </div>
        </div>

        <!-- Lyrics panel (large screens) - always in DOM for smooth transitions -->
        <aside
          v-if='isLargeScreen || isMobileLandscapeMode'
          @touchmove.stop
          @touchstart.stop
          :class="[
            'lyrics-panel',
            showLyrics ? 'lyrics-panel-open' : 'lyrics-panel-closed'
          ]"
        >
          <div class='lyrics-panel-content h-full flex items-center justify-center'>
            <div class='lyrics-inner h-full w-full max-w-xl'>
              <LyricsView
                @lyrics-loaded='onLyricsLoaded'
                @seek='handleLyricsSeek'
                :current-time='playerState.currentTime'
                :duration='playerState.duration'
                :is-in-sidebar='false'
                :size='isMobileLandscapeMode ? "small" : "large"'
                :song='playerState.currentSong'
                :visible='showLyrics'
                class='h-full'
              />
            </div>
          </div>
        </aside>
      </main>

      <!-- Bottom bar for mobile/narrow screens -->
      <footer
        @touchmove.stop
        @touchstart.stop
        v-if='isMobilePortraitMode || isNarrowScreen'
        :style='isMobilePortraitMode ? { paddingBottom: `calc(1rem + env(safe-area-inset-bottom))` } : {}'
        class='absolute bottom-0 left-0 right-0 z-20 flex items-center justify-center gap-3 sm:gap-4 p-3 sm:p-4'
      >
        <Button
          @click="playerState.currentSong && $emit('toggle-favorite', playerState.currentSong)"
          :class="['fs-control-btn', playerState.currentSong?.isFavorite && 'is-active']"
          size='icon'
          variant='ghost'
        >
          <Heart :class="['size-5', playerState.currentSong?.isFavorite && 'fill-current']" />
        </Button>

        <div class='relative'>
          <Button
            @click='toggleVolumePopup'
            :class="['fs-control-btn', isVolumePopupVisible && 'is-active']"
            size='icon'
            variant='ghost'
            data-volume-button
          >
            <Volume2 v-if='effectiveVolume > 50' class='size-5' />
            <Volume1 v-else-if='effectiveVolume > 0' class='size-5' />
            <VolumeX v-else class='size-5' />
          </Button>
          <Transition name='pop'>
            <div v-if='isVolumePopupVisible' ref='volumePopupRef' class='volume-popup'>
              <span class='text-xs text-white/70 font-medium tabular-nums'>
                {{ Math.round(effectiveVolume) }}%
              </span>
              <Slider
                @update:model-value="$event && $emit('volume-change', $event[0])"
                :max='100'
                :model-value='[effectiveVolume]'
                :step='1'
                class='h-20 w-1.5'
                orientation='vertical'
              />
              <button
                @click.stop="$emit('toggle-mute')"
                class='p-1 text-white/60 hover:text-white transition-colors'
              >
                <VolumeX v-if='effectiveVolume === 0' class='size-4' />
                <Volume2 v-else class='size-4' />
              </button>
            </div>
          </Transition>
        </div>

        <Button
          @click="$emit('toggle-queue')"
          :class="['fs-control-btn', showQueue && 'is-active']"
          size='icon'
          variant='ghost'
        >
          <ListMusic class='size-5' />
        </Button>

        <Button
          @click="$emit('toggle-equalizer')"
          :class="['fs-control-btn', showEqualizer && 'is-active']"
          size='icon'
          variant='ghost'
        >
          <Sliders class='size-5' />
        </Button>
      </footer>
    </div>
  </Transition>
</template>

<style scoped>
/* Main container */
.fullscreen-player {
  background: var(--background);
  transition: transform 0.35s cubic-bezier(0.32, 0.72, 0, 1),
              opacity 0.35s cubic-bezier(0.32, 0.72, 0, 1);
  will-change: transform, opacity;
}

/* Album art - fluid sizing based on viewport */
.album-art-wrapper {
  /* Scales fluidly with viewport - much larger on big screens */
  width: clamp(12rem, 42vmin, 28rem);
  aspect-ratio: 1;
}

/* Smaller album art when lyrics are open */
.album-art-small {
  width: clamp(10rem, 28vmin, 18rem);
}

.album-art {
  width: 100%;
  height: 100%;
  object-fit: cover;
  border-radius: 0.75rem;
  box-shadow:
    0 25px 50px -12px rgba(0, 0, 0, 0.5),
    0 0 0 1px rgba(255, 255, 255, 0.1);
}

/* Mobile adjustments - larger relative to screen */
.is-mobile .album-art-wrapper {
  width: clamp(12rem, 55vmin, 18rem);
}

.is-mobile .center-items {
  max-width: clamp(12rem, 55vmin, 18rem);
}

/* Control buttons - subtle ghost style */
.fs-control-btn {
  color: rgba(255, 255, 255, 0.6) !important;
  background: transparent !important;
  border: none !important;
  transition: color 0.15s ease;
}

.fs-control-btn:hover:not(:disabled) {
  color: rgba(255, 255, 255, 0.9) !important;
}

.fs-control-btn:disabled {
  opacity: 0.3;
}

.fs-control-btn.is-active {
  color: white !important;
}

/* Play button - circle, accent colored, no scale */
.play-btn {
  background: var(--accent) !important;
  color: var(--accent-foreground) !important;
  border: none !important;
  border-radius: 9999px !important;
  padding: 0 !important;
  height: auto !important;
  aspect-ratio: 1 / 1 !important;
  box-shadow: 0 4px 16px -4px rgba(0, 0, 0, 0.3);
  transition: box-shadow 0.15s ease, opacity 0.15s ease;
}

.play-btn:hover {
  box-shadow: 0 6px 20px -4px rgba(0, 0, 0, 0.4);
}

.play-btn:active {
  opacity: 0.9;
}

/* Volume popup */
.volume-popup {
  position: absolute;
  bottom: 100%;
  left: 50%;
  transform: translateX(-50%);
  margin-bottom: 0.5rem;
  padding: 0.75rem;
  background: rgba(30, 30, 30, 0.95);
  backdrop-filter: blur(20px);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 0.75rem;
  box-shadow: 0 10px 40px -10px rgba(0, 0, 0, 0.5);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.5rem;
  z-index: 100;
}

/* Transitions */
.fullscreen-player-enter-active,
.fullscreen-player-leave-active {
  transition: transform 0.4s cubic-bezier(0.32, 0.72, 0, 1),
              opacity 0.4s cubic-bezier(0.32, 0.72, 0, 1);
}

.fullscreen-player-enter-from {
  transform: translateY(100%);
  opacity: 0;
}

.fullscreen-player-leave-to {
  transform: translateY(100%);
  opacity: 0;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.5s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.scale-fade-enter-active,
.scale-fade-leave-active {
  transition: all 0.3s cubic-bezier(0.32, 0.72, 0, 1);
}

.scale-fade-enter-from {
  opacity: 0;
  transform: scale(0.95);
}

.scale-fade-leave-to {
  opacity: 0;
  transform: scale(1.02);
}

/* Side panel (EQ/Queue) - slides in from left */
.side-panel {
  transition: width 0.25s cubic-bezier(0.32, 0.72, 0, 1),
              padding 0.25s cubic-bezier(0.32, 0.72, 0, 1);
  will-change: width;
}

/* Side panel open state - fluid width */
.side-panel-open {
  width: clamp(16rem, 22vw, 22rem);
  padding: 1rem;
  padding-left: 1.5rem;
}

/* Side panel content - fluid size */
.side-panel-content {
  width: clamp(14rem, 20vw, 20rem);
  flex-shrink: 0;
  will-change: transform;
}

/* Lyrics panel */
.lyrics-panel {
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  flex-shrink: 0;
  transition: width 0.35s cubic-bezier(0.4, 0, 0.2, 1),
              padding 0.35s cubic-bezier(0.4, 0, 0.2, 1);
}

/* Lyrics panel closed */
.lyrics-panel-closed {
  width: 0;
  padding: 0;
}

/* Lyrics panel open */
.lyrics-panel-open {
  width: 50%;
  padding: 1.5rem;
}

/* Lyrics content */
.lyrics-panel-content {
  width: 100%;
  max-width: 36rem;
  flex-shrink: 0;
  opacity: 1;
  transition: opacity 0.25s ease;
}

/* Fade content when closed */
.lyrics-panel-closed .lyrics-panel-content {
  opacity: 0;
}

/* Center content area */
.center-content {
  transition: padding 0.25s ease;
}

/* Inner wrapper - matches album art width for cohesive layout */
.center-items {
  max-width: clamp(12rem, 42vmin, 28rem);
}

/* Album art size transition */
.album-art-wrapper {
  transition: width 0.25s cubic-bezier(0.32, 0.72, 0, 1);
}

.pop-enter-active,
.pop-leave-active {
  transition: all 0.2s cubic-bezier(0.32, 0.72, 0, 1);
}

.pop-enter-from,
.pop-leave-to {
  opacity: 0;
  transform: translateX(-50%) scale(0.9) translateY(0.5rem);
}

/* Responsive adjustments for different screen sizes */

/* Small screens (phones in landscape, small tablets) */
@media (max-width: 640px) {
  .album-art-wrapper {
    width: clamp(10rem, 45vmin, 16rem);
  }

  .center-items {
    max-width: clamp(10rem, 45vmin, 16rem);
  }

  .side-panel-open {
    width: clamp(14rem, 40vw, 18rem);
    padding: 0.75rem;
    padding-left: 1rem;
  }

  .side-panel-content {
    width: clamp(12rem, 38vw, 16rem);
  }
}

/* Medium screens (tablets) */
@media (min-width: 641px) and (max-width: 1023px) {
  .album-art-wrapper {
    width: clamp(12rem, 35vmin, 20rem);
  }

  .center-items {
    max-width: clamp(12rem, 35vmin, 20rem);
  }

  .lyrics-panel-open {
    width: clamp(16rem, 42vw, 28rem);
  }

  .lyrics-panel-content {
    width: clamp(14rem, 40vw, 26rem);
  }
}

/* Large screens - bigger album art */
@media (min-width: 1024px) {
  .album-art-wrapper {
    width: clamp(16rem, 38vmin, 26rem);
  }

  .center-items {
    max-width: clamp(16rem, 38vmin, 26rem);
  }
}

/* Extra large screens - even bigger */
@media (min-width: 1440px) {
  .album-art-wrapper {
    width: clamp(20rem, 36vmin, 32rem);
  }

  .center-items {
    max-width: clamp(20rem, 36vmin, 32rem);
  }

  .side-panel-open {
    width: clamp(18rem, 20vw, 24rem);
  }

  .side-panel-content {
    width: clamp(16rem, 18vw, 22rem);
  }
}

/* Short viewports (landscape phones, short windows) */
@media (max-height: 600px) {
  .album-art-wrapper {
    width: clamp(8rem, 35vh, 14rem);
  }

  .album-art-small {
    width: clamp(6rem, 25vh, 10rem);
  }
}
</style>