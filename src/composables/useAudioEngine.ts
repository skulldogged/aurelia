import { computed, type ComputedRef, ref, type Ref } from 'vue'

import { commands } from '@/bindings'
import { Song } from '@/bindings'
import { useWebAudioPlayer } from '@/composables/useWebAudioPlayer'
import { logger } from '@/lib/logger'
import { usePlayerStore } from '@/stores'

interface UseAudioEngineReturn {
  activePlayer:             ComputedRef<HTMLAudioElement | null>;
  audioPlayer1:             Ref<HTMLAudioElement | null>;
  audioPlayer2:             Ref<HTMLAudioElement | null>;
  initializePlayer:         () => Promise<void>;
  isGaplessTransition:      Ref<boolean>;
  loadSong:                 (song: null | Song, player: HTMLAudioElement | null) => Promise<void>;
  nextPlayer:               ComputedRef<HTMLAudioElement | null>;
  nextSong:                 () => void;
  nextSongInQueue:          ComputedRef<null | Song>;
  onCanPlay:                (playerIndex: number) => void;
  onEnded:                  (playerIndex: number) => Promise<void>;
  onError:                  (playerIndex: number) => void;
  onLoadedMetadata:         (playerIndex: number) => void;
  onPause:                  (playerIndex: number) => void;
  onPlay:                   (playerIndex: number) => void;
  onTimeUpdate:             (playerIndex: number) => void;
  playManuallyChangedSong:  (song: Song) => void;
  playSongAtIndex:          (index: number) => void;
  startWebAudioTimeUpdates: () => void;
  stopWebAudioTimeUpdates:  () => void;
  useWebAudio:              ComputedRef<boolean>;
  webAudioPlayer:           ReturnType<typeof useWebAudioPlayer>;
}

export const useAudioEngine = (
  props: { serverUrl: string; token: string },
): UseAudioEngineReturn => {
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
  const activePlayerIndex = ref(0)
  const players = [audioPlayer1, audioPlayer2]
  const activePlayer = computed(() => players[activePlayerIndex.value].value)
  const nextPlayer = computed(() => players[1 - activePlayerIndex.value].value)

  const playerType = ref<'html5' | 'webaudio'>('html5')
  const useWebAudio = computed(() => playerType.value === 'webaudio')

  const nextSongReady = ref(false)
  const isGaplessTransition = ref(false)
  const webAudioTimeUpdateInterval = ref<null | number>(null)

  const hasNext = computed(() =>
    playerStore.playlist.length > 1
    && playerStore.currentIndex > -1
    && playerStore.currentIndex < playerStore.playlist.length - 1,
  )

  const nextSongInQueue = computed(() => {
    if (!hasNext.value) return null

    let nextIndex
    if (playerStore.isShuffled)
      nextIndex = Math.floor(Math.random() * playerStore.playlist.length)
    else
      nextIndex = playerStore.currentIndex + 1

    return playerStore.playlist[nextIndex]
  })

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

  const onLoadedMetadata = (playerIndex: number): void => {
    if (players[playerIndex].value && playerIndex === activePlayerIndex.value) {
      if (playerStore.currentSong?.duration)
        playerStore.setDuration(playerStore.currentSong.duration)
      else
        playerStore.setDuration(players[playerIndex].value!.duration || 0)

      playerStore.setCurrentTime(0)
    }
  }

  const onTimeUpdate = (playerIndex: number): void => {
    if (playerIndex === activePlayerIndex.value && players[playerIndex].value)
      playerStore.setCurrentTime(players[playerIndex].value!.currentTime)
  }

  const onCanPlay = (playerIndex: number): void => {
    if (playerIndex === activePlayerIndex.value) {
      playerStore.setAudioReady(true)
    } else {
      nextSongReady.value = true
      logger.debug(`Next song ready (player ${playerIndex})`)
    }
  }

  const onError = (playerIndex: number): void => {
    const player = players[playerIndex].value
    logger.error(`Audio playback error on player ${playerIndex}:`, player?.error)

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

    logger.debug(`Track ended - next ready: ${nextSongReady.value}`)

    if (playerStore.repeatMode === 'one') {
      if (activePlayer.value) {
        activePlayer.value.currentTime = 0
        activePlayer.value.play()
      }
    } else if (nextSongReady.value && nextSongInQueue.value) {
      logger.debug('Using gapless playback')
      await fallbackToGapless()
    } else if (playerStore.repeatMode === 'all' || hasNext.value) {
      nextSong()
    } else {
      activePlayer.value?.pause()
    }
  }

  const fallbackToGapless = async (): Promise<void> => {
    logger.debug('Performing gapless fallback')

    const nextPlayerElement = nextPlayer.value
    if (nextPlayerElement && nextPlayerElement.paused && nextSongReady.value) {
      try {
        nextPlayerElement.currentTime = 0
        await nextPlayerElement.play()
        logger.debug('Next player started for gapless transition')
      } catch (error) {
        logger.error('Failed to start next player in gapless fallback:', error)
      }
    }

    activePlayer.value?.pause()

    isGaplessTransition.value = true
    activePlayerIndex.value = 1 - activePlayerIndex.value

    playerStore.setCurrentTime(0)
    playerStore.setDuration(nextSongInQueue.value?.duration || 0)
    playerStore.setCurrentSong(nextSongInQueue.value)

    logger.debug('Gapless fallback complete')
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
        logger.error('Failed to get audio stream URL:', streamResult.error)
        throw new Error(streamResult.error)
      }

      if (useWebAudio.value) {
        playerStore.setAudioReady(false)
        playerStore.setBuffering(true)

        const initialUrl = streamResult.data
        let loaded = await webAudioPlayer.loadAudio(initialUrl)

        if (!loaded && initialUrl.includes('/stream?')) {
          logger.warn('Initial stream failed, attempting fallback with .aac container')
          try {
            const fallbackUrl = new URL(initialUrl)
            fallbackUrl.pathname = fallbackUrl.pathname.replace('/stream', '/stream.aac')
            fallbackUrl.searchParams.delete('static') // This param seems to be part of the redirect URL
            const finalUrl = fallbackUrl.toString()
            logger.debug(`Fallback URL: ${finalUrl}`)
            loaded = await webAudioPlayer.loadAudio(finalUrl)
          } catch (e) {
            logger.error('Failed to construct fallback URL:', e)
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
      logger.error(`Failed to load audio for song ${song.name} (ID: ${song.id}):`, error)
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
          logger.error('Failed to start WebAudio playback')
          playerStore.pause()
        }
        playerStore.setBuffering(false)
      } else if (activePlayer.value) {
        try {
          await activePlayer.value.play()
        } catch (error) {
          logger.error('Failed to play audio:', error)
          playerStore.pause()
        } finally {
          playerStore.setBuffering(false)
        }
      } else {
        playerStore.setBuffering(false)
      }

      if (nextSongInQueue.value && !useWebAudio.value) {
        logger.debug(`Loading next song: ${nextSongInQueue.value.name}`)
        await loadSong(nextSongInQueue.value, nextPlayer.value)
      }
    }
    execute()
  }

  const advanceToNextSong = (): void => {
    logger.debug('WebAudio track ended, advancing to next song')

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
        logger.info('Using WebAudio API with streaming support')
      } else {
        playerType.value = 'html5'
        logger.warn('WebAudio API available but failed to initialize, falling back to HTML5')
      }
    } else {
      playerType.value = 'html5'
      logger.info('WebAudio API not available, using HTML5 Audio Player')
    }

    if (useWebAudio.value) {
      webAudioPlayer.setVolume(playerStore.volume)
    } else {
      if (audioPlayer1.value) audioPlayer1.value.volume = playerStore.volume
      if (audioPlayer2.value) audioPlayer2.value.volume = playerStore.volume
    }
  }

  return {
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
    playSongAtIndex,
    startWebAudioTimeUpdates,
    stopWebAudioTimeUpdates,
    useWebAudio,
    webAudioPlayer,
  }
}
