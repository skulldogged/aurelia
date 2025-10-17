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
  import { logger } from '@/lib/logger'
  import { getPlatform, isMobilePortrait, Platform } from '@/lib/platform'
  import { formatDuration, getSongFormatInfo } from '@/lib/utils'
  import { PlayerState, usePlayerStore } from '@/stores'

  const props = defineProps({
    analyserNode: {
      default: null,
      type:    Object as PropType<AnalyserNode | null>,
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
    serverUrl: {
      default: '',
      type:    String,
    },
    show: {
      required: true,
      type:     Boolean,
    },
    startWithLyrics: {
      default: false,
      type:    Boolean,
    },
    token: {
      default: '',
      type:    String,
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

  const playerStore = usePlayerStore()
  const { hasLyrics, visualizerEnabled, visualizerStyle } = storeToRefs(playerStore)

  const backgroundImageData = ref<null | string>(null)

  const onLyricsLoaded = (lyricsFound: boolean): void => {
    playerStore.setHasLyrics(lyricsFound)
  }

  watch(() => props.playerState.currentSong, async newSong => {
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
    if (props.playerState.duration > 0) {
      const percentage = (time / props.playerState.duration) * 100
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
  const songFormatInfo = computed(() => getSongFormatInfo(props.playerState.currentSong))
  const effectiveVolume = computed(() => props.playerState.isMuted ? 0 : props.playerState.volume)
  const progress = computed(() =>
    props.playerState.duration > 0
      ? (props.playerState.currentTime / props.playerState.duration) * 100
      : 0,
  )

  const isDesktop = computed(() => {
    const current = getPlatform()
    return current !== Platform.Android && current !== Platform.IOS
  })

  const isMobilePortraitMode = computed(() => isMobilePortrait())
</script>

<template>
  <div
    v-if='show'
    :class="[
      'fullscreen-player fixed inset-0 bg-background z-50 flex flex-col justify-center',
      { 'lyrics-active': isLyricsOpen }
    ]"
  >
    <!-- Draggable Top Bar -->
    <div
      :style='{ top: `env(safe-area-inset-top)` }'
      class='fixed left-0 right-0 z-[100] h-16'
      data-tauri-drag-region
    />

    <!-- Top Bar Controls -->
    <div
      :style='{ top: `env(safe-area-inset-top)` }'
      class='fixed left-0 right-0 z-[101] h-16 pointer-events-none'
    >
      <div class='p-4 h-full flex items-center'>
        <div :class="['flex items-center pointer-events-auto', isMobilePortraitMode ? 'gap-4' : 'gap-2']">
          <Button
            @click="$emit('close')"
            :size="isMobilePortraitMode ? 'lg' : 'icon'"
            class='bg-black/20 backdrop-blur-sm text-white border-white/20 hover:bg-white/10 hover:text-white'
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
            :size="isMobilePortraitMode ? 'lg' : 'icon'"
            variant='ghost'
          >
            <Mic2 class='size-4' />
          </Button>
        </div>
      </div>
    </div>

    <!-- Window Controls -->
    <WindowControls
      v-if='isDesktop'
      :style='{ top: `env(safe-area-inset-top)` }'
      class='absolute right-0 z-[110]'
    />

    <!-- Simplified Background -->
    <div
      v-if='backgroundImageData'
      :style='{ backgroundImage: `url(${backgroundImageData})` }'
      class='absolute inset-0 z-0 bg-cover bg-center'
    />
    <div class='absolute inset-0 z-0 bg-black/50 backdrop-blur-xl' />

    <div
      v-if='visualizerEnabled && analyserNode && playerState.isPlaying'
      class='absolute bottom-0 left-0 right-0 h-[150px] z-0 opacity-30'
    >
      <AudioVisualizer
        :analyser-node='analyserNode'
        :is-playing='playerState.isPlaying'
        :style='visualizerStyle'
      />
    </div>

    <div
      :style='{ paddingTop: `calc(4rem + env(safe-area-inset-top))` }'
      class='relative z-10 flex flex-col h-full justify-center'
    >
      <!-- Floating EQ/Queue Panel -->
      <div
        v-if='(showEqualizer || showQueue) && !isMobilePortraitMode'
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
      <div class='flex overflow-hidden'>
        <!-- Center - Player controls and album art -->
        <div
          :class="['flex flex-col items-center flex-1', isMobilePortraitMode ? 'p-4 gap-4' : 'p-8 gap-8']"
        >
          <div
            :class="[
              'relative w-full flex flex-col items-center gap-6',
              isSmallScreen ? 'mt-auto' : 'max-w-4xl',
              isSmallScreen && (!isLyricsOpen || !isMobilePortraitMode) ? 'max-w-md' : '',
              isMobilePortraitMode ? 'gap-4' : ''
            ]"
          >
            <!-- Album Art (when not showing lyrics) -->
            <div v-if='isLargeScreen || !isLyricsOpen' class='album-art-container aspect-square'>
              <ImageLoader
                v-if='playerState.currentSong'
                :item-id='playerState.currentSong.albumId || undefined'
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

            <!-- Full-width Lyrics on small screens in portrait mode -->
            <div v-if='!isLargeScreen && isLyricsOpen && isMobilePortraitMode' class='w-full h-[20rem] p-4'>
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

            <!-- Constrained Lyrics on small screens in landscape/other modes -->
            <div
              v-if='!isLargeScreen && isLyricsOpen && !isMobilePortraitMode'
              class='album-art-container aspect-square'
            >
              <div class='size-full p-4'>
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
            </div>

            <!-- Song Info -->
            <div class='w-full text-center'>
              <h1 class='text-2xl font-bold text-foreground truncate mb-1'>
                {{ playerState.currentSong?.name || 'Unknown Song' }}
              </h1>
              <p class='text-lg text-muted-foreground truncate mb-1'>
                {{ playerState.currentSong?.artists?.join(', ') || 'Unknown Artist' }}
              </p>
              <p class='text-base text-muted-foreground/80 truncate'>
                {{ playerState.currentSong?.album || 'Unknown Album' }}
              </p>
              <p v-if='songFormatInfo' class='text-xs text-muted-foreground/60 truncate mt-1'>
                {{ songFormatInfo }}
              </p>
            </div>
          </div>

          <!-- Progress Bar & Controls -->
          <div class='w-full flex flex-col gap-4'>
            <!-- Progress Bar -->
            <div class='w-96 mx-auto'>
              <Slider
                @update:model-value='$event && $emit("seek", $event[0])'
                :max='100'
                :model-value='[progress]'
                :step='0.1'
                class='w-full'
              />
              <div class='flex justify-between text-xs text-muted-foreground mt-2'>
                <span>{{ formatTime(playerState.currentTime) }}</span>
                <span>{{ formatTime(playerState.duration) }}</span>
              </div>
            </div>

            <!-- Main Controls -->
            <div :class="['flex items-center justify-center', isMobilePortraitMode ? 'space-x-4' : 'space-x-2']">
              <Button
                @click="$emit('toggle-shuffle')"
                :class="[playerState.isShuffled ? 'text-primary' : 'text-white hover:text-black']"
                :size="isMobilePortraitMode ? 'lg' : 'icon'"
                variant='ghost'
              >
                <Shuffle class='size-4' />
              </Button>
              <Button
                @click="$emit('previous-song')"
                :disabled='!playerState.hasPrevious'
                :size="isMobilePortraitMode ? 'lg' : 'icon'"
                variant='ghost'
              >
                <SkipBack :class="isMobilePortraitMode ? 'size-6' : 'size-5'" />
              </Button>

              <Button
                @click="$emit('toggle-play-pause')"
                :class="['!rounded-full', isMobilePortraitMode ? 'size-16' : 'size-14']"
                size='icon'
                variant='default'
              >
                <Pause v-if='playerState.isPlaying' :class="isMobilePortraitMode ? 'size-7' : 'size-6'" />
                <Play v-else :class="isMobilePortraitMode ? 'size-7' : 'size-6'" />
              </Button>

              <Button
                @click="$emit('next-song')"
                :disabled='!playerState.hasNext'
                :size="isMobilePortraitMode ? 'lg' : 'icon'"
                variant='ghost'
              >
                <SkipForward :class="isMobilePortraitMode ? 'size-6' : 'size-5'" />
              </Button>
              <Button
                @click="$emit('toggle-repeat')"
                :class="[playerState.repeatMode !== 'none' ? 'text-primary' : 'text-white hover:text-black']"
                :size="isMobilePortraitMode ? 'lg' : 'icon'"
                variant='ghost'
              >
                <Repeat1 v-if="playerState.repeatMode === 'one'" class='size-4' />
                <Repeat v-else class='size-4' />
              </Button>
            </div>
          </div>
        </div>

        <!-- Right side - Lyrics (when active) -->
        <div v-if='showLyrics && isLargeScreen' class='flex-1 flex justify-center items-center p-8'>
          <div
            class='w-[600px] xl:w-[700px] 2xl:w-[800px]'
          >
            <LyricsView
              @lyrics-loaded='onLyricsLoaded'
              @seek='handleLyricsSeek'
              :current-time='playerState.currentTime'
              :duration='playerState.duration'
              :is-in-sidebar='false'
              :song='playerState.currentSong'
              :visible='showLyrics'
              size='large'
            />
          </div>
        </div>
      </div>

      <div
        :class="[
          'absolute bottom-0 left-0 right-0 flex items-center justify-center p-4 z-50',
          isMobilePortraitMode ? 'space-x-4' : 'space-x-2'
        ]"
      >
        <Button
          @click="$emit('toggle-favorite', playerState.currentSong)"
          v-if='playerState.currentSong'
          :class="[playerState.currentSong.isFavorite ? 'text-white' : 'text-white hover:text-black']"
          :size="isMobilePortraitMode ? 'lg' : 'icon'"
          variant='ghost'
        >
          <Heart :class="['size-4', playerState.currentSong.isFavorite ? 'fill-current' : '']" />
        </Button>

        <!-- Volume button -->
        <div class='relative'>
          <Button
            @click='toggleVolumePopup'
            :class="isVolumePopupVisible ? 'bg-accent/20' : ''"
            :size="isMobilePortraitMode ? 'lg' : 'icon'"
            variant='ghost'
            data-volume-button
          >
            <Volume2 v-if='effectiveVolume > 0.5' class='size-4' />
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
                {{ Math.round(effectiveVolume * 100) }}%
              </span>
              <Slider
                @update:model-value="$event && $emit('volume-change', $event[0])"
                :max='100'
                :model-value='[effectiveVolume * 100]'
                :step='1'
                class='h-16 w-1.5'
                orientation='vertical'
              />
              <button
                @click.stop="$emit('toggle-mute')"
                class='text-muted-foreground hover:text-foreground transition-colors p-1 rounded'
              >
                <Volume2 v-if='effectiveVolume > 0.5' class='size-4' />
                <Volume1 v-else-if='effectiveVolume > 0' class='size-4' />
                <VolumeX v-else class='size-4' />
              </button>
            </div>
          </div>
        </div>

        <Button
          @click="$emit('toggle-queue')"
          :size="isMobilePortraitMode ? 'lg' : 'icon'"
          :variant="isQueueOpen ? 'default' : 'ghost'"
        >
          <ListMusic class='size-4' />
        </Button>
        <Button
          @click="$emit('toggle-equalizer')"
          :size="isMobilePortraitMode ? 'lg' : 'icon'"
          :variant="isEqualizerOpen ? 'default' : 'ghost'"
        >
          <Sliders class='size-4' />
        </Button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.album-art-container {
  /* Always maintain square aspect ratio */
  aspect-ratio: 1;
  /* Size based on available space, responsive */
  width: min(60vw, 60vh, 50rem);
  height: min(60vw, 60vh, 50rem);
  max-width: 50rem;
  max-height: 50rem;
}

/* Responsive adjustments for smaller screens */
@media (max-width: 1024px) {
  .album-art-container {
    width: min(45vw, 45vh, 30rem);
    height: min(45vw, 45vh, 30rem);
    max-width: 30rem;
    max-height: 30rem;
  }
}

@media (max-width: 768px) {
  /* On smaller screens, hide the main album art and show the background */
  .album-art-container {
    display: none;
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

/* Mobile portrait adjustments */
@media (max-width: 768px) and (orientation: portrait) {
  .album-art-container {
    display: block;
    width: min(70vw, 50vh, 18rem);
    height: min(70vw, 50vh, 18rem);
    max-width: 18rem;
    max-height: 18rem;
  }
}
</style>