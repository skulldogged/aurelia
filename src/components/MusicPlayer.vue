<template>
  <div v-if='currentSong' class='bg-gray-300 dark:bg-black p-2'>
    <div class='mx-auto'>
      <div class='grid grid-cols-3 items-center px-2'>
        <!-- Song Info (Left) -->
        <div class='flex items-center'>
          <div class='flex items-center space-x-4'>
            <div class='flex-shrink-0'>
              <img
                v-if='currentSong.albumArtUrl'
                :src='currentSong.albumArtUrl'
                alt='Album art'
                class='w-12 h-12 rounded-md'
              >
              <div v-else class='w-12 h-12 bg-muted rounded-md flex items-center justify-center'>
                <Music2 class='w-6 h-6 text-muted-foreground' />
              </div>
            </div>
            <div class='flex-1 min-w-0'>
              <h3 class='text-foreground font-medium truncate'>
                {{ currentSong.name }}
              </h3>
              <p class='text-muted-foreground text-sm truncate'>
                <template
                  v-if='
                    currentSong.artists
                      && currentSong.artistIds
                      && currentSong.artists.length === currentSong.artistIds.length
                  '
                >
                  <template v-for='(artist, index) in currentSong.artists' :key='currentSong.artistIds[index]'>
                    <router-link
                      :to="{ name: 'artist-detail', params: { artistId: currentSong.artistIds[index] } }"
                      class='hover:underline'
                    >
                      {{ artist }}
                    </router-link>
                    <span v-if='index < currentSong.artists.length - 1'>, </span>
                  </template>
                </template>
                <template v-else>
                  {{ currentSong.artists?.join(', ') || 'Unknown Artist' }}
                </template>
                •
                <router-link
                  v-if='currentSong.album'
                  :to="{ name: 'album-detail', params: { albumName: currentSong.album } }"
                  class='hover:underline'
                >
                  {{ currentSong.album }}
                </router-link>
                <span v-else>{{ 'Unknown Album' }}</span>
              </p>
            </div>
          </div>
        </div>

        <!-- Controls and Seekbar (Middle) -->
        <div class='flex-grow px-4'>
          <div class='flex justify-center'>
            <div class='flex items-center space-x-2'>
              <!-- Previous -->
              <Button
                @click='previousSong'
                :disabled='!hasPrevious'
                size='icon'
                variant='ghost'
              >
                <SkipBack class='w-4 h-4' />
              </Button>

              <!-- Play/Pause -->
              <Button
                @click='togglePlayPause'
                :disabled='!audioReady || isBuffering'
                class='rounded-full w-10 h-10'
              >
                <Loader2 v-if='isBuffering' class='w-5 h-5 animate-spin' />
                <Play v-else-if='!isPlaying' class='w-5 h-5' />
                <Pause v-else class='w-5 h-5' />
              </Button>

              <!-- Next -->
              <Button
                @click='nextSong'
                :disabled='!hasNext'
                size='icon'
                variant='ghost'
              >
                <SkipForward class='w-4 h-4' />
              </Button>
            </div>
          </div>
          <!-- Progress Bar -->
          <div class='flex items-center space-x-2 mt-2 text-sm text-muted-foreground'>
            <span>{{ formatTime(currentTime) }}</span>
            <Slider
              @update:model-value='onSeek'
              :max='100'
              :model-value='[progress]'
              :step='0.1'
              class='w-full'
            />
            <span>{{ formatTime(duration) }}</span>
          </div>
        </div>

        <!-- Additional Controls (Right) -->
        <div class='flex justify-end'>
          <div class='flex items-center space-x-2'>
            <!-- Shuffle -->
            <Button
              @click='toggleShuffle'
              :class="[isShuffled ? 'text-primary' : 'text-muted-foreground']"
              size='icon'
              variant='ghost'
            >
              <Shuffle class='w-5 h-5' />
            </Button>

            <!-- Repeat -->
            <Button
              @click='toggleRepeat'
              :class="[repeatMode !== 'none' ? 'text-primary' : 'text-muted-foreground']"
              size='icon'
              variant='ghost'
            >
              <Repeat1 v-if="repeatMode === 'one'" class='w-5 h-5' />
              <Repeat v-else class='w-5 h-5' />
            </Button>

            <!-- Volume -->
            <div class='flex items-center space-x-2'>
              <Button @click='toggleMute' size='icon' variant='ghost'>
                <Volume2 v-if='props.volume > 0.5' class='w-5 h-5' />
                <Volume1 v-else-if='props.volume > 0' class='w-5 h-5' />
                <VolumeX v-else class='w-5 h-5' />
              </Button>
              <Slider
                @update:model-value='onVolumeInput'
                :max='1'
                :model-value='[props.volume]'
                :step='0.01'
                class='w-24'
              />
            </div>

            <!-- Queue -->
            <Button @click="$emit('toggle-queue')" size='icon' variant='ghost'>
              <ListMusic class='w-5 h-5' />
            </Button>
          </div>
        </div>
      </div>

      <!-- Hidden Audio Element -->
      <audio
        @canplay='onCanPlay'
        @ended='onEnded'
        @error='onError'
        @loadedmetadata='onLoadedMetadata'
        @timeupdate='onTimeUpdate'
        ref='audioElement'
        preload='metadata'
      />
    </div>
  </div>
</template>

<script setup lang="ts">
  import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
  import { invoke } from '@tauri-apps/api/core'
  import {
    Music2,
    SkipBack,
    Play,
    Pause,
    SkipForward,
    Shuffle,
    Repeat,
    Repeat1,
    Volume2,
    Volume1,
    VolumeX,
    Loader2,
    ListMusic,
  } from 'lucide-vue-next'
  import { Button } from '@/components/ui/button'
  import { Slider } from '@/components/ui/slider'
  import { MusicItem } from '@/types'

  const props = defineProps<{
    currentSong: MusicItem | null
    serverUrl:   string
    token:       string
    playlist:    MusicItem[]
    volume:      number
  }>()

  const emit = defineEmits<{
    songEnded:         []
    songChanged:       [song: MusicItem]
    updateCurrentSong: [song: MusicItem | null, isPlaying: boolean]
    volumeChanged:     [volume: number]
    'toggle-queue':    []
  }>()

  // Audio element
  const audioElement = ref<HTMLAudioElement | null>(null)

  // Player state
  const isPlaying = ref(false)
  const currentTime = ref(0)
  const duration = ref(0)
  const progress = ref(0)
  const audioReady = ref(false)
  const isBuffering = ref(false)
  const isSeeking = ref(false)

  // Playback controls
  const isShuffled = ref(false)
  const repeatMode = ref<'none' | 'all' | 'one'>('none')
  const currentIndex = ref(0)

  // Computed properties
  const hasPrevious = computed(() => {
    return props.playlist.length > 1 && currentIndex.value > 0
  })

  const hasNext = computed(() => {
    return props.playlist.length > 1 && currentIndex.value > -1 && currentIndex.value < props.playlist.length - 1
  })

  // Methods
  const formatTime = (seconds: number) => {
    const mins = Math.floor(seconds / 60)
    const secs = Math.floor(seconds % 60)
    return `${mins}:${secs.toString().padStart(2, '0')}`
  }

  const updateProgress = () => {
    if (audioElement.value && duration.value > 0) {
      progress.value = (currentTime.value / duration.value) * 100
    }
  }

  const onLoadedMetadata = () => {
    if (audioElement.value) {
      console.log('[MusicPlayer] Audio metadata loaded. Duration:', audioElement.value.duration)
      // Prioritize pre-fetched duration, fall back to audio element's duration
      if (props.currentSong?.duration) {
        duration.value = props.currentSong.duration
      } else {
        duration.value = audioElement.value.duration || 0
      }

      currentTime.value = 0
      progress.value = 0
    }
  }

  const onTimeUpdate = () => {
    if (audioElement.value && !isSeeking.value) {
      currentTime.value = audioElement.value.currentTime
      updateProgress()
    }
  }

  const onCanPlay = () => {
    console.log('[MusicPlayer] Audio can play.')
    audioReady.value = true
  }

  const onError = () => {
    audioReady.value = false
    console.error('[MusicPlayer] Audio playback error:', audioElement.value?.error)
    isBuffering.value = false
  }

  const onEnded = () => {
    if (repeatMode.value === 'one') {
      // Repeat current song
      if (audioElement.value) {
        audioElement.value.currentTime = 0
        audioElement.value.play()
      }
    } else if (repeatMode.value === 'all' || hasNext.value) {
      // Play next song
      nextSong()
    } else {
      // Stop playback
      isPlaying.value = false
      emit('updateCurrentSong', props.currentSong, false)
    }
  }

  const togglePlayPause = async () => {
    if (!audioElement.value || !audioReady.value) return

    try {
      if (isPlaying.value) {
        audioElement.value.pause()
        isPlaying.value = false
      } else {
        await audioElement.value.play()
        isPlaying.value = true
      }
      emit('updateCurrentSong', props.currentSong, isPlaying.value)
    } catch (error) {
      console.error('Playback error:', error)
    }
  }

  const debounce = <P extends unknown[]>(
    func: (...args: P) => void,
    wait: number,
    ): ((...args: P) => void) => {
    let timeout: number | undefined

    return (...args: P) => {
      const later = () => {
        clearTimeout(timeout)
        func(...args)
      }

      clearTimeout(timeout)
      timeout = setTimeout(later, wait)
    }
  }

  const seekTo = (seekTime: number) => {
    if (audioElement.value && isFinite(seekTime))
      audioElement.value.currentTime = seekTime
  }

  const debouncedSeek = debounce((seekTime: number) => {
    seekTo(seekTime)
    isSeeking.value = false
  }, 200)

  const onSeek = (value: number[] | undefined) => {
    if (!value || !audioElement.value || !audioReady.value) return

    isSeeking.value = true
    const progressValue = value[0]
    const seekTime = (progressValue / 100) * duration.value

    // Update UI immediately
    progress.value = progressValue
    currentTime.value = seekTime

    debouncedSeek(seekTime)
  }

  const onVolumeInput = (value: number[] | undefined) => {
    if (!value) return
    const newVolume = value[0]
    emit('volumeChanged', newVolume)
  }

  const toggleMute = () => {
    if (props.volume > 0) {
      emit('volumeChanged', 0)
    } else {
      emit('volumeChanged', 0.5) // Restore to 50%
    }
  }

  const previousSong = () => {
    if (hasPrevious.value) {
      const newIndex = currentIndex.value - 1
      playSongAtIndex(newIndex)
    }
  }

  const nextSong = () => {
    if (hasNext.value) {
      let newIndex: number

      if (isShuffled.value) {
        // Simple shuffle - pick random song
        newIndex = Math.floor(Math.random() * props.playlist.length)
      } else {
        newIndex = currentIndex.value + 1
      }

      playSongAtIndex(newIndex)
    } else if (repeatMode.value === 'all') {
      // Go back to first song
      playSongAtIndex(0)
    }
  }

  const playSongAtIndex = (index: number) => {
    if (index >= 0 && index < props.playlist.length) {
      currentIndex.value = index
      const song = props.playlist[index]
      emit('songChanged', song)
    }
  }

  const toggleShuffle = () => {
    isShuffled.value = !isShuffled.value
  }

  const toggleRepeat = () => {
    if (repeatMode.value === 'none') {
      repeatMode.value = 'all'
    } else if (repeatMode.value === 'all') {
      repeatMode.value = 'one'
    } else {
      repeatMode.value = 'none'
    }
  }

  // Watch for song or playlist changes to update the current index
  watch(
    [() => props.currentSong, () => props.playlist],
    ([newSong, newPlaylist]) => {
      if (newSong && newPlaylist) {
        const index = newPlaylist.findIndex(song => song.id === newSong.id)
        currentIndex.value = index
      } else {
        currentIndex.value = -1
      }
    },
    { deep: true, immediate: true },
  )

  // Initialize volume
  onMounted(() => {
    if (audioElement.value)
      audioElement.value.volume = props.volume

    const newSong = props.currentSong
    if (newSong && audioElement.value) {
      console.log('[MusicPlayer] Mounting for new song:', newSong.name)
      audioReady.value = false
      isBuffering.value = true

      // Get stream URL from backend
      invoke<string>('get_audio_stream_url', {
        serverUrl: props.serverUrl,
        token:     props.token,
        itemId:    newSong.id,
      })
        .then(streamUrl => {
          console.log('[MusicPlayer] Fetched stream URL:', streamUrl)

          if (audioElement.value) {
            audioElement.value.src = streamUrl
            audioElement.value.load()
            audioElement.value.play()
              .then(() => {
                isPlaying.value = true
                emit('updateCurrentSong', props.currentSong, true)
              })
              .catch(error => {
                console.error('[MusicPlayer] Failed to play audio:', error)
              })
              .finally(() => {
                isBuffering.value = false
              })
          }
        })
        .catch(error => {
          console.error('[MusicPlayer] Failed to load audio:', error)
          isBuffering.value = false
        })
    }
  })

  watch(() => props.volume, newVolume => {
    if (audioElement.value) {
      audioElement.value.volume = newVolume
    }
  })

  onUnmounted(() => {
    if (audioElement.value) {
      audioElement.value.pause()
    }
  })
</script>

<style scoped>
.slider::-webkit-slider-thumb {
  appearance: none;
  height: 12px;
  width: 12px;
  border-radius: 50%;
  background: #4b5563;
  cursor: pointer;
}

.slider::-moz-range-thumb {
  height: 12px;
  width: 12px;
  border-radius: 50%;
  background: #4b5563;
  cursor: pointer;
  border: none;
}
</style>
