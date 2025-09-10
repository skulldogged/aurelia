<template>
  <div
    v-if='show'
    :class="['fullscreen-player fixed inset-0 bg-background z-50 flex flex-col', { 'lyrics-active': showLyrics }]"
  >
    <!-- Album Art Background (visible on thin screens) -->
    <div
      v-if='backgroundImageData'
      class='absolute inset-0 z-0 album-art-bg'
    >
      <!-- Blurred album art (top portion) -->
      <div
        :style='{ backgroundImage: `url(${backgroundImageData})` }'
        class='absolute inset-0 bg-cover bg-center filter blur-xl album-art-blurred'
      />
      <!-- Clear album art (bottom portion) -->
      <div
        :style='{ backgroundImage: `url(${backgroundImageData})` }'
        class='absolute inset-0 bg-cover bg-center album-art-clear'
      />
    </div>

    <!-- Background -->
    <div
      v-if='backgroundImageData'
      :style='{ backgroundImage: `url(${backgroundImageData})` }'
      class='absolute inset-0 bg-cover bg-center filter blur-3xl opacity-30 z-0'
    />

    <!-- Content Wrapper -->
    <div class='relative z-10 flex flex-col h-full'>
      <!-- Top Corner Buttons -->
      <div class='absolute top-4 left-4 right-4 flex justify-between items-center z-20'>
        <Button
          @click="$emit('close')"
          class='bg-black/20 hover:bg-black/40 backdrop-blur-sm text-white border-white/20'
          size='icon'
          variant='ghost'
        >
          <ChevronDown class='w-5 h-5' />
        </Button>
        <Button
          @click='showLyrics = !showLyrics'
          class='bg-black/20 hover:bg-black/40 backdrop-blur-sm text-white border-white/20'
          size='icon'
          variant='ghost'
        >
          <Mic2 v-if='!showLyrics' class='w-5 h-5' />
          <Album v-else class='w-5 h-5' />
        </Button>
      </div>

      <!-- Main Content & Footer -->
      <div
        class='flex-grow flex flex-col items-center p-8 gap-8 overflow-hidden'
      >
        <div class='relative flex-1 w-full min-h-0 flex flex-col items-center justify-center gap-6'>
          <!-- Album Art / Lyrics Area -->
          <div class='relative w-full h-full'>
            <!-- Album Art -->
            <div
              :class="[
                'flex justify-center items-center w-full h-full transition-opacity duration-300 ease-in-out',
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
                  class='w-full h-full object-cover rounded-lg shadow-2xl'
                >
                  <template #fallback>
                    <div class='flex items-center justify-center w-full h-full rounded-lg bg-muted'>
                      <Music2 class='w-24 h-24 text-muted-foreground' />
                    </div>
                  </template>
                </ImageLoader>
              </div>
            </div>
            <!-- Lyrics -->
            <div
              :class="[
                'absolute inset-0 w-full h-full transition-opacity duration-300 ease-in-out',
                showLyrics ? 'opacity-100' : 'opacity-0 pointer-events-none'
              ]"
            >
              <div class='w-full h-full'>
                <LyricsView
                  @lyrics-loaded='onLyricsLoaded'
                  @seek='handleLyricsSeek'
                  :current-time='currentTime'
                  :duration='duration'
                  :song='song'
                  :visible='showLyrics'
                  class='w-full h-full'
                />
              </div>
            </div>
          </div>

          <!-- Player Controls -->
          <div class='w-full max-w-md mx-auto'>
            <!-- Song Info -->
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

            <!-- Progress Bar -->
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

            <!-- Player Controls -->
            <div class='flex items-center justify-center space-x-4'>
              <Button
                @click='$emit("toggle-shuffle")'
                :class="[isShuffled ? 'text-primary' : 'text-muted-foreground']"
                size='icon'
                variant='ghost'
              >
                <Shuffle class='w-5 h-5' />
              </Button>
              <Button
                @click='$emit("previous-song")'
                :disabled='!hasPrevious'
                size='icon'
                variant='ghost'
              >
                <SkipBack class='w-6 h-6' />
              </Button>
              <Button
                @click='$emit("toggle-play-pause")'
                class='rounded-full w-16 h-16'
              >
                <Pause v-if='isPlaying' class='w-8 h-8' />
                <Play v-else class='w-8 h-8' />
              </Button>
              <Button
                @click='$emit("next-song")'
                :disabled='!hasNext'
                size='icon'
                variant='ghost'
              >
                <SkipForward class='w-6 h-6' />
              </Button>
              <Button
                @click='$emit("toggle-repeat")'
                :class="[repeatMode !== 'none' ? 'text-primary' : 'text-muted-foreground']"
                size='icon'
                variant='ghost'
              >
                <Repeat1 v-if="repeatMode === 'one'" class='w-5 h-5' />
                <Repeat v-else class='w-5 h-5' />
              </Button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
  import { ref, watch, computed } from 'vue'
  import {
    ChevronDown,
    Music2,
    Mic2,
    Shuffle,
    SkipBack,
    Play,
    Pause,
    SkipForward,
    Repeat,
    Repeat1,
    Album,
  } from 'lucide-vue-next'
  import { Button } from '@/components/ui/button'
  import { Slider } from '@/components/ui/slider'
  import LyricsView from '@/components/shared/LyricsView.vue'
  import ImageLoader from '@/components/shared/ImageLoader.vue'
  import { Song } from '@/bindings'
  import { PropType } from 'vue'
  import { useImageLoader } from '@/composables/useImageLoader'

  const props = defineProps({
    show: {
      type:     Boolean,
      required: true,
    },
    song: {
      type:    Object as PropType<Song | null>,
      default: null,
    },
    isPlaying: {
      type:     Boolean,
      required: true,
    },
    progress: {
      type:     Number,
      required: true,
    },
    currentTime: {
      type:     Number,
      required: true,
    },
    duration: {
      type:     Number,
      required: true,
    },
    isShuffled: {
      type:     Boolean,
      required: true,
    },
    repeatMode: {
      type:     String as PropType<'none' | 'all' | 'one'>,
      required: true,
    },
    hasPrevious: {
      type:     Boolean,
      required: true,
    },
    hasNext: {
      type:     Boolean,
      required: true,
    },
    startWithLyrics: {
      type:    Boolean,
      default: false,
    },
    serverUrl: {
      type:    String,
      default: '',
    },
    token: {
      type:    String,
      default: '',
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

  // Background image data
  const backgroundImageData = ref<string | null>(null)

  const onLyricsLoaded = (lyricsFound: boolean) => {
    hasLyrics.value = lyricsFound
  }

  // Watch for song changes to update background image
  watch(() => props.song, async newSong => {
    if (newSong && props.serverUrl && props.token) {
      try {
        const imageId = newSong.albumId || newSong.id
        const imageData = getImageUrl(imageId, props.serverUrl, props.token, 'Primary')
        backgroundImageData.value = imageData
      } catch (error) {
        console.error('Failed to load background image:', error)
        backgroundImageData.value = null
      }
    } else {
      backgroundImageData.value = null
    }
  }, { immediate: true })

  watch(() => props.song, newSong => {
    if (newSong)
      hasLyrics.value = false
  })

  watch(() => props.show, newVal => {
    if (newVal) {
      if (props.startWithLyrics)
        showLyrics.value = true
    } else {
      // Reset when closing
      showLyrics.value = false
    }
  })

  const handleLyricsSeek = (time: number) => {
    if (props.duration > 0) {
      const percentage = (time / props.duration) * 100
      emit('seek', percentage)
    }
  }

  const formatTime = (seconds: number) => {
    const mins = Math.floor(seconds / 60)
    const secs = Math.floor(seconds % 60)
    return `${mins}:${secs.toString().padStart(2, '0')}`
  }

  const songFormatInfo = computed(() => {
    if (!props.song) return ''
    const parts: string[] = []
    if (props.song.codec) parts.push(props.song.codec.toUpperCase())
    if (props.song.sampleRate) parts.push(`${props.song.sampleRate / 1000} kHz`)
    if (props.song.bitRate) parts.push(`${Math.round(props.song.bitRate / 1000)} kbps`)
    return parts.join(' / ')
  })
</script>

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
