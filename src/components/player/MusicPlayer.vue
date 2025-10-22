<script setup lang="ts">
  import {
    Expand,
    Heart,
    ListMusic,
    Loader2,
    Mic2,
    MoreHorizontal,
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
  import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'

  import { commands } from '@/bindings'
  import { Song } from '@/bindings'
  import AudioVisualizer from '@/components/player/AudioVisualizer.vue'
  import ImageLoader from '@/components/shared/ImageLoader.vue'
  import Button from '@/components/ui/Button.vue'
  import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
  } from '@/components/ui/dropdown-menu'
  import { Slider } from '@/components/ui/slider'
  import { useAudioEngine } from '@/composables/useAudioEngine'
  import { useSwipe } from '@/composables/useSwipe'
  import { logger } from '@/lib/logger'
  import { isMobilePortrait } from '@/lib/platform'
  import { getSongFormatInfo } from '@/lib/utils'
  import { usePlayerStore } from '@/stores'

  const props = defineProps<{
    isEqualizerOpen?: boolean
    isLyricsOpen?:    boolean
    isQueueOpen?:     boolean
    serverUrl:        string
    token:            string
  }>()

  const emit = defineEmits<{
    'swipe-progress':    [progress: null | {
      deltaY:    number
      direction: 'down' | 'left' | 'right' | 'up' | null
      startY:    number
    }]
    'toggle-equalizer':  []
    'toggle-favorite':   [song: Song]
    'toggle-fullscreen': []
    'toggle-lyrics':     []
    'toggle-queue':      []
  }>()

  const playerStore = usePlayerStore()

  const {
    activePlayer,
    audioPlayer1,
    audioPlayer2,
    initializePlayer,
    isGaplessTransition,
    loadSong,
    nextPlayer,
    nextSong,
    nextSongInQueue,
    onCanPlay,
    onEnded,
    onError,
    onLoadedMetadata,
    onPause,
    onPlay,
    onTimeUpdate,
    playManuallyChangedSong,
    startWebAudioTimeUpdates,
    stopWebAudioTimeUpdates,
    useWebAudio,
    webAudioPlayer,
  } = useAudioEngine(props)

  const containerRef = ref<HTMLDivElement | null>(null)
  const resizeObserver = ref<null | ResizeObserver>(null)
  const volumePopupRef = ref<HTMLDivElement | null>(null)
  const isVolumePopupVisible = ref(false)

  const hasPrevious = computed(() => playerStore.playlist.length > 1 && playerStore.currentIndex > 0)
  const hasNext = computed(() =>
    playerStore.playlist.length > 1
    && playerStore.currentIndex > -1
    && playerStore.currentIndex < playerStore.playlist.length - 1,
  )

  const hasLyrics = computed(() => playerStore.hasLyrics)

  watch(() => playerStore.currentSong, async newSong => {
    if (newSong) {
      if (newSong.lyrics != null && newSong.lyrics.trim() !== '') {
        playerStore.setHasLyrics(true)
      } else {
        if (newSong.artists && newSong.artists.length > 0) {
          try {
            const result = await commands.getLyrics(
              newSong.id,
              newSong.artists[0],
              newSong.name,
              null,
            )

            playerStore.setHasLyrics(result.status === 'ok' && !!result.data && result.data.trim() !== '')
          } catch {
            playerStore.setHasLyrics(false)
          }
        } else {
          playerStore.setHasLyrics(false)
        }
      }
    } else {
      playerStore.setHasLyrics(false)
    }
  }, { immediate: true })

  // Watch for index changes (including switching to same song at different position)
  watch(() => playerStore.currentIndex, (newIndex, oldIndex) => {
    if (newIndex !== oldIndex && playerStore.currentSong) {
      // Reset playback position when switching to same song at different index
      playerStore.setCurrentTime(0)
      if (useWebAudio.value && webAudioPlayer.getIsReady()) {
        webAudioPlayer.seek(0)
      } else if (activePlayer.value) {
        activePlayer.value.currentTime = 0
      }
    }
  })

  const visibleIcons = ref<string[]>([
    'favorite',
    'fullscreen',
    'equalizer',
    'volume',
    'lyrics',
    'queue',
  ])

  const iconWidths = {
    equalizer:  44,
    favorite:   44,
    fullscreen: 44,
    lyrics:     44,
    queue:      44,
    volume:     44,
  }

  const activeView = computed(() =>
    props.isQueueOpen
      ? 'queue'
      : props.isEqualizerOpen
        ? 'equalizer'
        : props.isLyricsOpen
          ? 'lyrics'
          : null,
  )

  const updateVisibleIcons = (): void => {
    if (!containerRef.value) return

    const containerWidth = containerRef.value.offsetWidth
    const centerControlsWidth = 180
    const leftSectionWidth = 280
    const availableWidth = containerWidth - centerControlsWidth - leftSectionWidth - 60

    let usedWidth = 0
    const newVisibleIcons: string[] = []

    const allIcons = ['volume', 'equalizer', 'fullscreen', 'favorite', 'lyrics', 'queue']

    for (const icon of allIcons) {
      const iconWidth = iconWidths[icon as keyof typeof iconWidths]

      if (usedWidth + iconWidth <= availableWidth) {
        newVisibleIcons.push(icon)
        usedWidth += iconWidth
      }
    }

    if (activeView.value && !newVisibleIcons.includes(activeView.value))
      newVisibleIcons.push(activeView.value)

    visibleIcons.value = newVisibleIcons
  }

  const hasHiddenIcons = computed(() =>
    (
      useWebAudio.value
        ? ['favorite', 'fullscreen', 'volume', 'equalizer', 'lyrics', 'queue']
        : ['favorite', 'fullscreen', 'volume', 'lyrics', 'queue']
    ).some(icon => !visibleIcons.value.includes(icon)),
  )

  const songFormatInfo = computed(() => getSongFormatInfo(playerStore.currentSong))

  const isMobilePortraitMode = computed(() => isMobilePortrait())

  const { startTracking, stopTracking, swipeProgress, updateTracking } = useSwipe({ maxTime: 300 })

  const handleSwipeMove = (event: TouchEvent): void => {
    updateTracking(event)
    // Emit swipe progress for real-time visual feedback
    if (swipeProgress.value && swipeProgress.value.direction) {
      emit('swipe-progress', swipeProgress.value)
    }
  }

  const handleSwipeEnd = (event: TouchEvent): void => {
    const swipeResult = stopTracking(event)
    if (swipeResult?.direction === 'up' && swipeResult.isIntentional) {
      emit('toggle-fullscreen')
    }
    // Clear swipe progress
    emit('swipe-progress', null)
  }

  const closeVolumePopup = (): void => {
    isVolumePopupVisible.value = false
  }

  const handleVolumeClick = (): void => {
    toggleVolumePopup()
  }

  const toggleVolumePopup = (): void => {
    isVolumePopupVisible.value = !isVolumePopupVisible.value
  }

  const handleClickOutside = (event: Event): void => {
    const target = event.target as Element
    const volumeButton = target.closest('[data-volume-button]')
    const insidePopup = volumePopupRef.value && volumePopupRef.value.contains(target)

    if (volumeButton || insidePopup) return

    if (isVolumePopupVisible.value)
      closeVolumePopup()
  }

  const swipeTransform = computed(() => {
    if (!swipeProgress.value || swipeProgress.value.direction !== 'up') return ''
    // Allow unlimited upward movement following the finger
    return `translateY(${Math.min(0, swipeProgress.value.deltaY)}px)`
  })

  const swipeOpacity = computed(() => {
    if (!swipeProgress.value || swipeProgress.value.direction !== 'up') return 0
    // Fade in hint based on upward movement (deltaY is negative for up)
    return Math.min(Math.abs(swipeProgress.value.deltaY) / 100, 0.8)
  })

  watch(isVolumePopupVisible, visible => {
    if (visible)
      document.addEventListener('click', handleClickOutside)
    else
      document.removeEventListener('click', handleClickOutside)
  })

  onUnmounted(() => {
    document.removeEventListener('click', handleClickOutside)
  })

  const titleContainerRef = ref<HTMLDivElement | null>(null)
  const titleTextRef = ref<HTMLSpanElement | null>(null)
  const marqueeTrackRef = ref<HTMLDivElement | null>(null)
  const shouldMarquee = ref(false)
  const isMarqueePaused = ref(false)
  const scrollDistance = ref(0)
  const animationDuration = ref(0)
  const marqueeGap = 24
  const marqueeSpeedPxPerSecond = 80
  const marqueePauseMs = 1200
  let marqueePauseTimeoutId: null | number = null

  const marqueeStyle = computed(() => {
    if (!shouldMarquee.value) return {}
    const styleVars: Record<string, string> = {}
    styleVars['--scroll-distance'] = `${scrollDistance.value}px`
    styleVars['--marquee-duration'] = `${animationDuration.value}s`
    styleVars['--marquee-gap'] = `${marqueeGap}px`
    return styleVars
  })

  const measureMarquee = (): void => {
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

  const handleMarqueeIteration = (): void => {
    if (!shouldMarquee.value) return
    if (marqueePauseTimeoutId) clearTimeout(marqueePauseTimeoutId)

    isMarqueePaused.value = true

    marqueePauseTimeoutId = window.setTimeout(() => {
      isMarqueePaused.value = false
    }, marqueePauseMs)
  }

  const onTitleMouseEnter = (): void => {
    if (!shouldMarquee.value) return
    isMarqueePaused.value = false
  }

  const onTitleMouseLeave = (): void => {
    isMarqueePaused.value = true
  }

  const formatTime = (seconds: number): string =>
    `${Math.floor(seconds / 60)}:${(Math.floor(seconds % 60)).toString().padStart(2, '0')}`

  const togglePlayPause = async (): Promise<void> => {
    if (!playerStore.audioReady) return

    try {
      if (useWebAudio.value) {
        if (playerStore.isPlaying) {
          webAudioPlayer.pause()
          playerStore.pause()
          stopWebAudioTimeUpdates()
        } else {
          const success = await webAudioPlayer.play()
          if (success) {
            playerStore.play()
            startWebAudioTimeUpdates()
          }
        }
      } else {
        if (!activePlayer.value) return

        if (playerStore.isPlaying) {
          activePlayer.value.pause()
        } else {
          await activePlayer.value.play()
        }
      }
    } catch (error) {
      logger.error('Playback error:', error)
    }
  }

  const onSeek = async (value: number[] | undefined): Promise<void> => {
    if (!value || !playerStore.audioReady) return

    const progressValue = value[0]
    const seekTime = (progressValue / 100) * playerStore.duration

    if (isFinite(seekTime)) {
      if (useWebAudio.value) {
        const success = await webAudioPlayer.seek(seekTime)
        if (success)
          playerStore.setCurrentTime(seekTime, true)
      } else {
        if (!activePlayer.value) return
        activePlayer.value.currentTime = seekTime
        playerStore.setCurrentTime(seekTime, true)
      }
    }
  }

  const onVolumeInput = (value: number[] | undefined): void =>
    void(value?.length && playerStore.setVolume(value[0] / 100))

  const previousSong = (): void => {
    if (hasPrevious.value)
      playerStore.previousSong()
  }

  watch(() => playerStore.currentSong?.id, (newSongId, oldSongId) => {
    if (newSongId !== oldSongId) {
      const newSong = playerStore.currentSong!
      const newIndex = playerStore.playlist.findIndex(s => s.id === newSongId)
      if (newIndex !== -1)
        playerStore.setCurrentIndex(newIndex)

      if (isGaplessTransition.value) {
        isGaplessTransition.value = false
        if (useWebAudio.value) {
          playerStore.setBuffering(true)
          webAudioPlayer.play()
            .then(success => {
              if (success) {
                playerStore.play()
                startWebAudioTimeUpdates()
              } else {
                logger.error('Failed to play WebAudio in gapless transition')
                playerStore.pause()
              }
            })
            .catch(error => {
              logger.error('Failed to play WebAudio in gapless transition:', error)
              playerStore.pause()
            })
            .finally(() => {
              playerStore.setBuffering(false)
            })
        } else if (activePlayer.value) {
          playerStore.setBuffering(true)
          activePlayer.value.play()
            .then(() => {
            })
            .catch(error => {
              logger.error('Failed to play audio:', error)
              playerStore.pause()
            })
            .finally(() => {
              playerStore.setBuffering(false)
            })
        } else {
          loadSong(nextSongInQueue.value, nextPlayer.value)
        }
      } else {
        playManuallyChangedSong(newSong)
      }
    } else if (!playerStore.currentSong) {
      stopWebAudioTimeUpdates()
      webAudioPlayer.stop()

      if (audioPlayer1.value) {
        audioPlayer1.value.src = ''; audioPlayer1.value.pause()
      }
      if (audioPlayer2.value) {
        audioPlayer2.value.src = ''; audioPlayer2.value.pause()
      }
      playerStore.pause()
    }
  })

  watch(
    () => playerStore.playlist,
    newPlaylist => {
      if (playerStore.currentSong) {
        const index = newPlaylist.findIndex(song => song.id === playerStore.currentSong!.id)
        playerStore.setCurrentIndex(index)
        if (nextSongInQueue.value)
          loadSong(nextSongInQueue.value, nextPlayer.value)
      } else {
        playerStore.setCurrentIndex(-1)
      }
    },
  )

  onMounted(async () => {
    await initializePlayer()

    if (playerStore.currentSong) {
      const index = playerStore.playlist.findIndex(song => song.id === playerStore.currentSong!.id)
      playerStore.setCurrentIndex(index)
      playManuallyChangedSong(playerStore.currentSong)
    }

    measureMarquee()
    updateVisibleIcons()
    window.addEventListener('resize', measureMarquee)
    window.addEventListener('resize', updateVisibleIcons)

    // Setup responsive layout
    updateVisibleIcons()
    resizeObserver.value = new ResizeObserver(updateVisibleIcons)
    if (containerRef.value) {
      resizeObserver.value.observe(containerRef.value)
    }
  })

  watch(() => playerStore.volume, newVolume => {
    if (useWebAudio.value) {
      webAudioPlayer.setVolume(newVolume)
    } else {
      if (audioPlayer1.value) audioPlayer1.value.volume = newVolume
      if (audioPlayer2.value) audioPlayer2.value.volume = newVolume
    }
  })

  // Watch active view changes to update visible icons
  watch(activeView, () => {
    updateVisibleIcons()
  })

  onUnmounted(() => {
    stopWebAudioTimeUpdates()

    webAudioPlayer.cleanup()

    if (audioPlayer1.value) audioPlayer1.value.pause()
    if (audioPlayer2.value) audioPlayer2.value.pause()

    if (typeof window !== 'undefined') {
      const w = window as typeof window & { advanceToNextSong?: () => void }
      delete w.advanceToNextSong
    }

    window.removeEventListener('resize', measureMarquee)
    window.removeEventListener('resize', updateVisibleIcons)
    if (marqueePauseTimeoutId) clearTimeout(marqueePauseTimeoutId)

    // Cleanup resize observer
    if (resizeObserver.value) {
      resizeObserver.value.disconnect()
    }
  })

  watch(() => playerStore.currentSong?.name, async () => {
    await nextTick()
    measureMarquee()
  })

  defineExpose({
    nextSong,
    onSeek,
    previousSong,
    toggleMute:    playerStore.toggleMute,
    togglePlayPause,
    toggleRepeat:  playerStore.cycleRepeatMode,
    toggleShuffle: playerStore.toggleShuffle,
  })
</script>

<template>
  <div
    @touchend='handleSwipeEnd'
    @touchmove='handleSwipeMove'
    @touchstart='startTracking'
    v-if='playerStore.currentSong'
    class='px-2 py-3 relative'
  >
    <!-- Swipe hint overlay -->
    <div
      v-if='false'
      :style='{ opacity: swipeOpacity }'
      class='fixed inset-0 bg-black/20 backdrop-blur-sm z-40 pointer-events-none'
    >
      <div class='flex items-center justify-center h-full'>
        <div
          :style='{ transform: swipeTransform }'
          class='bg-background/90 backdrop-blur-md border border-border rounded-xl p-6 shadow-2xl'
        >
          <div class='text-center'>
            <div class='w-16 h-16 mx-auto mb-4 bg-primary/20 rounded-full flex items-center justify-center'>
              <Expand class='w-8 h-8 text-primary' />
            </div>

            <p class='text-sm font-medium'>
              Swipe up to open fullscreen
            </p>
          </div>
        </div>
      </div>
    </div>
    <!-- Audio Visualizer Background -->
    <div
      v-if='playerStore.visualizerEnabled'
      class='absolute inset-0 overflow-hidden opacity-30 pointer-events-none'
      style='z-index: 0;'
    >
      <AudioVisualizer
        v-if='useWebAudio && playerStore.isPlaying'
        :analyser-node='webAudioPlayer.getAnalyserNode()'
        :is-playing='playerStore.isPlaying'
        :style='playerStore.visualizerStyle'
      />
    </div>

    <div ref='containerRef' class='mx-auto max-w-full relative' style='z-index: 1;'>
      <div :class="isMobilePortraitMode ? 'flex items-center px-2' : 'grid grid-cols-3 items-center px-2'">
        <div
          @mouseenter='onTitleMouseEnter'
          @mouseleave='onTitleMouseLeave'
          :class="['flex items-center space-x-4 min-w-0', shouldMarquee ? 'marquee-enabled' : '']"
        >
          <div @click="emit('toggle-fullscreen')" class='shrink-0'>
            <ImageLoader
              v-if='playerStore.currentSong'
              :item-id='playerStore.currentSong.albumId || playerStore.currentSong.id'
              :server-url='serverUrl'
              :token='token'
              alt='Album art'
              class='size-12 rounded-md cursor-pointer'
            >
              <template #fallback>
                <div
                  class='size-12 bg-muted rounded-md flex items-center justify-center cursor-pointer'
                >
                  <Music2 class='size-6 text-muted-foreground' />
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
                    :to='`/artists/${playerStore.currentSong.artistIds[index]}`'
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
                v-if='playerStore.currentSong.album && playerStore.currentSong.albumId'
                :to='`/albums/${playerStore.currentSong.albumId}`'
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

        <div v-if='!isMobilePortraitMode' class='grow px-4'>
          <div class='flex justify-center'>
            <div class='flex items-center space-x-2'>
              <button
                @click='playerStore.toggleShuffle'
                :class="[
                  'inline-flex items-center justify-center rounded-button text-sm font-medium',
                  'transition-colors h-8 w-8',
                  playerStore.isShuffled
                    ? 'bg-accent text-accent-foreground hover:bg-accent/90'
                    : 'hover:bg-accent/20',
                ]"
              >
                <Shuffle
                  :class="['size-4', playerStore.isShuffled ? '' : 'text-muted-foreground']"
                />
              </button>
              <Button
                @click='previousSong'
                :disabled='!hasPrevious'
                size='icon'
                variant='ghost'
              >
                <SkipBack class='size-4' />
              </Button>

              <Button
                @click='togglePlayPause'
                :disabled='!playerStore.audioReady || playerStore.isBuffering'
                class='rounded-full! size-10'
                size='icon'
                variant='default'
              >
                <Loader2 v-if='playerStore.isBuffering' class='size-4 animate-spin' />
                <Play v-else-if='!playerStore.isPlaying' class='size-4' />
                <Pause v-else class='size-4' />
              </Button>

              <Button
                @click='nextSong'
                :disabled='!hasNext'
                size='icon'
                variant='ghost'
              >
                <SkipForward class='size-4' />
              </Button>
              <button
                @click='playerStore.cycleRepeatMode'
                :class="[
                  'inline-flex items-center justify-center rounded-button text-sm font-medium',
                  'transition-colors h-8 w-8',
                  playerStore.repeatMode !== 'none'
                    ? 'bg-accent text-accent-foreground hover:bg-accent/90'
                    : 'hover:bg-accent/20',
                ]"
              >
                <Repeat1
                  v-if="playerStore.repeatMode === 'one'"
                  class='size-4'
                />
                <Repeat
                  v-else
                  :class="[
                    'size-4',
                    playerStore.repeatMode === 'none' ? 'text-muted-foreground' : '',
                  ]"
                />
              </button>
            </div>
          </div>
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

        <div :class="isMobilePortraitMode ? 'flex justify-end flex-1' : 'flex justify-end'">
          <!-- Mobile portrait: only play button -->
          <Button
            @click='togglePlayPause'
            v-if='isMobilePortraitMode'
            :disabled='!playerStore.audioReady || playerStore.isBuffering'
            class='rounded-full! size-12'
            size='icon'
            variant='default'
          >
            <Loader2 v-if='playerStore.isBuffering' class='size-5 animate-spin' />
            <Play v-else-if='!playerStore.isPlaying' class='size-5' />
            <Pause v-else class='size-5' />
          </Button>

          <!-- Desktop: all buttons -->
          <div v-else class='flex items-center space-x-2'>
            <!-- Queue button -->
            <Button
              @click="emit('toggle-queue')"
              v-if="visibleIcons.includes('queue')"
              :variant="activeView === 'queue' ? 'default' : 'ghost'"
              size='icon'
            >
              <ListMusic class='size-4' />
            </Button>

            <!-- Lyrics button -->
            <Button
              @click="emit('toggle-lyrics')"
              v-if="visibleIcons.includes('lyrics')"
              :disabled='!hasLyrics'
              :variant="activeView === 'lyrics' ? 'default' : 'ghost'"
              size='icon'
            >
              <Mic2 class='size-4' />
            </Button>

            <!-- Favorite button -->
            <Button
              @click="emit('toggle-favorite', playerStore.currentSong)"
              v-if="visibleIcons.includes('favorite')"
              size='icon'
              variant='ghost'
            >
              <Heart
                :class="[
                  'size-4',
                  playerStore.currentSong.isFavorite
                    ? 'fill-current'
                    : '',
                ]"
              />
            </Button>

            <!-- Fullscreen button -->
            <Button
              @click="emit('toggle-fullscreen')"
              v-if="visibleIcons.includes('fullscreen')"
              size='icon'
              variant='ghost'
            >
              <Expand class='size-4' />
            </Button>

            <!-- Equalizer button -->
            <Button
              @click="emit('toggle-equalizer')"
              v-if="visibleIcons.includes('equalizer') && useWebAudio"
              :variant="activeView === 'equalizer' ? 'default' : 'ghost'"
              size='icon'
            >
              <Sliders class='size-4' />
            </Button>

            <!-- Volume button -->
            <div v-if="visibleIcons.includes('volume')" class='relative'>
              <Button
                @click='handleVolumeClick'
                :class='isVolumePopupVisible ? "bg-accent/20" : ""'
                size='icon'
                variant='ghost'
                data-volume-button
              >
                <Volume2 v-if='playerStore.volume > 50' class='size-4' />
                <Volume1 v-else-if='playerStore.volume > 0' class='size-4' />
                <VolumeX v-else class='size-4' />
              </Button>
              <div
                v-if='isVolumePopupVisible'
                ref='volumePopupRef'
                class='absolute bottom-full left-1/2 transform -translate-x-1/2 mb-1 p-3
                       bg-card border border-border rounded-md shadow-lg z-50'
              >
                <div class='flex flex-col items-center gap-2'>
                  <span class='text-xs text-muted-foreground font-medium'>
                    {{ Math.round(playerStore.volume * 100) }}%
                  </span>
                  <Slider
                    @update:model-value='onVolumeInput'
                    :max='100'
                    :model-value='[playerStore.volume * 100]'
                    :step='1'
                    class='h-16 w-1.5'
                    orientation='vertical'
                  />
                  <button
                    @click.stop='playerStore.toggleMute'
                    class='text-muted-foreground hover:text-foreground transition-colors p-1 rounded'
                  >
                    <Volume2 v-if='playerStore.volume > 50' class='size-4' />
                    <Volume1 v-else-if='playerStore.volume > 0' class='size-4' />
                    <VolumeX v-else class='size-4' />
                  </button>
                </div>
              </div>
            </div>

            <!-- Three-dot menu for hidden icons -->
            <DropdownMenu v-if='hasHiddenIcons'>
              <DropdownMenuTrigger as-child>
                <Button size='icon' variant='ghost'>
                  <MoreHorizontal class='size-4' />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align='end'>
                <DropdownMenuItem v-if="!visibleIcons.includes('volume')">
                  <div class='flex items-center space-x-2 w-full'>
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
                      class='w-24 flex-1'
                    />
                  </div>
                </DropdownMenuItem>
                <DropdownMenuItem
                  @click="emit('toggle-equalizer')"
                  v-if="!visibleIcons.includes('equalizer') && useWebAudio"
                >
                  <Sliders class='size-4 mr-2' />
                  Equalizer
                </DropdownMenuItem>
                <DropdownMenuItem
                  @click="emit('toggle-fullscreen')"
                  v-if="!visibleIcons.includes('fullscreen')"
                >
                  <Expand class='size-4 mr-2' />
                  Fullscreen
                </DropdownMenuItem>
                <DropdownMenuItem
                  @click="emit('toggle-favorite', playerStore.currentSong)"
                  v-if="!visibleIcons.includes('favorite')"
                >
                  <Heart
                    :class="[
                      'size-4 mr-2',
                      playerStore.currentSong.isFavorite
                        ? 'fill-current'
                        : '',
                    ]"
                  />
                  Favorite
                </DropdownMenuItem>
                <DropdownMenuItem
                  @click="emit('toggle-lyrics')"
                  v-if="!visibleIcons.includes('lyrics')"
                  :disabled='!hasLyrics'
                >
                  <Mic2 class='size-4 mr-2' />
                  Lyrics
                </DropdownMenuItem>
                <DropdownMenuItem
                  @click="emit('toggle-queue')"
                  v-if="!visibleIcons.includes('queue')"
                >
                  <ListMusic class='size-4 mr-2' />
                  Queue
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
        </div>
      </div>
    </div>

    <audio
      @canplaythrough='onCanPlay(0)'
      @ended='onEnded(0)'
      @error='onError(0)'
      @loadedmetadata='onLoadedMetadata(0)'
      @pause='onPause(0)'
      @play='onPlay(0)'
      @timeupdate='onTimeUpdate(0)'
      id='audio-player-1'
      ref='audioPlayer1'
      crossorigin='anonymous'
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
      id='audio-player-2'
      ref='audioPlayer2'
      crossorigin='anonymous'
      preload='auto'
    />
  </div>
</template>
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
