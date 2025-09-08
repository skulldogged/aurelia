<template>
  <transition name='slide-fade'>
    <div
      v-if='show'
      class='fullscreen-player fixed inset-0 bg-background z-50 flex flex-col'
    >
      <!-- Background -->
      <div
        :style='{ backgroundImage: `url(${song?.albumArtUrl})` }'
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
            v-if='hasLyrics'
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
            <div
              class='relative grid place-items-center aspect-square album-art-container'
            >
              <transition name='fade'>
                <div
                  v-show='!showLyrics'
                  class='w-full h-full [grid-area:1/1] overflow-hidden rounded-lg'
                >
                  <img
                    v-if='song?.albumArtUrl'
                    :src='song.albumArtUrl'
                    alt='Album art'
                    class='w-full h-full object-cover rounded-lg shadow-2xl'
                  >
                  <div
                    v-else
                    class='flex items-center justify-center
                           w-full h-full rounded-lg bg-muted'
                  >
                    <Music2 class='w-24 h-24 text-muted-foreground' />
                  </div>
                </div>
              </transition>
              <transition name='fade'>
                <div
                  v-show='showLyrics'
                  class='w-full h-full [grid-area:1/1] rounded-lg overflow-hidden'
                >
                  <LyricsView
                    @lyrics-loaded='onLyricsLoaded'
                    @seek='handleLyricsSeek'
                    :current-time='currentTime.value'
                    :duration='duration.value'
                    :song='song'
                    class='w-full h-full'
                  />
                </div>
              </transition>
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
              </div>

              <!-- Progress Bar -->
              <div>
                <Slider
                  @update:model-value='$event && $emit("seek", $event[0])'
                  :max='100'
                  :model-value='[progress.value]'
                  :step='0.1'
                  class='w-full'
                />
                <div class='flex justify-between text-xs text-muted-foreground mt-2'>
                  <span>{{ formatTime(currentTime.value) }}</span>
                  <span>{{ formatTime(duration.value) }}</span>
                </div>
              </div>

              <!-- Player Controls -->
              <div class='flex items-center justify-center space-x-4'>
                <Button
                  @click='$emit("toggle-shuffle")'
                  :class="[isShuffled.value ? 'text-primary' : 'text-muted-foreground']"
                  size='icon'
                  variant='ghost'
                >
                  <Shuffle class='w-5 h-5' />
                </Button>
                <Button
                  @click='$emit("previous-song")'
                  :disabled='!hasPrevious.value'
                  size='icon'
                  variant='ghost'
                >
                  <SkipBack class='w-6 h-6' />
                </Button>
                <Button
                  @click='$emit("toggle-play-pause")'
                  class='rounded-full w-16 h-16'
                >
                  <Pause v-if='isPlaying.value' class='w-8 h-8' />
                  <Play v-else class='w-8 h-8' />
                </Button>
                <Button
                  @click='$emit("next-song")'
                  :disabled='!hasNext.value'
                  size='icon'
                  variant='ghost'
                >
                  <SkipForward class='w-6 h-6' />
                </Button>
                <Button
                  @click='$emit("toggle-repeat")'
                  :class="[repeatMode.value !== 'none' ? 'text-primary' : 'text-muted-foreground']"
                  size='icon'
                  variant='ghost'
                >
                  <Repeat1 v-if="repeatMode.value === 'one'" class='w-5 h-5' />
                  <Repeat v-else class='w-5 h-5' />
                </Button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </transition>
</template>

<script setup lang="ts">
  import { ref, watch } from 'vue'
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
  import { MusicItem } from '@/types'
  import { PropType, Ref } from 'vue'

  const props = defineProps({
    show: {
      type:     Boolean,
      required: true,
    },
    song: {
      type:    Object as PropType<MusicItem | null>,
      default: null,
    },
    isPlaying: {
      type:     Object as PropType<Ref<boolean>>,
      required: true,
    },
    progress: {
      type:     Object as PropType<Ref<number>>,
      required: true,
    },
    currentTime: {
      type:     Object as PropType<Ref<number>>,
      required: true,
    },
    duration: {
      type:     Object as PropType<Ref<number>>,
      required: true,
    },
    isShuffled: {
      type:     Object as PropType<Ref<boolean>>,
      required: true,
    },
    repeatMode: {
      type:     Object as PropType<Ref<'none' | 'all' | 'one'>>,
      required: true,
    },
    hasPrevious: {
      type:     Object as PropType<Ref<boolean>>,
      required: true,
    },
    hasNext: {
      type:     Object as PropType<Ref<boolean>>,
      required: true,
    },
    startWithLyrics: {
      type:    Boolean,
      default: false,
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

  const onLyricsLoaded = (lyricsFound: boolean) => {
    hasLyrics.value = lyricsFound
  }

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
    if (props.duration.value > 0) {
      const percentage = (time / props.duration.value) * 100
      emit('seek', percentage)
    }
  }

  const formatTime = (seconds: number) => {
    const mins = Math.floor(seconds / 60)
    const secs = Math.floor(seconds % 60)
    return `${mins}:${secs.toString().padStart(2, '0')}`
  }
</script>

<style scoped>
.slide-fade-enter-active,
.slide-fade-leave-active {
  transition: all 0.3s ease-out;
}

.slide-fade-enter-from,
.slide-fade-leave-to {
  transform: translateY(100%);
  opacity: 0;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.album-art-container {
  max-height: min(40vh, 50rem);
  max-width: min(50vw, 50rem);
  height: min(40vh, 50vw, 50rem);
  width: min(40vh, 50vw, 50rem);
}
</style>
