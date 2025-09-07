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
        <!-- Header -->
        <div class='flex-shrink-0 grid grid-cols-3 items-center p-4'>
          <Button
            @click="$emit('close')"
            class='justify-self-start'
            size='icon'
            variant='ghost'
          >
            <ChevronDown class='w-6 h-6' />
          </Button>
          <div class='text-center'>
            <p
              :class="[
                'text-sm uppercase text-muted-foreground',
                { 'invisible': showLyrics }
              ]"
            >
              Playing From Album
            </p>
            <h2 class='font-bold text-foreground'>
              <template v-if='!showLyrics'>
                {{ song?.album || 'Unknown Album' }}
              </template>
              <template v-else>
                Lyrics
              </template>
            </h2>
          </div>
          <Button
            @click='showLyrics = !showLyrics'
            class='justify-self-end'
            size='icon'
            variant='ghost'
          >
            <Mic2 v-if='!showLyrics' class='w-5 h-5' />
            <Album v-else class='w-5 h-5' />
          </Button>
        </div>

        <!-- Main Content -->
        <div class='flex-grow flex flex-col items-center justify-center p-8 gap-8 overflow-hidden'>
          <!-- Album Art & Info -->
          <div v-show='!showLyrics' class='contents'>
            <div class='relative'>
              <img
                v-if='song?.albumArtUrl'
                :src='song.albumArtUrl'
                alt='Album art'
                class='w-64 h-64 md:w-80 md:h-80 rounded-lg shadow-2xl'
              >
              <div
                v-else
                class='w-64 h-64 md:w-80 md:h-80 bg-muted rounded-lg flex items-center justify-center'
              >
                <Music2 class='w-24 h-24 text-muted-foreground' />
              </div>
            </div>
            <div class='w-full max-w-md text-center'>
              <h1 class='text-3xl font-bold text-foreground'>
                {{ song?.name || 'Unknown Song' }}
              </h1>
              <p class='text-lg text-muted-foreground'>
                {{ song?.artists?.join(', ') || 'Unknown Artist' }}
              </p>
            </div>
          </div>

          <!-- Lyrics View -->
          <LyricsView
            v-show='showLyrics'
            :current-time='currentTime.value'
            :song='song'
            class='w-full h-full'
          />
        </div>

        <!-- Footer Controls -->
        <div class='flex-shrink-0 p-8 pt-0 w-full max-w-md mx-auto'>
          <!-- Progress Bar -->
          <div>
            <Slider
              @update:model-value='$emit("seek", $event)'
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
          <div class='flex items-center justify-center space-x-4 mt-4'>
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
      type:     Object as PropType<MusicItem | null>,
      required: true,
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

  defineEmits<{
    (e: 'close'): void
    (e: 'toggle-play-pause'): void
    (e: 'previous-song'): void
    (e: 'next-song'): void
    (e: 'toggle-shuffle'): void
    (e: 'toggle-repeat'): void
    (e: 'seek', value: number[]): void
  }>()

  const showLyrics = ref(false)

  watch(() => props.show, newVal => {
    if (newVal) {
      if (props.startWithLyrics) {
        showLyrics.value = true
      }
    } else {
      // Reset when closing
      showLyrics.value = false
    }
  })

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
</style>
