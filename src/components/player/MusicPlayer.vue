<template>
  <div v-if='playerStore.currentSong' class='bg-sidebar px-2 py-3'>
    <div class='mx-auto'>
      <div class='grid grid-cols-3 items-center px-2'>
        <!-- Song Info (Left) -->
        <div
          @mouseenter='onTitleMouseEnter'
          @mouseleave='onTitleMouseLeave'
          :class="['flex items-center space-x-4 min-w-0', shouldMarquee ? 'marquee-enabled' : '']"
        >
          <div @click="$emit('toggle-fullscreen')" class='flex-shrink-0'>
            <ImageLoader
              v-if='playerStore.currentSong'
              :item-id='playerStore.currentSong.albumId || playerStore.currentSong.id'
              :server-url='serverUrl'
              :token='token'
              alt='Album art'
              class='w-12 h-12 rounded-md cursor-pointer'
            >
              <template #fallback>
                <div
                  class='w-12 h-12 bg-muted rounded-md flex items-center justify-center cursor-pointer'
                >
                  <Music2 class='w-6 h-6 text-muted-foreground' />
                </div>
              </template>
            </ImageLoader>
          </div>
          <div class='flex-1 min-w-0'>
            <div ref='titleContainerRef' class='text-foreground font-medium select-text overflow-hidden'>
              <div
                @animationiteration='handleMarqueeIteration'
                ref='marqueeTrackRef'
                :class="[
                  'marquee-track',
                  isMarqueePaused ? 'marquee-paused' : ''
                ]"
                :style='marqueeStyle'
              >
                <span
                  ref='titleTextRef'
                  class='whitespace-nowrap'
                >
                  {{ playerStore.currentSong.name }}
                </span>
                <span
                  v-if='shouldMarquee'
                  aria-hidden='true'
                  class='whitespace-nowrap'
                >
                  {{ playerStore.currentSong.name }}
                </span>
              </div>
            </div>
            <p class='text-muted-foreground text-sm truncate select-text'>
              <template
                v-if='
                  playerStore.currentSong.artists
                    && playerStore.currentSong.artistIds
                    && playerStore.currentSong.artists.length === playerStore.currentSong.artistIds.length
                '
              >
                <template
                  v-for='(artist, index) in playerStore.currentSong.artists'
                  :key='playerStore.currentSong.artistIds[index]'
                >
                  <router-link
                    :to="{ name: 'artist-detail', params: { artistId: playerStore.currentSong.artistIds[index] } }"
                    class='hover:underline'
                  >
                    {{ artist }}
                  </router-link>
                  <span v-if='index < playerStore.currentSong.artists.length - 1'>, </span>
                </template>
              </template>
              <template v-else>
                {{ playerStore.currentSong.artists?.join(', ') || 'Unknown Artist' }}
              </template>
              •
              <router-link
                v-if='playerStore.currentSong.album'
                :to="{ name: 'album-detail', params: { albumName: playerStore.currentSong.album } }"
                class='hover:underline'
              >
                {{ playerStore.currentSong.album }}
              </router-link>
              <span v-else>{{ 'Unknown Album' }}</span>
            </p>
            <p v-if='songFormatInfo' class='text-xs text-muted-foreground/80 truncate select-text'>
              {{ songFormatInfo }}
            </p>
          </div>
        </div>

        <!-- Controls and Seekbar (Middle) -->
        <div class='flex-grow px-4'>
          <div class='flex justify-center'>
            <div class='flex items-center space-x-2'>
              <!-- Shuffle -->
              <Button
                @click='playerStore.toggleShuffle'
                size='icon'
                variant='ghost'
              >
                <Shuffle
                  :class="['w-5 h-5', playerStore.isShuffled ? 'text-foreground' : 'text-muted-foreground']"
                />
              </Button>
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
                :disabled='!playerStore.audioReady || playerStore.isBuffering'
                class='rounded-full w-10 h-10'
              >
                <Loader2 v-if='playerStore.isBuffering' class='w-5 h-5 animate-spin' />
                <Play v-else-if='!playerStore.isPlaying' class='w-5 h-5' />
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
              <!-- Repeat -->
              <Button
                @click='playerStore.cycleRepeatMode'
                size='icon'
                variant='ghost'
              >
                <Repeat1
                  v-if="playerStore.repeatMode === 'one'"
                  class='w-5 h-5 text-foreground'
                />
                <Repeat
                  v-else
                  :class="[
                    'w-5 h-5',
                    playerStore.repeatMode !== 'none' ? 'text-foreground' : 'text-muted-foreground',
                  ]"
                />
              </Button>
            </div>
          </div>
          <!-- Progress Bar -->
          <div class='flex items-center space-x-2 mt-2 text-sm text-muted-foreground'>
            <span>{{ formatTime(playerStore.currentTime) }}</span>
            <Slider
              @update:model-value='onSeek'
              :max='100'
              :model-value='[playerStore.progress]'
              :step='0.1'
              class='w-full'
            />
            <span>{{ formatTime(playerStore.duration) }}</span>
          </div>
        </div>

        <!-- Additional Controls (Right) -->
        <div class='flex justify-end'>
          <div class='flex items-center space-x-2'>
            <!-- Like -->
            <Button
              @click="$emit('toggle-favorite', playerStore.currentSong)"
              size='icon'
              variant='ghost'
            >
              <Heart
                :class="[
                  'w-5 h-5',
                  playerStore.currentSong.isFavorite
                    ? 'text-foreground fill-current'
                    : 'text-muted-foreground',
                ]"
              />
            </Button>
            <!-- Fullscreen -->
            <Button @click="$emit('toggle-fullscreen')" size='icon' variant='ghost'>
              <Expand class='w-5 h-5 text-muted-foreground' />
            </Button>
            <!-- Volume -->
            <div class='flex items-center space-x-2'>
              <button
                @click='playerStore.toggleMute'
                class='text-muted-foreground hover:text-foreground'
              >
                <Volume2 v-if='playerStore.volume > 50' class='h-4 w-4' />
                <Volume1 v-else-if='playerStore.volume > 0' class='h-4 w-4' />
                <VolumeX v-else class='h-4 w-4' />
              </button>
              <Slider
                @update:model-value='onVolumeInput'
                :max='100'
                :model-value='[playerStore.volume * 100]'
                :step='1'
                class='w-20'
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
        @pause='onPause(0)'
        @play='onPlay(0)'
        @timeupdate='onTimeUpdate(0)'
        ref='audioPlayer1'
        preload='auto'
      />
      <audio
        @canplaythrough='onCanPlay(1)'
        @ended='onEnded(1)'
        @error='onError(1)'
        @loadedmetadata='onLoadedMetadata(1)'
        @pause='onPause(1)'
        @play='onPlay(1)'
        @timeupdate='onTimeUpdate(1)'
        ref='audioPlayer2'
        preload='auto'
      />
    </div>
  </div>
</template>

<script setup lang="ts">
  import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
  import { commands } from '@/bindings'
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
    Heart,
  } from 'lucide-vue-next'
  import { Button } from '@/components/ui/button'
  import { Slider } from '@/components/ui/slider'
  import { Song } from '@/bindings'
  import { usePlayerStore } from '@/stores'
  import ImageLoader from '@/components/shared/ImageLoader.vue'
  import { playerLogger } from '@/lib/logger'

  const props = defineProps<{
    serverUrl: string
    token:     string
  }>()

  defineEmits<{
    'toggle-queue':      []
    'toggle-fullscreen': []
    'toggle-favorite':   [song: Song]
  }>()

  const playerStore = usePlayerStore()
  const { getAudioStreamUrl } = commands

  // Audio elements
  const audioPlayer1 = ref<HTMLAudioElement | null>(null)
  const audioPlayer2 = ref<HTMLAudioElement | null>(null)
  const activePlayerIndex = ref(0)
  const players = [audioPlayer1, audioPlayer2]
  const activePlayer = computed(() => players[activePlayerIndex.value].value)
  const nextPlayer = computed(() => players[1 - activePlayerIndex.value].value)

  // Player state (only DOM-related local state)
  const nextSongReady = ref(false)
  const isGaplessTransition = ref(false)

  // Computed properties
  const hasPrevious = computed(() => playerStore.playlist.length > 1 && playerStore.currentIndex > 0)
  const hasNext = computed(() =>
    playerStore.playlist.length > 1
    && playerStore.currentIndex > -1
    && playerStore.currentIndex < playerStore.playlist.length - 1,
  )

  const nextSongInQueue = computed(() => {
    if (!hasNext.value) return null

    let nextIndex
    if (playerStore.isShuffled) {
      nextIndex = Math.floor(Math.random() * playerStore.playlist.length)
    } else {
      nextIndex = playerStore.currentIndex + 1
    }
    return playerStore.playlist[nextIndex]
  })

  const songFormatInfo = computed(() => {
    if (!playerStore.currentSong) return ''
    const parts: string[] = []
    if (playerStore.currentSong.codec) parts.push(playerStore.currentSong.codec.toUpperCase())
    if (playerStore.currentSong.sampleRate) parts.push(`${playerStore.currentSong.sampleRate / 1000} kHz`)
    if (playerStore.currentSong.bitRate) parts.push(`${Math.round(playerStore.currentSong.bitRate / 1000)} kbps`)
    return parts.join(' / ')
  })

  // Marquee title scrolling
  const titleContainerRef = ref<HTMLDivElement | null>(null)
  const titleTextRef = ref<HTMLSpanElement | null>(null)
  const marqueeTrackRef = ref<HTMLDivElement | null>(null)
  const shouldMarquee = ref(false)
  const isMarqueePaused = ref(false)
  const scrollDistance = ref(0)
  const animationDuration = ref(0)
  const marqueeGap = 24 // px
  const marqueeSpeedPxPerSecond = 80
  const marqueePauseMs = 1200
  let marqueePauseTimeoutId: number | null = null

  const marqueeStyle = computed(() => {
    if (!shouldMarquee.value) return {}
    const styleVars: Record<string, string> = {}
    styleVars['--scroll-distance'] = `${scrollDistance.value}px`
    styleVars['--marquee-duration'] = `${animationDuration.value}s`
    styleVars['--marquee-gap'] = `${marqueeGap}px`
    return styleVars
  })

  const measureMarquee = () => {
    const container = titleContainerRef.value
    const textEl = titleTextRef.value
    if (!container || !textEl) return
    const containerWidth = container.clientWidth
    const textWidth = Math.ceil(textEl.scrollWidth)
    shouldMarquee.value = textWidth > containerWidth
    if (shouldMarquee.value) {
      scrollDistance.value = textWidth + marqueeGap
      const duration = scrollDistance.value / marqueeSpeedPxPerSecond
      animationDuration.value = Math.max(8, parseFloat(duration.toFixed(2)))
    } else {
      isMarqueePaused.value = false
    }
  }

  const handleMarqueeIteration = () => {
    if (!shouldMarquee.value) return
    if (marqueePauseTimeoutId) {
      clearTimeout(marqueePauseTimeoutId)
    }
    isMarqueePaused.value = true
    marqueePauseTimeoutId = window.setTimeout(() => {
      isMarqueePaused.value = false
    }, marqueePauseMs)
  }

  const onTitleMouseEnter = () => {
    if (!shouldMarquee.value) return
    // Restart from beginning each hover to avoid mid-loop starts
    isMarqueePaused.value = false
  }

  const onTitleMouseLeave = () => {
    // Stop animation when not hovered
    isMarqueePaused.value = true
  }

  // Methods
  const formatTime = (seconds: number) => {
    const mins = Math.floor(seconds / 60)
    const secs = Math.floor(seconds % 60)
    return `${mins}:${secs.toString().padStart(2, '0')}`
  }

  const updateProgress = () => {
    if (activePlayer.value && playerStore.duration > 0) {
      // progress is computed in store, no need to calculate here
    }
  }

  const onLoadedMetadata = (playerIndex: number) => {
    const player = players[playerIndex].value
    if (player) {
      if (playerIndex === activePlayerIndex.value) {
        if (playerStore.currentSong?.duration) {
          playerStore.setDuration(playerStore.currentSong.duration)
        } else {
          playerStore.setDuration(player.duration || 0)
        }
        playerStore.setCurrentTime(0)
        // progress is computed, no need to set
      }
    }
  }

  const onTimeUpdate = (playerIndex: number) => {
    if (playerIndex === activePlayerIndex.value) {
      const player = players[playerIndex].value
      if (player) {
        playerStore.setCurrentTime(player.currentTime)
        updateProgress()
      }
    }
  }

  const onCanPlay = (playerIndex: number) => {
    if (playerIndex === activePlayerIndex.value) {
      playerStore.setAudioReady(true)
    } else {
      nextSongReady.value = true
    }
  }

  const onError = (playerIndex: number) => {
    const player = players[playerIndex].value
    playerLogger.error(`Audio playback error on player ${playerIndex}:`, player?.error)

    if (playerIndex === activePlayerIndex.value) {
      playerStore.setAudioReady(false)
      playerStore.setBuffering(false)
    }
  }

  const onPlay = (playerIndex: number) => {
    if (playerIndex === activePlayerIndex.value) {
      playerStore.play()
    }
  }

  const onPause = (playerIndex: number) => {
    if (playerIndex === activePlayerIndex.value) {
      playerStore.pause()
    }
  }

  const onEnded = (playerIndex: number) => {
    if (playerIndex !== activePlayerIndex.value) return

    if (playerStore.repeatMode === 'one') {
      if (activePlayer.value) {
        activePlayer.value.currentTime = 0
        activePlayer.value.play()
        // Store state will be updated by onPlay event
      }
    } else if (nextSongReady.value && nextSongInQueue.value) {
      // Switch players for gapless playback
      activePlayer.value?.pause()
      // Store state will be updated by onPause event
      isGaplessTransition.value = true
      activePlayerIndex.value = 1 - activePlayerIndex.value

      // Reset player state for the new song
      playerStore.setCurrentTime(0)
      playerStore.setDuration(nextSongInQueue.value?.duration || 0)
      playerStore.setCurrentSong(nextSongInQueue.value)
    } else if (playerStore.repeatMode === 'all' || hasNext.value) {
      // Fallback if next song is not ready
      nextSong()
    } else {
      // End of playlist - pause and update store
      activePlayer.value?.pause()
      // Store state will be updated by onPause event
    }
  }

  const togglePlayPause = async () => {
    if (!activePlayer.value || !playerStore.audioReady) return

    try {
      if (playerStore.isPlaying) {
        activePlayer.value.pause()
        // Store state will be updated by onPause event
      } else {
        await activePlayer.value.play()
        // Store state will be updated by onPlay event
      }
    } catch (error) {
      playerLogger.error('Playback error:', error)
    }
  }

  const onSeek = (value: number[] | undefined) => {
    if (!value || !activePlayer.value || !playerStore.audioReady) return

    const progressValue = value[0]
    const seekTime = (progressValue / 100) * playerStore.duration

    if (isFinite(seekTime)) {
      activePlayer.value.currentTime = seekTime
      playerStore.setCurrentTime(seekTime)
      // progress is computed from store, no need to set directly
    }
  }

  const onVolumeInput = (value: number[] | undefined) => {
    if (!value) return
    const newVolume = value[0]
    playerStore.setVolume(newVolume / 100)
  }

  const previousSong = () => {
    if (hasPrevious.value) {
      playerStore.previousSong()
    }
  }

  const nextSong = () => {
    if (hasNext.value) {
      playerStore.nextSong()
    } else if (playerStore.repeatMode === 'all') {
      // Go back to first song
      playSongAtIndex(0)
    }
  }

  const playSongAtIndex = (index: number) => {
    if (index < 0 || index >= playerStore.playlist.length) return

    playerStore.playSongAtIndex(index)

    // Load and play the new song
    loadSong(playerStore.playlist[index], activePlayer.value)
  }

  const loadSong = async (song: Song | null, player: HTMLAudioElement | null) => {
    if (!player || !song) {
      if (player && player.src && player.src !== '')
        player.src = ''

      return
    }

    try {
      const streamResult = await getAudioStreamUrl(
        props.serverUrl,
        props.token,
        song.id,
        song.container,
      )
      if (streamResult.status === 'error') {
        playerLogger.error('Failed to get audio stream URL:', streamResult.error)
        throw new Error(streamResult.error)
      }
      player.src = streamResult.data
      player.load()

      // Reset states when loading active player
      if (player === activePlayer.value) {
        playerStore.setAudioReady(false)
        playerStore.setBuffering(true)
      }
    } catch (error) {
      playerLogger.error(`Failed to load audio for song ${song.name} (ID: ${song.id}):`, error)
    }
  }

  const playManuallyChangedSong = (song: Song) => {
    playerStore.setAudioReady(false)
    nextSongReady.value = false
    playerStore.setBuffering(true)

    const execute = async () => {
      await loadSong(song, activePlayer.value)

      if (activePlayer.value) {
        try {
          await activePlayer.value.play()
          // Store state will be updated by onPlay event
        } catch (error) {
          playerLogger.error('Failed to play audio:', error)
          playerStore.pause()
        } finally {
          playerStore.setBuffering(false)
        }
      } else {
        playerStore.setBuffering(false)
      }

      // Only load next song if there actually is one
      if (nextSongInQueue.value) {
        await loadSong(nextSongInQueue.value, nextPlayer.value)
      }
    }
    execute()
  }

  // Watch for song changes
  watch(() => playerStore.currentSong, (newSong, oldSong) => {

    if (newSong && newSong.id !== oldSong?.id) {
      const newIndex = playerStore.playlist.findIndex(s => s.id === newSong.id)
      if (newIndex !== -1)
        playerStore.setCurrentIndex(newIndex)

      if (isGaplessTransition.value) {
        isGaplessTransition.value = false
        if (activePlayer.value) {
          playerStore.setBuffering(true)
          activePlayer.value.play()
            .then(() => {
              // Store state will be updated by onPlay event
            })
            .catch(error => {
              playerLogger.error('Failed to play audio:', error)
              playerStore.pause()
            })
            .finally(() => {
              playerStore.setBuffering(false)
            })
        }
        loadSong(nextSongInQueue.value, nextPlayer.value)
      } else {
        playManuallyChangedSong(newSong)
      }
    } else if (!newSong) {
      if (audioPlayer1.value) { audioPlayer1.value.src = ''; audioPlayer1.value.pause() }
      if (audioPlayer2.value) { audioPlayer2.value.src = ''; audioPlayer2.value.pause() }
      playerStore.pause()
    }
  })

  // Watch for playlist changes
  watch(
    () => playerStore.playlist,
    newPlaylist => {

      if (playerStore.currentSong) {
        const index = newPlaylist.findIndex(song => song.id === playerStore.currentSong!.id)
        playerStore.setCurrentIndex(index)
        // Only load next song if there actually is one
        if (nextSongInQueue.value) {
          loadSong(nextSongInQueue.value, nextPlayer.value)
        }
      } else {
        playerStore.setCurrentIndex(-1)
      }
    },
    { deep: true },
  )

  // Initialize volume and first song
  onMounted(() => {
    if (audioPlayer1.value) audioPlayer1.value.volume = playerStore.volume
    if (audioPlayer2.value) audioPlayer2.value.volume = playerStore.volume

    if (playerStore.currentSong) {
      const index = playerStore.playlist.findIndex(song => song.id === playerStore.currentSong!.id)
      playerStore.setCurrentIndex(index)
      playManuallyChangedSong(playerStore.currentSong)
    }

    // Measure marquee after mount and on resize
    measureMarquee()
    window.addEventListener('resize', measureMarquee)
  })

  watch(() => playerStore.volume, newVolume => {
    if (audioPlayer1.value) audioPlayer1.value.volume = newVolume
    if (audioPlayer2.value) audioPlayer2.value.volume = newVolume
  })

  onUnmounted(() => {
    if (audioPlayer1.value) audioPlayer1.value.pause()
    if (audioPlayer2.value) audioPlayer2.value.pause()
    window.removeEventListener('resize', measureMarquee)
    if (marqueePauseTimeoutId) clearTimeout(marqueePauseTimeoutId)
  })

  // Re-measure marquee when the song title changes
  watch(() => playerStore.currentSong?.name, async () => {
    await nextTick()
    measureMarquee()
  })

  defineExpose({
    togglePlayPause,
    previousSong,
    nextSong,
    toggleShuffle: playerStore.toggleShuffle,
    toggleRepeat:  playerStore.cycleRepeatMode,
    toggleMute:    playerStore.toggleMute,
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

/* Marquee title scrolling */
.marquee-track {
  display: inline-flex;
  align-items: center;
  gap: var(--marquee-gap, 24px);
  will-change: transform;
  transform: translateX(0);
}

.marquee-running {
  animation: marquee-scroll var(--marquee-duration, 10s) linear infinite;
}

.marquee-paused {
  animation-play-state: paused;
}

/* Only animate when container is hovered */
.marquee-enabled:hover .marquee-track {
  animation: marquee-scroll var(--marquee-duration, 10s) linear infinite;
}

@keyframes marquee-scroll {
  0% {
    transform: translateX(0);
  }
  100% {
    transform: translateX(calc(-1 * var(--scroll-distance, 300px)));
  }
}
</style>
