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
  import { computed, ref, watch } from 'vue'
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
  import { playerLogger } from '@/lib/logger'
  import { formatDuration, getSongFormatInfo } from '@/lib/utils'
  import { usePlayerStore } from '@/stores'

  const props = defineProps({
    analyserNode: {
      default: null,
      type:    Object as PropType<AnalyserNode | null>,
    },
    currentTime: {
      required: true,
      type:     Number,
    },
    duration: {
      required: true,
      type:     Number,
    },
    hasNext: {
      required: true,
      type:     Boolean,
    },
    hasPrevious: {
      required: true,
      type:     Boolean,
    },
    isEqualizerOpen: {
      default: false,
      type:    Boolean,
    },
    isLyricsOpen: {
      default: false,
      type:    Boolean,
    },
    isMuted: {
      default: false,
      type:    Boolean,
    },
    isPlaying: {
      required: true,
      type:     Boolean,
    },
    isQueueOpen: {
      default: false,
      type:    Boolean,
    },
    isShuffled: {
      required: true,
      type:     Boolean,
    },
    playlist: {
      default: () => [],
      type:    Array as PropType<Song[]>,
    },
    progress: {
      required: true,
      type:     Number,
    },
    repeatMode: {
      required: true,
      type:     String as PropType<'all' | 'none' | 'one'>,
    },
    serverUrl: {
      default: '',
      type:    String,
    },
    show: {
      required: true,
      type:     Boolean,
    },
    song: {
      default: null,
      type:    Object as PropType<null | Song>,
    },
    startWithLyrics: {
      default: false,
      type:    Boolean,
    },
    token: {
      default: '',
      type:    String,
    },
    visualizerEnabled: {
      default: true,
      type:    Boolean,
    },
    visualizerStyle: {
      default: 'bars-mirror',
      type:    String as PropType<'bars' | 'bars-mirror' | 'curve' | 'wave'>,
    },
    volume: {
      default: 100,
      type:    Number,
    },
  })

  const emit = defineEmits<{
    (e: 'close'): void
    (e: 'toggle-play-pause'): void
    (e: 'previous-song'): void
    (e: 'next-song'): void
    (e: 'toggle-shuffle'): void
    (e: 'toggle-repeat'): void
    (e: 'seek', value: number): void
    (e: 'toggle-equalizer'): void
    (e: 'toggle-lyrics'): void
    (e: 'toggle-queue'): void
    (e: 'toggle-favorite', song: Song): void
    (e: 'volume-change', value: number): void
    (e: 'toggle-mute'): void
    (e: 'remove-song', song: Song): void
    (e: 'update:playlist', playlist: Song[]): void
  }>()

  const showLyrics = ref(false)
  const showEqualizer = ref(false)
  const showQueue = ref(false)
  const volumePopupRef = ref<HTMLDivElement | null>(null)
  const isVolumePopupVisible = ref(false)
  const { getImageUrl } = useImageLoader()
  const isLargeScreen = useMediaQuery('(min-width: 1081px)')
  const isSmallScreen = useMediaQuery('(max-width: 768px)')

  // Get hasLyrics from store instead of local ref
  const { hasLyrics } = storeToRefs(usePlayerStore())

  const backgroundImageData = ref<null | string>(null)

  const onLyricsLoaded = (lyricsFound: boolean): void => {
    usePlayerStore().setHasLyrics(lyricsFound)
  }

  watch(() => props.song, async newSong => {
    if (newSong && props.serverUrl && props.token) {
      try {
        const imageId = newSong.albumId || newSong.id
        const imageData = await getImageUrl(imageId, props.serverUrl, props.token, 'Primary')
        backgroundImageData.value = imageData
      } catch (error) {
        playerLogger.error('Failed to load background image:', error)
        backgroundImageData.value = null
      }
    } else {
      backgroundImageData.value = null
    }
  }, { immediate: true })

  watch(() => props.show, newVal => {
    if (newVal) {
      if (props.startWithLyrics)
        showLyrics.value = true
    } else {
      showLyrics.value = false
      showEqualizer.value = false
      showQueue.value = false
    }
  })

  watch(() => props.isLyricsOpen, newVal => {
    showLyrics.value = newVal
  })

  watch(() => props.isEqualizerOpen, newVal => {
    showEqualizer.value = newVal
  })

  watch(() => props.isQueueOpen, newVal => {
    showQueue.value = newVal
  })

  const handleLyricsSeek = (time: number): void => {
    if (props.duration > 0) {
      const percentage = (time / props.duration) * 100
      emit('seek', percentage)
    }
  }

  const toggleVolumePopup = (): void => {
    isVolumePopupVisible.value = !isVolumePopupVisible.value
  }

  const closeVolumePopup = (): void => {
    isVolumePopupVisible.value = false
  }

  const handleClickOutside = (event: Event): void => {
    const target = event.target as Element
    const volumeButton = target.closest('[data-volume-button]')
    const insidePopup = volumePopupRef.value && volumePopupRef.value.contains(target)

    if (volumeButton || insidePopup) return

    if (isVolumePopupVisible.value)
      closeVolumePopup()
  }

  watch(isVolumePopupVisible, visible => {
    if (visible)
      document.addEventListener('click', handleClickOutside)
    else
      document.removeEventListener('click', handleClickOutside)
  })

  const formatTime = formatDuration
  const songFormatInfo = computed(() => getSongFormatInfo(props.song))
  const effectiveVolume = computed(() => props.isMuted ? 0 : props.volume)
</script>

<template>
  <div
    v-if='show'
    :class="['fullscreen-player fixed inset-0 bg-background z-50 flex flex-col', { 'lyrics-active': isLyricsOpen }]"
  >
    <!-- Draggable Top Bar -->
    <div class='fixed top-0 left-0 right-0 z-[100] h-16' data-tauri-drag-region />

    <!-- Top Bar Controls -->
    <div class='fixed top-0 left-0 right-0 z-[101] h-16 pointer-events-none'>
      <div class='p-4 h-full flex items-center'>
        <div class='flex items-center gap-2 pointer-events-auto'>
          <Button
            @click="$emit('close')"
            class='bg-black/20 backdrop-blur-sm text-white border-white/20 hover:bg-white/10 hover:text-white'
            size='icon'
            variant='ghost'
          >
            <ChevronDown class='size-4' />
          </Button>
          <Button
            @click="$emit('toggle-lyrics')"
            :class="[
              'bg-black/20 backdrop-blur-sm text-white border-white/20 hover:bg-white/10 hover:text-white',
              isLyricsOpen ? 'bg-black/40' : ''
            ]"
            :disabled='!hasLyrics'
            size='icon'
            variant='ghost'
          >
            <Mic2 class='size-4' />
          </Button>
        </div>
      </div>
    </div>

    <!-- Window Controls -->
    <WindowControls class='absolute top-0 right-0 z-[110]' />

    <div
      v-if='backgroundImageData'
      class='absolute inset-0 z-0 album-art-bg'
    >
      <div
        :style='{ backgroundImage: `url(${backgroundImageData})` }'
        class='absolute inset-0 bg-cover bg-center filter blur-xl album-art-blurred'
      />
      <div
        :style='{ backgroundImage: `url(${backgroundImageData})` }'
        class='absolute inset-0 bg-cover bg-center album-art-clear'
      />
    </div>

    <div
      v-if='backgroundImageData'
      :style='{ backgroundImage: `url(${backgroundImageData})` }'
      class='absolute inset-0 bg-cover bg-center filter blur-3xl opacity-30 z-0'
    />

    <div
      v-if='visualizerEnabled && analyserNode && isPlaying'
      class='absolute bottom-0 left-0 right-0 h-[150px] z-0 opacity-30'
    >
      <AudioVisualizer
        :analyser-node='analyserNode'
        :is-playing='isPlaying'
        :style='visualizerStyle'
      />
    </div>

    <div class='relative z-10 flex flex-col h-full pt-16'>
      <!-- Floating EQ/Queue Panel -->
      <div
        v-if='showEqualizer || showQueue'
        class='absolute left-6 top-1/2 -translate-y-1/2 z-10'
      >
        <div
          class='w-[400px] max-h-[80vh] bg-background/90 backdrop-blur-md border border-border/50
                 rounded-xl shadow-2xl overflow-hidden'
        >
          <FullscreenEqualizer
            v-if='showEqualizer'
            class='h-full'
          />
          <FullscreenQueue
            @remove-song='$emit("remove-song", $event)'
            @update:playlist='$emit("update:playlist", $event)'
            v-if='showQueue'
            class='h-full'
          />
        </div>
      </div>

      <!-- Main Content -->
      <div class='flex-1 flex overflow-hidden'>
        <!-- Center - Player controls and album art -->
        <div :class="['flex-1 flex flex-col items-center p-8 gap-8 min-w-0', isSmallScreen ? '' : 'justify-center']">
          <!-- Lyrics on small screens -->
          <transition name='fade'>
            <div v-if='!isLargeScreen && isLyricsOpen' class='w-full flex-grow p-4 min-h-0'>
              <LyricsView
                @lyrics-loaded='onLyricsLoaded'
                @seek='handleLyricsSeek'
                :current-time='currentTime'
                :duration='duration'
                :is-in-sidebar='false'
                :song='song'
                :visible='showLyrics'
                class='h-full'
              />
            </div>
          </transition>

          <div :class="['relative w-full flex flex-col items-center gap-6 max-w-md', isSmallScreen ? 'mt-auto' : '']">
            <!-- Album Art & Song Info -->
            <div v-if='isLargeScreen || !isLyricsOpen' class='contents'>
              <!-- Album Art -->
              <div class='album-art-container aspect-square'>
                <ImageLoader
                  v-if='song'
                  :item-id='song.id'
                  :server-url='serverUrl'
                  :token='token'
                  alt='Album art'
                  class='size-full object-cover rounded-lg shadow-2xl'
                >
                  <template #fallback>
                    <div class='flex items-center justify-center size-full rounded-lg bg-muted'>
                      <Music2 class='size-24 text-muted-foreground' />
                    </div>
                  </template>
                </ImageLoader>
              </div>

              <!-- Song Info -->
              <div class='w-full text-center'>
                <h1 class='text-2xl font-bold text-foreground truncate mb-1'>
                  {{ song?.name || 'Unknown Song' }}
                </h1>
                <p class='text-lg text-muted-foreground truncate mb-1'>
                  {{ song?.artists?.join(', ') || 'Unknown Artist' }}
                </p>
                <p class='text-base text-muted-foreground/80 truncate'>
                  {{ song?.album || 'Unknown Album' }}
                </p>
                <p v-if='songFormatInfo' class='text-xs text-muted-foreground/60 truncate mt-1'>
                  {{ songFormatInfo }}
                </p>
              </div>
            </div>

            <!-- Progress Bar & Controls -->
            <div class='w-full flex flex-col gap-4'>
              <!-- Progress Bar -->
              <div class='w-full'>
                <Slider
                  @update:model-value='$event && $emit("seek", $event[0])'
                  :max='100'
                  :model-value='[progress]'
                  :step='0.1'
                  class='w-full'
                />
                <div class='flex justify-between text-xs text-muted-foreground mt-2'>
                  <span>{{ formatTime(currentTime) }}</span>
                  <span>{{ formatTime(duration) }}</span>
                </div>
              </div>

              <!-- Main Controls -->
              <div class='flex items-center justify-between w-full'>
                <!-- Left side - Favorite and Volume -->
                <div class='flex items-center space-x-2'>
                  <Button
                    @click="$emit('toggle-favorite', song)"
                    v-if='song'
                    :class="[song.isFavorite ? 'text-white' : 'text-white hover:text-black']"
                    size='icon'
                    variant='ghost'
                  >
                    <Heart :class="['size-4', song.isFavorite ? 'fill-current' : '']" />
                  </Button>

                  <!-- Volume button -->
                  <div class='relative'>
                    <Button
                      @click='toggleVolumePopup'
                      :class='isVolumePopupVisible ? "bg-accent/20" : ""'
                      size='icon'
                      variant='ghost'
                      data-volume-button
                    >
                      <Volume2 v-if='effectiveVolume > 50' class='size-4' />
                      <Volume1 v-else-if='effectiveVolume > 0' class='size-4' />
                      <VolumeX v-else class='size-4' />
                    </Button>
                    <div
                      v-if='isVolumePopupVisible'
                      ref='volumePopupRef'
                      class='absolute bottom-full left-1/2 transform -translate-x-1/2 mb-2 p-3
                           bg-card border border-border rounded-md shadow-lg z-50'
                    >
                      <div class='flex flex-col items-center gap-2'>
                        <span class='text-xs text-muted-foreground font-medium'>
                          {{ Math.round(effectiveVolume) }}%
                        </span>
                        <Slider
                          @update:model-value='$event && $emit("volume-change", $event[0])'
                          :max='100'
                          :model-value='[effectiveVolume]'
                          :step='1'
                          class='h-16 w-1.5'
                          orientation='vertical'
                        />
                        <button
                          @click.stop='$emit("toggle-mute")'
                          class='text-muted-foreground hover:text-foreground transition-colors p-1 rounded'
                        >
                          <Volume2 v-if='effectiveVolume > 50' class='size-4' />
                          <Volume1 v-else-if='effectiveVolume > 0' class='size-4' />
                          <VolumeX v-else class='size-4' />
                        </button>
                      </div>
                    </div>
                  </div>
                </div>

                <!-- Center - Playback Controls -->
                <div class='flex items-center space-x-2'>
                  <Button
                    @click='$emit("toggle-shuffle")'
                    :class="[isShuffled ? 'text-primary' : 'text-white hover:text-black']"
                    size='icon'
                    variant='ghost'
                  >
                    <Shuffle class='size-4' />
                  </Button>
                  <Button
                    @click='$emit("previous-song")'
                    :disabled='!hasPrevious'
                    size='icon'
                    variant='ghost'
                  >
                    <SkipBack class='size-5' />
                  </Button>

                  <Button
                    @click='$emit("toggle-play-pause")'
                    class='!rounded-full size-14'
                    size='icon'
                    variant='default'
                  >
                    <Pause v-if='isPlaying' class='size-6' />
                    <Play v-else class='size-6' />
                  </Button>

                  <Button
                    @click='$emit("next-song")'
                    :disabled='!hasNext'
                    size='icon'
                    variant='ghost'
                  >
                    <SkipForward class='size-5' />
                  </Button>
                  <Button
                    @click='$emit("toggle-repeat")'
                    :class="[repeatMode !== 'none' ? 'text-primary' : 'text-white hover:text-black']"
                    size='icon'
                    variant='ghost'
                  >
                    <Repeat1 v-if="repeatMode === 'one'" class='size-4' />
                    <Repeat v-else class='size-4' />
                  </Button>
                </div>

                <!-- Right side - EQ and Queue -->
                <div class='flex items-center space-x-2'>
                  <Button
                    @click="$emit('toggle-queue')"
                    :variant="isQueueOpen ? 'default' : 'ghost'"
                    size='icon'
                  >
                    <ListMusic class='size-4' />
                  </Button>
                  <Button
                    @click="$emit('toggle-equalizer')"
                    :variant="isEqualizerOpen ? 'default' : 'ghost'"
                    size='icon'
                  >
                    <Sliders class='size-4' />
                  </Button>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Right side - Lyrics (when active) -->
        <div v-if='showLyrics && isLargeScreen' class='flex-1 flex justify-center items-center p-8'>
          <div
            class='w-[600px] xl:w-[700px] 2xl:w-[800px] h-full'
          >
            <LyricsView
              @lyrics-loaded='onLyricsLoaded'
              @seek='handleLyricsSeek'
              :current-time='currentTime'
              :duration='duration'
              :is-in-sidebar='false'
              :song='song'
              :visible='showLyrics'
              class='h-full'
              size='large'
            />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.album-art-container {
  /* Always maintain square aspect ratio */
  aspect-ratio: 1;
  /* Size based on available space, responsive */
  width: min(40vw, 40vh, 35rem);
  height: min(40vw, 40vh, 35rem);
  max-width: 35rem;
  max-height: 35rem;
}

/* Album art background - hidden by default, visible on thin screens */
.album-art-bg {
  opacity: 0;
}

.album-art-bg::before {
  content: '';
  position: absolute;
  inset: 0;
  background-color: rgba(0, 0, 0, 0);
  z-index: 2;
  transition: background-color 0.3s ease-in-out;
  pointer-events: none;
}

/* Blurred album art - shows through at bottom */
.album-art-blurred {
  /* Ensure this layer sits above the clear album art */
  z-index: 1;
  pointer-events: none;

  /* Strong blur and slight darkening so controls are readable */
  filter: blur(28px) brightness(0.6) saturate(0.9);

  /* Prevent blur edge cropping and include corners */
  top: -56px;
  right: -56px;
  bottom: -56px;
  left: -56px;
  transform: scale(1.02);

  /* Only show this blur near the bottom, fading upward */
  -webkit-mask-image: linear-gradient(
    to top,
    rgba(0, 0, 0, 1) 280px,
    rgba(0, 0, 0, 0.95) 340px,
    rgba(0, 0, 0, 0) 460px
  );
  mask-image: linear-gradient(
    to top,
    rgba(0, 0, 0, 1) 280px,
    rgba(0, 0, 0, 0.95) 340px,
    rgba(0, 0, 0, 0) 460px
  );
}

.album-art-blurred::after {
  content: '';
  position: absolute;
  inset: 0;
  /* Darken bottom area further with a soft gradient like Cider */
  background: linear-gradient(
    to top,
    rgba(0, 0, 0, 0.70) 0%,
    rgba(0, 0, 0, 0.55) 24%,
    rgba(0, 0, 0, 0.28) 48%,
    rgba(0, 0, 0, 0) 70%
  );
}

/* Show album art background on thin screens */
@media (max-width: 768px) {
  .album-art-bg {
    opacity: 1;
  }

  .fullscreen-player.lyrics-active .album-art-bg::before {
    background-color: transparent;
  }

  .fullscreen-player.lyrics-active .album-art-blurred {
    -webkit-mask-image: none;
    mask-image: none;
    /* Make it a bit darker when lyrics are on */
    filter: blur(28px) brightness(0.6) saturate(0.9);
  }

  /* Hide the foreground album art on thin screens */
  .album-art-container {
    opacity: 0;
    pointer-events: none;
  }
}

/* Responsive adjustments for smaller screens */
@media (max-width: 1024px) {
  .album-art-container {
    width: min(35vw, 35vh, 20rem);
    height: min(35vw, 35vh, 20rem);
    max-width: 20rem;
    max-height: 20rem;
  }
}

@media (max-width: 640px) {
  .album-art-container {
    width: min(50vw, 50vh, 15rem);
    height: min(50vw, 50vh, 15rem);
    max-width: 15rem;
    max-height: 15rem;
  }
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
