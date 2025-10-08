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
  import ImageLoader from '@/components/shared/ImageLoader.vue'
  import { Button } from '@/components/ui/button'
  import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
  } from '@/components/ui/dropdown-menu'
  import { Slider } from '@/components/ui/slider'
  import { useWebAudioPlayer } from '@/composables/useWebAudioPlayer'
  import { playerLogger } from '@/lib/logger'
  import { getSongFormatInfo } from '@/lib/utils'
  import { usePlayerStore } from '@/stores'

  const props = defineProps<{
    isEqualizerOpen?: boolean
    isLyricsOpen?:    boolean
    isQueueOpen?:     boolean
    serverUrl:        string
    token:            string
  }>()

  defineEmits<{
    'toggle-equalizer':  []
    'toggle-favorite':   [song: Song]
    'toggle-fullscreen': []
    'toggle-lyrics':     []
    'toggle-queue':      []
  }>()

  const playerStore = usePlayerStore()
  const { getAudioStreamUrl } = commands

  const webAudioPlayer = useWebAudioPlayer()

  webAudioPlayer.setOnDurationChange((duration: number) => {
    if (
      isFinite(duration)
      && duration > 0
      && duration !== Infinity
      && duration !== playerStore.duration
    )
      playerStore.setDuration(duration)
  })

  const audioPlayer1 = ref<HTMLAudioElement | null>(null)
  const audioPlayer2 = ref<HTMLAudioElement | null>(null)
  const containerRef = ref<HTMLDivElement | null>(null)
  const resizeObserver = ref<null | ResizeObserver>(null)
  const volumePopupRef = ref<HTMLDivElement | null>(null)
  const isVolumePopupVisible = ref(false)
  const activePlayerIndex = ref(0)
  const players = [audioPlayer1, audioPlayer2]
  const activePlayer = computed(() => players[activePlayerIndex.value].value)
  const nextPlayer = computed(() => players[1 - activePlayerIndex.value].value)

  const playerType = ref<'html5' | 'webaudio'>('html5')
  const useWebAudio = computed(() => playerType.value === 'webaudio')

  const nextSongReady = ref(false)
  const isGaplessTransition = ref(false)
  const webAudioTimeUpdateInterval = ref<null | number>(null)

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

  const nextSongInQueue = computed(() => {
    if (!hasNext.value) return null

    let nextIndex
    if (playerStore.isShuffled)
      nextIndex = Math.floor(Math.random() * playerStore.playlist.length)
    else
      nextIndex = playerStore.currentIndex + 1

    return playerStore.playlist[nextIndex]
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

  const toggleVolumePopup = (): void => {
    isVolumePopupVisible.value = !isVolumePopupVisible.value
  }

  const closeVolumePopup = (): void => {
    isVolumePopupVisible.value = false
  }

  const handleVolumeClick = (): void => {
    toggleVolumePopup()
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

  const startWebAudioTimeUpdates = (): void => {
    if (webAudioTimeUpdateInterval.value) {
      clearInterval(webAudioTimeUpdateInterval.value)
    }
    webAudioTimeUpdateInterval.value = window.setInterval(() => {
      if (useWebAudio.value && webAudioPlayer.getIsPlaying()) {
        const currentTime = webAudioPlayer.getCurrentTime()
        playerStore.setCurrentTime(currentTime)
      }
    }, 100)
  }

  const stopWebAudioTimeUpdates = (): void => {
    if (webAudioTimeUpdateInterval.value) {
      clearInterval(webAudioTimeUpdateInterval.value)
      webAudioTimeUpdateInterval.value = null
    }
  }

  const formatTime = (seconds: number): string =>
    `${Math.floor(seconds / 60)}:${(Math.floor(seconds % 60)).toString().padStart(2, '0')}`

  const onLoadedMetadata = (playerIndex: number): void => {
    if (players[playerIndex].value && playerIndex === activePlayerIndex.value) {
      if (playerStore.currentSong?.duration)
        playerStore.setDuration(playerStore.currentSong.duration)
      else
        playerStore.setDuration(players[playerIndex].value.duration || 0)

      playerStore.setCurrentTime(0)
    }
  }

  const onTimeUpdate = (playerIndex: number): void => {
    if (playerIndex === activePlayerIndex.value && players[playerIndex].value)
      playerStore.setCurrentTime(players[playerIndex].value.currentTime)
  }

  const onCanPlay = (playerIndex: number): void => {
    if (playerIndex === activePlayerIndex.value) {
      playerStore.setAudioReady(true)
    } else {
      nextSongReady.value = true
      playerLogger.debug(`Next song ready (player ${playerIndex})`)
    }
  }

  const onError = (playerIndex: number): void => {
    const player = players[playerIndex].value
    playerLogger.error(`Audio playback error on player ${playerIndex}:`, player?.error)

    if (playerIndex === activePlayerIndex.value) {
      playerStore.setAudioReady(false)
      playerStore.setBuffering(false)
    }
  }

  const onPlay = (playerIndex: number): void     => {
    if (playerIndex === activePlayerIndex.value)
      playerStore.play()
  }

  const onPause = (playerIndex: number): void => {
    if (playerIndex === activePlayerIndex.value)
      playerStore.pause()
  }

  const onEnded = async (playerIndex: number): Promise<void> => {
    if (playerIndex !== activePlayerIndex.value) return

    playerLogger.debug(`Track ended - next ready: ${nextSongReady.value}`)

    if (playerStore.repeatMode === 'one') {
      if (activePlayer.value) {
        activePlayer.value.currentTime = 0
        activePlayer.value.play()
      }
    } else if (nextSongReady.value && nextSongInQueue.value) {
      playerLogger.debug('Using gapless playback')
      await fallbackToGapless()
    } else if (playerStore.repeatMode === 'all' || hasNext.value) {
      nextSong()
    } else {
      activePlayer.value?.pause()
    }
  }

  const fallbackToGapless = async (): Promise<void> => {
    playerLogger.debug('Performing gapless fallback')

    const nextPlayerElement = nextPlayer.value
    if (nextPlayerElement && nextPlayerElement.paused && nextSongReady.value) {
      try {
        nextPlayerElement.currentTime = 0
        await nextPlayerElement.play()
        playerLogger.debug('Next player started for gapless transition')
      } catch (error) {
        playerLogger.error('Failed to start next player in gapless fallback:', error)
      }
    }

    activePlayer.value?.pause()

    isGaplessTransition.value = true
    activePlayerIndex.value = 1 - activePlayerIndex.value

    playerStore.setCurrentTime(0)
    playerStore.setDuration(nextSongInQueue.value?.duration || 0)
    playerStore.setCurrentSong(nextSongInQueue.value)

    playerLogger.debug('Gapless fallback complete')
  }

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
      playerLogger.error('Playback error:', error)
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
          playerStore.setCurrentTime(seekTime)
      } else {
        if (!activePlayer.value) return
        activePlayer.value.currentTime = seekTime
        playerStore.setCurrentTime(seekTime)
      }
    }
  }

  const onVolumeInput = (value: number[] | undefined): void =>
    void(value?.length && playerStore.setVolume(value[0] / 100))

  const previousSong = (): void => {
    if (hasPrevious.value)
      playerStore.previousSong()
  }

  const nextSong = (): void =>
    hasNext.value
      ? playerStore.nextSong()
      : playerStore.repeatMode === 'all'
        ? playSongAtIndex(0)
        : void(0)

  const playSongAtIndex = (index: number): void => {
    if (index < 0 || index >= playerStore.playlist.length) return

    playerStore.playSongAtIndex(index)

    loadSong(playerStore.playlist[index], activePlayer.value)
  }

  const loadSong = async (song: null | Song, player: HTMLAudioElement | null): Promise<void> => {
    if (!song) {
      if (useWebAudio.value) {
        webAudioPlayer.stop()
        playerStore.setAudioReady(false)
      } else if (player && player.src && player.src !== '') {
        player.src = ''
      }

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

      if (useWebAudio.value) {
        playerStore.setAudioReady(false)
        playerStore.setBuffering(true)

        const initialUrl = streamResult.data
        let loaded = await webAudioPlayer.loadAudio(initialUrl)

        if (!loaded && initialUrl.includes('/stream?')) {
          playerLogger.warn('Initial stream failed, attempting fallback with .aac container')
          try {
            const fallbackUrl = new URL(initialUrl)
            fallbackUrl.pathname = fallbackUrl.pathname.replace('/stream', '/stream.aac')
            fallbackUrl.searchParams.delete('static') // This param seems to be part of the redirect URL
            const finalUrl = fallbackUrl.toString()
            playerLogger.debug(`Fallback URL: ${finalUrl}`)
            loaded = await webAudioPlayer.loadAudio(finalUrl)
          } catch (e) {
            playerLogger.error('Failed to construct fallback URL:', e)
          }
        }

        if (loaded) {
          playerStore.setAudioReady(true)
          playerStore.setBuffering(false)
          const webAudioDuration = webAudioPlayer.getDuration()
          // For streaming, WebAudio initially reports Infinity, so keep the song duration
          // Only update if we get a finite duration
          if (isFinite(webAudioDuration) && webAudioDuration > 0) {
            playerStore.setDuration(webAudioDuration)
          }
          playerStore.setCurrentTime(0)
        } else {
          throw new Error('Failed to load audio via WebAudio API')
        }
      } else {
        if (!player) return

        player.src = streamResult.data
        player.load()

        if (player === activePlayer.value) {
          playerStore.setAudioReady(false)
          playerStore.setBuffering(true)
        }
      }
    } catch (error) {
      playerLogger.error(`Failed to load audio for song ${song.name} (ID: ${song.id}):`, error)
      playerStore.setAudioReady(false)
      playerStore.setBuffering(false)
    }
  }

  const playManuallyChangedSong = (song: Song): void => {
    playerStore.setAudioReady(false)
    nextSongReady.value = false
    playerStore.setBuffering(true)

    const execute = async (): Promise<void> => {
      await loadSong(song, activePlayer.value)

      if (useWebAudio.value) {
        const success = await webAudioPlayer.play()
        if (success) {
          playerStore.play()
          startWebAudioTimeUpdates()
        } else {
          playerLogger.error('Failed to start WebAudio playback')
          playerStore.pause()
        }
        playerStore.setBuffering(false)
      } else if (activePlayer.value) {
        try {
          await activePlayer.value.play()
        } catch (error) {
          playerLogger.error('Failed to play audio:', error)
          playerStore.pause()
        } finally {
          playerStore.setBuffering(false)
        }
      } else {
        playerStore.setBuffering(false)
      }

      if (nextSongInQueue.value && !useWebAudio.value) {
        playerLogger.debug(`Loading next song: ${nextSongInQueue.value.name}`)
        await loadSong(nextSongInQueue.value, nextPlayer.value)
      }
    }
    execute()
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
                playerLogger.error('Failed to play WebAudio in gapless transition')
                playerStore.pause()
              }
            })
            .catch(error => {
              playerLogger.error('Failed to play WebAudio in gapless transition:', error)
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
              playerLogger.error('Failed to play audio:', error)
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

  const advanceToNextSong = (): void => {
    playerLogger.debug('WebAudio track ended, advancing to next song')

    if (playerStore.repeatMode === 'one') {
      playerStore.setCurrentTime(0)
      if (useWebAudio.value) {
        webAudioPlayer.seek(0)
        webAudioPlayer.play()
          .then(success => {
            if (success) {
              playerStore.play()
              startWebAudioTimeUpdates()
            }
          })
      }
    } else if (playerStore.repeatMode === 'all' || hasNext.value) {
      nextSong()
    } else {
      playerStore.pause()
    }
  }

  if (typeof window !== 'undefined') {
    const w = window as typeof window & { advanceToNextSong?: () => void }
    w.advanceToNextSong = advanceToNextSong
  }

  const initializePlayer = async (): Promise<void> => {
    const webAudioAvailable = webAudioPlayer.isWebAudioAvailable()

    if (webAudioAvailable) {
      const initialized = await webAudioPlayer.initializeWebAudio()
      if (initialized) {
        playerType.value = 'webaudio'
        playerLogger.info('Using WebAudio API with streaming support')
      } else {
        playerType.value = 'html5'
        playerLogger.warn('WebAudio API available but failed to initialize, falling back to HTML5')
      }
    } else {
      playerType.value = 'html5'
      playerLogger.info('WebAudio API not available, using HTML5 Audio Player')
    }

    if (useWebAudio.value) {
      webAudioPlayer.setVolume(playerStore.volume)
    } else {
      if (audioPlayer1.value) audioPlayer1.value.volume = playerStore.volume
      if (audioPlayer2.value) audioPlayer2.value.volume = playerStore.volume
    }
  }

  onMounted(async () => {
    const webAudioAvailable = webAudioPlayer.isWebAudioAvailable()
    playerLogger.info(`Audio APIs available - WebAudio: ${webAudioAvailable}`)
    playerLogger.info(`HTML5 Audio available: ${typeof Audio !== 'undefined'}`)

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
  <div v-if='playerStore.currentSong' class='px-2 py-3'>
    <div ref='containerRef' class='mx-auto max-w-full'>
      <div class='grid grid-cols-3 items-center px-2'>
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

        <div class='flex-grow px-4'>
          <div class='flex justify-center'>
            <div class='flex items-center space-x-2'>
              <button
                @click='playerStore.toggleShuffle'
                :class="[
                  'inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors h-8 w-8',
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
                class='rounded-full size-10'
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
                  'inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors h-8 w-8',
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

        <div class='flex justify-end'>
          <div class='flex items-center space-x-2'>
            <!-- Queue button -->
            <Button
              @click="$emit('toggle-queue')"
              v-if="visibleIcons.includes('queue')"
              :variant="activeView === 'queue' ? 'default' : 'ghost'"
              size='icon'
            >
              <ListMusic class='size-4' />
            </Button>

            <!-- Lyrics button -->
            <Button
              @click="$emit('toggle-lyrics')"
              v-if="visibleIcons.includes('lyrics')"
              :disabled='!hasLyrics'
              :variant="activeView === 'lyrics' ? 'default' : 'ghost'"
              size='icon'
            >
              <Mic2 class='size-4' />
            </Button>

            <!-- Favorite button -->
            <Button
              @click="$emit('toggle-favorite', playerStore.currentSong)"
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
              @click="$emit('toggle-fullscreen')"
              v-if="visibleIcons.includes('fullscreen')"
              size='icon'
              variant='ghost'
            >
              <Expand class='size-4' />
            </Button>

            <!-- Equalizer button -->
            <Button
              @click="$emit('toggle-equalizer')"
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
                  @click="$emit('toggle-equalizer')"
                  v-if="!visibleIcons.includes('equalizer') && useWebAudio"
                >
                  <Sliders class='size-4 mr-2' />
                  Equalizer
                </DropdownMenuItem>
                <DropdownMenuItem
                  @click="$emit('toggle-fullscreen')"
                  v-if="!visibleIcons.includes('fullscreen')"
                >
                  <Expand class='size-4 mr-2' />
                  Fullscreen
                </DropdownMenuItem>
                <DropdownMenuItem
                  @click="$emit('toggle-favorite', playerStore.currentSong)"
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
                  @click="$emit('toggle-lyrics')"
                  v-if="!visibleIcons.includes('lyrics')"
                  :disabled='!hasLyrics'
                >
                  <Mic2 class='size-4 mr-2' />
                  Lyrics
                </DropdownMenuItem>
                <DropdownMenuItem
                  @click="$emit('toggle-queue')"
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
