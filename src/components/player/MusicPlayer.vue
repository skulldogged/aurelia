<template>
  <div v-if='currentSong' class='bg-sidebar p-2'>
    <div class='mx-auto'>
      <div class='grid grid-cols-3 items-center px-2'>
        <!-- Song Info (Left) -->
        <div class='flex items-center'>
          <div class='flex items-center space-x-4'>
            <div class='flex-shrink-0'>
              <img
                @click="$emit('toggle-fullscreen')"
                v-if='currentSong.albumArtUrl'
                :src='currentSong.albumArtUrl'
                alt='Album art'
                class='w-12 h-12 rounded-md cursor-pointer'
              >
              <div
                @click="$emit('toggle-fullscreen')"
                v-else
                class='w-12 h-12 bg-muted rounded-md flex items-center justify-center cursor-pointer'
              >
                <Music2 class='w-6 h-6 text-muted-foreground' />
              </div>
            </div>
            <div class='flex-1 min-w-0'>
              <h3 class='text-foreground font-medium truncate select-text'>
                {{ currentSong.name }}
              </h3>
              <p class='text-muted-foreground text-sm truncate select-text'>
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
            <!-- Fullscreen -->
            <Button @click="$emit('toggle-fullscreen')" size='icon' variant='ghost'>
              <Expand class='w-5 h-5' />
            </Button>
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

      <!-- Hidden Audio Elements -->
      <audio
        @canplaythrough='onCanPlay(0)'
        @ended='onEnded(0)'
        @error='onError(0)'
        @loadedmetadata='onLoadedMetadata(0)'
        @timeupdate='onTimeUpdate(0)'
        ref='audioPlayer1'
        preload='auto'
      />
      <audio
        @canplaythrough='onCanPlay(1)'
        @ended='onEnded(1)'
        @error='onError(1)'
        @loadedmetadata='onLoadedMetadata(1)'
        @timeupdate='onTimeUpdate(1)'
        ref='audioPlayer2'
        preload='auto'
      />
    </div>
  </div>
</template>

<script setup lang="ts">
  import { ref, computed, watch, onMounted, onUnmounted, watchEffect } from 'vue'
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
    Expand,
  } from 'lucide-vue-next'
  import { Button } from '@/components/ui/button'
  import { Slider } from '@/components/ui/slider'
  import { MusicItem } from '@/types'
  import { usePlayerState } from '@/composables/usePlayerState'

  const props = defineProps<{
    currentSong: MusicItem | null
    serverUrl:   string
    token:       string
    playlist:    MusicItem[]
    volume:      number
  }>()

  const emit = defineEmits<{
    songEnded:           []
    songChanged:         [song: MusicItem]
    updateCurrentSong:   [song: MusicItem | null, isPlaying: boolean]
    volumeChanged:       [volume: number]
    'toggle-queue':      [],
    'toggle-fullscreen': [],
  }>()

  const {
    isPlaying,
    currentTime,
    duration,
    progress,
    isShuffled,
    repeatMode,
    hasPrevious,
    hasNext,
  } = usePlayerState()

  // Audio elements
  const audioPlayer1 = ref<HTMLAudioElement | null>(null)
  const audioPlayer2 = ref<HTMLAudioElement | null>(null)
  const activePlayerIndex = ref(0)
  const players = [audioPlayer1, audioPlayer2]
  const activePlayer = computed(() => players[activePlayerIndex.value].value)
  const nextPlayer = computed(() => players[1 - activePlayerIndex.value].value)

  // Player state
  const audioReady = ref(false)
  const isBuffering = ref(false)
  const nextSongReady = ref(false)
  const isGaplessTransition = ref(false)

  // Playback controls
  const currentIndex = ref(0)

  // Computed properties
  watchEffect(() => {
    hasPrevious.value = props.playlist.length > 1 && currentIndex.value > 0
    hasNext.value
      = props.playlist.length > 1
        && currentIndex.value > -1
        && currentIndex.value < props.playlist.length - 1
  })

  const nextSongInQueue = computed(() => {
    if (!hasNext.value) return null

    let nextIndex
    if (isShuffled.value) {
      nextIndex = Math.floor(Math.random() * props.playlist.length)
    } else {
      nextIndex = currentIndex.value + 1
    }
    return props.playlist[nextIndex]
  })

  // Methods
  const formatTime = (seconds: number) => {
    const mins = Math.floor(seconds / 60)
    const secs = Math.floor(seconds % 60)
    return `${mins}:${secs.toString().padStart(2, '0')}`
  }

  const updateProgress = () => {
    if (activePlayer.value && duration.value > 0) {
      progress.value = (currentTime.value / duration.value) * 100
    }
  }

  const onLoadedMetadata = (playerIndex: number) => {
    const player = players[playerIndex].value
    if (player) {
      if (playerIndex === activePlayerIndex.value) {
        if (props.currentSong?.duration) {
          duration.value = props.currentSong.duration
        } else {
          duration.value = player.duration || 0
        }
        currentTime.value = 0
        progress.value = 0
      }
    }
  }

  const onTimeUpdate = (playerIndex: number) => {
    if (playerIndex === activePlayerIndex.value) {
      const player = players[playerIndex].value
      if (player) {
        currentTime.value = player.currentTime
        updateProgress()
      }
    }
  }

  const onCanPlay = (playerIndex: number) => {
    if (playerIndex === activePlayerIndex.value) {
      audioReady.value = true
    } else {
      nextSongReady.value = true
    }
  }

  const onError = (playerIndex: number) => {
    const player = players[playerIndex].value
    console.error(`[MusicPlayer] Audio playback error on player ${playerIndex}:`, player?.error)
    if (playerIndex === activePlayerIndex.value) {
      audioReady.value = false
      isBuffering.value = false
    }
  }

  const onEnded = (playerIndex: number) => {
    if (playerIndex !== activePlayerIndex.value) return

    if (repeatMode.value === 'one') {
      if (activePlayer.value) {
        activePlayer.value.currentTime = 0
        activePlayer.value.play()
      }
    } else if (nextSongReady.value && nextSongInQueue.value) {
      // Switch players for gapless playback
      activePlayer.value?.pause()
      isGaplessTransition.value = true
      activePlayerIndex.value = 1 - activePlayerIndex.value

      emit('songChanged', nextSongInQueue.value)
    } else if (repeatMode.value === 'all' || hasNext.value) {
      // Fallback if next song is not ready
      nextSong()
    } else {
      isPlaying.value = false
      emit('updateCurrentSong', props.currentSong, false)
    }
  }

  const togglePlayPause = async () => {
    if (!activePlayer.value || !audioReady.value) return

    try {
      if (isPlaying.value) {
        activePlayer.value.pause()
        isPlaying.value = false
      } else {
        await activePlayer.value.play()
        isPlaying.value = true
      }
      emit('updateCurrentSong', props.currentSong, isPlaying.value)
    } catch (error) {
      console.error('Playback error:', error)
    }
  }

  const onSeek = (value: number[] | undefined) => {
    if (!value || !activePlayer.value || !audioReady.value) return

    const progressValue = value[0]
    const seekTime = (progressValue / 100) * duration.value

    if (isFinite(seekTime)) {
      activePlayer.value.currentTime = seekTime
      currentTime.value = seekTime
      progress.value = progressValue
    }
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

  const loadSong = async (song: MusicItem | null, player: HTMLAudioElement | null) => {
    if (!player || !song) {
      if (player) player.src = ''
      return
    }

    try {
      const streamUrl = await invoke<string>('get_audio_stream_url', {
        serverUrl: props.serverUrl,
        token:     props.token,
        itemId:    song.id,
        container: song.container,
      })
      player.src = streamUrl
      player.load()
    } catch (error) {
      console.error(`[MusicPlayer] Failed to load audio for song ${song.id}:`, error)
    }
  }

  const playManuallyChangedSong = (song: MusicItem) => {
    audioReady.value = false
    nextSongReady.value = false
    isBuffering.value = true

    const execute = async () => {
      await loadSong(song, activePlayer.value)
      if (activePlayer.value) {
        try {
          await activePlayer.value.play()
          isPlaying.value = true
          emit('updateCurrentSong', props.currentSong, true)
        } catch (error) {
          console.error('[MusicPlayer] Failed to play audio:', error)
          isPlaying.value = false
        } finally {
          isBuffering.value = false
        }
      } else {
        isBuffering.value = false
      }
      await loadSong(nextSongInQueue.value, nextPlayer.value)
    }
    execute()
  }

  // Watch for song changes
  watch(() => props.currentSong, (newSong, oldSong) => {
    if (newSong && newSong.id !== oldSong?.id) {
      if (isGaplessTransition.value) {
        isGaplessTransition.value = false
        if (activePlayer.value) {
          isBuffering.value = true
          activePlayer.value.play()
            .then(() => {
              isPlaying.value = true
              emit('updateCurrentSong', newSong, true)
            })
            .catch(error => {
              console.error('[MusicPlayer] Failed to play audio:', error)
              isPlaying.value = false
            })
            .finally(() => {
              isBuffering.value = false
            })
        }
        loadSong(nextSongInQueue.value, nextPlayer.value)
      } else {
        playManuallyChangedSong(newSong)
      }
    } else if (!newSong) {
      if (audioPlayer1.value) { audioPlayer1.value.src = ''; audioPlayer1.value.pause() }
      if (audioPlayer2.value) { audioPlayer2.value.src = ''; audioPlayer2.value.pause() }
      isPlaying.value = false
    }
  })

  // Watch for playlist changes
  watch(
    () => props.playlist,
    newPlaylist => {
      if (props.currentSong) {
        const index = newPlaylist.findIndex(song => song.id === props.currentSong!.id)
        currentIndex.value = index
        loadSong(nextSongInQueue.value, nextPlayer.value)
      } else {
        currentIndex.value = -1
      }
    },
    { deep: true },
  )

  // Initialize volume and first song
  onMounted(() => {
    if (audioPlayer1.value) audioPlayer1.value.volume = props.volume
    if (audioPlayer2.value) audioPlayer2.value.volume = props.volume

    if (props.currentSong) {
      const index = props.playlist.findIndex(song => song.id === props.currentSong!.id)
      currentIndex.value = index
      playManuallyChangedSong(props.currentSong)
    }
  })

  watch(() => props.volume, newVolume => {
    if (audioPlayer1.value) audioPlayer1.value.volume = newVolume
    if (audioPlayer2.value) audioPlayer2.value.volume = newVolume
  })

  onUnmounted(() => {
    if (audioPlayer1.value) audioPlayer1.value.pause()
    if (audioPlayer2.value) audioPlayer2.value.pause()
  })

  defineExpose({
    togglePlayPause,
    previousSong,
    nextSong,
    toggleShuffle,
    toggleRepeat,
    onSeek,
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
