<script setup lang="ts">
  import {
    Album,
    ChevronDown,
    Mic2,
    Music2,
    Pause,
    Play,
    Repeat,
    Repeat1,
    Shuffle,
    SkipBack,
    SkipForward,
  } from 'lucide-vue-next'
  import { computed, ref, watch } from 'vue'
  import { PropType } from 'vue'

  import { Song } from '@/bindings'
  import AudioVisualizer from '@/components/player/AudioVisualizer.vue'
  import ImageLoader from '@/components/shared/ImageLoader.vue'
  import LyricsView from '@/components/shared/LyricsView.vue'
  import Button from '@/components/ui/Button.vue'
  import { Slider } from '@/components/ui/slider'
  import { useImageLoader } from '@/composables/useImageLoader'
  import { playerLogger } from '@/lib/logger'
  import { formatDuration, getSongFormatInfo } from '@/lib/utils'

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
    isPlaying: {
      required: true,
      type:     Boolean,
    },
    isShuffled: {
      required: true,
      type:     Boolean,
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
  })

  const emit = defineEmits<{
    (e: 'close'): void
    (e: 'toggle-play-pause'): void
    (e: 'previous-song'): void
    (e: 'next-song'): void
    (e: 'toggle-shuffle'): void
    (e: 'toggle-repeat'): void
    (e: 'seek', value: number): void
  }>()

  const showLyrics = ref(false)
  const hasLyrics = ref(false)
  const { getImageUrl } = useImageLoader()

  const backgroundImageData = ref<null | string>(null)

  const onLyricsLoaded = (lyricsFound: boolean): void => {
    hasLyrics.value = lyricsFound
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

  watch(() => props.song, newSong => {
    if (newSong) {
      // Immediately check if lyrics exist in the song data
      hasLyrics.value = newSong.lyrics != null && newSong.lyrics.trim() !== ''
    } else {
      hasLyrics.value = false
    }
  }, { immediate: true })

  watch(() => props.show, newVal => {
    if (newVal) {
      if (props.startWithLyrics)
        showLyrics.value = true
    } else {
      showLyrics.value = false
    }
  })

  const handleLyricsSeek = (time: number): void => {
    if (props.duration > 0) {
      const percentage = (time / props.duration) * 100
      emit('seek', percentage)
    }
  }

  const formatTime = formatDuration
  const songFormatInfo = computed(() => getSongFormatInfo(props.song))
</script>

<template>
  <div
    v-if='show'
    :class="['fullscreen-player fixed inset-0 bg-background z-50 flex flex-col', { 'lyrics-active': showLyrics }]"
  >
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

    <div class='relative z-10 flex flex-col h-full'>
      <div class='absolute top-4 left-4 right-4 flex justify-between items-center z-20'>
        <Button
          @click="$emit('close')"
          class='bg-black/20 backdrop-blur-sm text-white border-white/20'
          size='icon'
          variant='ghost'
        >
          <ChevronDown class='size-4' />
        </Button>
        <Button
          @click='showLyrics = !showLyrics'
          :class="[
            'bg-black/20 backdrop-blur-sm text-white border-white/20',
            hasLyrics ? '' : 'opacity-50 cursor-not-allowed'
          ]"
          :disabled='!hasLyrics'
          size='icon'
          variant='ghost'
        >
          <Mic2 v-if='!showLyrics' class='size-4' />
          <Album v-else class='size-4' />
        </Button>
      </div>

      <div
        class='flex flex-col items-center justify-center p-8 gap-8 overflow-hidden flex-1'
      >
        <div class='relative w-full flex flex-col items-center gap-6'>
          <div class='relative size-full'>
            <div
              :class="[
                'flex justify-center items-center size-full transition-opacity duration-300 ease-in-out',
                showLyrics ? 'opacity-0 pointer-events-none' : 'opacity-100'
              ]"
            >
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
            </div>
            <div
              :class="[
                'absolute inset-0 transition-opacity duration-300 ease-in-out',
                showLyrics ? 'opacity-100' : 'opacity-0 pointer-events-none'
              ]"
            >
              <div class='size-full flex justify-center'>
                <LyricsView
                  @lyrics-loaded='onLyricsLoaded'
                  @seek='handleLyricsSeek'
                  :current-time='currentTime'
                  :duration='duration'
                  :is-in-sidebar='false'
                  :song='song'
                  :visible='showLyrics'
                  class='w-full max-w-3xl h-full'
                />
              </div>
            </div>
          </div>

          <div class='w-full max-w-md mx-auto'>
            <div class='w-full text-center mb-4'>
              <h1 class='text-2xl font-bold text-foreground truncate'>
                {{ song?.name || 'Unknown Song' }}
              </h1>
              <p class='text-md text-muted-foreground truncate'>
                {{ song?.artists?.join(', ') || 'Unknown Artist' }}
              </p>
              <p class='text-sm text-muted-foreground truncate'>
                {{ song?.album || 'Unknown Album' }}
              </p>
              <p v-if='songFormatInfo' class='text-xs text-muted-foreground/80 truncate mt-1'>
                {{ songFormatInfo }}
              </p>
            </div>

            <div>
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

            <div class='flex items-center justify-center space-x-4'>
              <Button
                @click='$emit("toggle-shuffle")'
                :class="[isShuffled ? 'text-primary' : 'text-muted-foreground']"
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
                class='rounded-full size-14'
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
                :class="[repeatMode !== 'none' ? 'text-primary' : 'text-muted-foreground']"
                size='icon'
                variant='ghost'
              >
                <Repeat1 v-if="repeatMode === 'one'" class='size-4' />
                <Repeat v-else class='size-4' />
              </Button>
            </div>
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
  /* Size based on available space, but always square */
  width: min(50vw, 50vh, 40rem);
  height: min(50vw, 50vh, 40rem);
  max-width: 40rem;
  max-height: 40rem;
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
    rgba(0, 0, 0, 1) 22%,
    rgba(0, 0, 0, 0.95) 36%,
    rgba(0, 0, 0, 0) 62%
  );
  mask-image: linear-gradient(
    to top,
    rgba(0, 0, 0, 1) 22%,
    rgba(0, 0, 0, 0.95) 36%,
    rgba(0, 0, 0, 0) 62%
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
    background-color: rgba(0, 0, 0, 0.6);
  }

  /* Hide the foreground album art on thin screens */
  .album-art-container {
    opacity: 0;
    pointer-events: none;
  }
}

</style>
