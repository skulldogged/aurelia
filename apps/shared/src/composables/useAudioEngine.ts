/**
 * Audio Engine Composable
 *
 * Unified audio orchestration layer using the AudioPlayer abstraction.
 * Platform-specific logic is handled internally by the AudioPlayer implementations.
 */

import { computed, type ComputedRef, onUnmounted, ref, type Ref, watch } from 'vue'

import type { NowPlayingPayload, Song } from '../lib/api/types'

import { type AudioPlayer, type AudioPosition, getAudioPlayer } from '../audio'
import { getApiClient } from '../index'
import { logger } from '../lib/logger'
import { isDesktop, isTauri } from '../lib/platform'
import { usePlayerStore } from '../stores'

type UnlistenFn = () => void

interface UseAudioEngineReturn {
  audioPlayer:             AudioPlayer
  initializePlayer:        () => Promise<void>
  isGaplessTransition:     Ref<boolean>
  loadSong:                (song: null | Song) => Promise<void>
  nextSong:                () => void
  nextSongInQueue:         ComputedRef<null | Song>
  playManuallyChangedSong: (song: Song) => void
  playSongAtIndex:         (index: number) => void
  resetEQ:                 () => Promise<void>
  resumeContext:           () => Promise<void>
  seek:                    (positionSecs: number) => Promise<void>
  setEQBand:               (band: number, gain: number) => Promise<void>
  setEQEnabled:            (enabled: boolean) => Promise<void>
}

export const useAudioEngine = (
  props: { serverUrl: string; token: string },
): UseAudioEngineReturn => {
  const playerStore = usePlayerStore()
  const audioPlayer = getAudioPlayer()

  // State
  const isGaplessTransition = ref(false)
  const mediaEventUnlisteners = ref<UnlistenFn[]>([])

  const hasNext = computed(() =>
    playerStore.playlist.length > 1
    && playerStore.currentIndex > -1
    && playerStore.currentIndex < playerStore.playlist.length - 1,
  )

  const hasPrevious = computed(() =>
    playerStore.playlist.length > 1 && playerStore.currentIndex > 0,
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

  // Setup media control event listeners (desktop only)
  const setupMediaEventListeners = async (): Promise<void> => {
    if (!isTauri()) return

    const { listen } = await import('@tauri-apps/api/event')

    // Clean up existing listeners
    mediaEventUnlisteners.value.forEach(unlisten => unlisten())
    mediaEventUnlisteners.value = []

    mediaEventUnlisteners.value.push(
      await listen('media:play', () => {
        logger.debug('Media key: Play')
        playerStore.play()
      }),
      await listen('media:pause', () => {
        logger.debug('Media key: Pause')
        playerStore.pause()
      }),
      await listen('media:next', () => {
        logger.debug('Media key: Next')
        nextSong()
      }),
      await listen('media:previous', () => {
        logger.debug('Media key: Previous')
        if (playerStore.currentIndex > 0) {
          playerStore.previousSong()
        }
      }),
      await listen('media:stop', () => {
        logger.debug('Media key: Stop')
        playerStore.pause()
      }),
    )

    logger.debug('Media control event listeners registered')
  }

  // Update OS Now Playing with current song metadata
  const updateNowPlaying = async (song: Song): Promise<void> => {
    if (!isTauri()) return

    try {
      const payload: NowPlayingPayload = {
        album:        song.album ?? null,
        artist:       song.artists?.join(', ') ?? null,
        coverUrl:     song.albumArtUrl ?? null,
        durationSecs: song.duration ?? null,
        title:        song.name,
      }
      await getApiClient().mediaUpdateNowPlaying(payload)
      logger.debug(`Updated OS Now Playing: ${song.name}`)
    } catch (error) {
      logger.error('Failed to update Now Playing:', error)
    }
  }

  // Update OS media control button states based on queue position
  const updateMediaButtonStates = async (): Promise<void> => {
    if (!isTauri()) return

    try {
      const canGoNext = hasNext.value || playerStore.repeatMode === 'all'
      const canGoPrevious = hasPrevious.value

      await Promise.all([
        getApiClient().mediaSetButtonEnabled('next', canGoNext),
        getApiClient().mediaSetButtonEnabled('previous', canGoPrevious),
      ])
      logger.debug(`Updated media buttons: next=${canGoNext}, previous=${canGoPrevious}`)
    } catch (error) {
      logger.error('Failed to update media button states:', error)
    }
  }

  // Cleanup on unmount
  onUnmounted(() => {
    audioPlayer.destroy()
    mediaEventUnlisteners.value.forEach(unlisten => unlisten())
    mediaEventUnlisteners.value = []
  })

  const handleTrackEnded = async (): Promise<void> => {
    logger.debug('Track ended')

    if (playerStore.repeatMode === 'one') {
      const song = playerStore.currentSong
      if (song) {
        await loadSong(song)
        playerStore.play()
      }
    } else if (playerStore.repeatMode === 'all' || hasNext.value) {
      const upcomingSong = nextSongInQueue.value
      logger.debug(`[Gapless] upcomingSong: ${upcomingSong?.name}, currentIndex: ${playerStore.currentIndex}`)

      if (upcomingSong) {
        if (isDesktop()) {
          isGaplessTransition.value = true
          logger.debug('[Gapless] Set isGaplessTransition = true, calling advanceGapless')
          const success = await audioPlayer.advanceGapless()
          logger.debug(`[Gapless] advanceGapless returned: ${success}`)

          if (success) {
            const newIndex = playerStore.playlist.findIndex(s => s.id === upcomingSong.id)
            playerStore.setCurrentSong(upcomingSong)
            if (newIndex !== -1) {
              playerStore.setCurrentIndex(newIndex)
            }

            playerStore.setCurrentTime(0)
            playerStore.setDuration(upcomingSong.duration || 0)
            playerStore.play()

            await prepareNextTrack()
          } else {
            logger.debug('[Gapless] advanceGapless failed, falling back to nextSong')
            isGaplessTransition.value = false
            nextSong()
          }
          isGaplessTransition.value = false
        } else {
          // Web doesn't support gapless
          nextSong()
        }
      } else if (playerStore.repeatMode === 'all') {
        playSongAtIndex(0)
      }
    } else {
      playerStore.pause()
    }
  }

  const prepareNextTrack = async (): Promise<void> => {
    if (!isDesktop()) return

    const next = nextSongInQueue.value
    logger.debug(`[PrepareNext] nextSongInQueue: ${next?.name}, currentIndex: ${playerStore.currentIndex}`)
    if (!next) {
      logger.debug('[PrepareNext] No next song to prepare')
      return
    }

    try {
      const streamResult = await getApiClient().getAudioStreamUrl({
        container: next.container,
        itemId:    next.id,
        serverUrl: props.serverUrl,
        token:     props.token,
      })

      if (streamResult.status === 'ok') {
        await audioPlayer.prepareNext(streamResult.data, props.token)
        logger.debug(`[PrepareNext] Successfully prepared: ${next.name}`)
      }
    } catch (error) {
      logger.error('[PrepareNext] Failed to prepare next track:', error)
    }
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
  }

  const setEQEnabled = async (enabled: boolean): Promise<void> => {
    await audioPlayer.setEQEnabled(enabled)
    playerStore.setEQEnabled(enabled)
  }

  const setEQBand = async (band: number, gain: number): Promise<void> => {
    await audioPlayer.setEQBand(band, gain)
    playerStore.setEQBandGain(band, gain)
  }

  const resetEQ = async (): Promise<void> => {
    await audioPlayer.resetEQ()
    playerStore.resetEQ()
  }

  const seek = async (positionSecs: number): Promise<void> => {
    await audioPlayer.seek(positionSecs)
  }

  const resumeContext = async (): Promise<void> => {
    await audioPlayer.reinitialize()
  }

  const loadSong = async (song: null | Song): Promise<void> => {
    if (!song) {
      await audioPlayer.stop()
      playerStore.setAudioReady(false)
      return
    }

    try {
      playerStore.setAudioReady(false)
      playerStore.setBuffering(true)

      const streamResult = await getApiClient().getAudioStreamUrl(
        song.id,
        props.serverUrl,
        props.token,
        song.container ?? undefined,
      )

      if (streamResult.status === 'error') {
        logger.error('Failed to get audio stream URL:', streamResult.error)
        throw new Error(streamResult.error)
      }

      const loadResult = await audioPlayer.load(streamResult.data, props.token, {
        album:      song.album ?? null,
        artist:     song.artists?.join(', ') ?? null,
        artworkUrl: song.albumArtUrl ?? null,
        title:      song.name,
      })

      if (loadResult.success) {
        playerStore.setAudioReady(true)
        playerStore.setDuration(song.duration || loadResult.duration || 0)
        playerStore.setCurrentTime(0)
        logger.info(`Now playing: ${song.name}`)

        if (isDesktop()) {
          await updateNowPlaying(song)
          await updateMediaButtonStates()
          await prepareNextTrack()
        } else if (playerStore.isPlaying) {
          await audioPlayer.play()
        }
      } else {
        throw new Error('Failed to load audio')
      }
    } catch (error) {
      logger.error(`Failed to load audio for song ${song?.name}:`, error)
      playerStore.setAudioReady(false)
    } finally {
      playerStore.setBuffering(false)
    }
  }

  const playManuallyChangedSong = (song: Song): void => {
    playerStore.setAudioReady(false)
    playerStore.setBuffering(true)

    const execute = async (): Promise<void> => {
      await loadSong(song)
      const playing = await audioPlayer.isPlaying()
      if (playing) {
        playerStore.play()
      } else {
        playerStore.pause()
      }
    }
    execute()
  }

  const initializePlayer = async (): Promise<void> => {
    logger.info(`Initializing audio player (${isDesktop() ? 'Desktop/Rust' : 'Web'})...`)

    const initialized = await audioPlayer.initialize()
    if (!initialized) {
      logger.error('Failed to initialize audio player')
      return
    }

    logger.info('Audio player initialized successfully')

    // Set initial volume
    await audioPlayer.setVolume(playerStore.volume)

    // Restore EQ settings
    if (isDesktop()) {
      for (let i = 0; i < playerStore.eqBands.length; i++) {
        await audioPlayer.setEQBand(i, playerStore.eqBands[i].gain)
      }
    }
    await audioPlayer.setEQEnabled(playerStore.eqEnabled)
    logger.debug(`EQ restored: enabled=${playerStore.eqEnabled}`)

    // Setup event listeners
    audioPlayer.onPositionUpdate((event: AudioPosition) => {
      const { isFinished, position } = event

      if (!playerStore.isSeeking) {
        playerStore.setCurrentTime(position)
      }

      if (isFinished && playerStore.isPlaying) {
        handleTrackEnded()
      }
    })

    audioPlayer.onError((error: Error) => {
      logger.warn(`Audio stream error: ${error.message}`)
      playerStore.pause()
      playerStore.setNeedsReload(true)
    })

    audioPlayer.onTrackEnd(() => {
      // Track end is handled by position update with isFinished
      // This callback is intentionally empty - the handler is registered for API completeness
    })

    // Setup media control listeners (desktop only)
    if (isDesktop()) {
      await setupMediaEventListeners()
    }

    // For web, restore session song as ready for lazy loading
    if (!isDesktop() && playerStore.currentSong) {
      playerStore.setAudioReady(true)
    }
  }

  // Throttle volume updates
  let volumeThrottleTimer: null | ReturnType<typeof setTimeout> = null
  let pendingVolume: null | number = null
  const VOLUME_THROTTLE_MS = 50

  watch(() => playerStore.volume, newVolume => {
    pendingVolume = newVolume

    if (volumeThrottleTimer) return

    volumeThrottleTimer = setTimeout(async () => {
      if (pendingVolume !== null) {
        await audioPlayer.setVolume(pendingVolume)
        pendingVolume = null
      }
      volumeThrottleTimer = null
    }, VOLUME_THROTTLE_MS)
  })

  // Watch for play/pause from store
  watch(() => playerStore.isPlaying, async isPlaying => {
    const currentlyPlaying = await audioPlayer.isPlaying()

    if (isPlaying && !currentlyPlaying) {
      await audioPlayer.play()
    } else if (!isPlaying && currentlyPlaying) {
      await audioPlayer.pause()
    }

    // Sync playback status to OS Now Playing (desktop only)
    if (isDesktop() && getApiClient().mediaSetPlaybackStatus) {
      getApiClient().mediaSetPlaybackStatus?.(isPlaying, playerStore.currentTime).catch(() => {})
    }
  })

  // Watch for EQ enabled changes
  watch(() => playerStore.eqEnabled, async enabled => {
    await audioPlayer.setEQEnabled(enabled)
  })

  // Watch for queue position changes to update media button states
  watch([hasNext, hasPrevious, () => playerStore.repeatMode], () => {
    if (playerStore.currentSong) {
      updateMediaButtonStates()
    }
  })

  // Watch for song changes and auto-load
  watch(() => playerStore.currentSong?.id, async (newId, oldId) => {
    logger.debug(`Song watcher triggered: ${oldId} -> ${newId}, isGaplessTransition: ${isGaplessTransition.value}`)
    if (newId === oldId) return
    if (isGaplessTransition.value) {
      logger.debug('Skipping load - gapless transition in progress')
      return
    }

    const song = playerStore.currentSong
    if (song) {
      logger.debug(`Song changed to: ${song.name}, loading...`)
      await loadSong(song)
    }
  })

  return {
    audioPlayer,
    initializePlayer,
    isGaplessTransition,
    loadSong,
    nextSong,
    nextSongInQueue,
    playManuallyChangedSong,
    playSongAtIndex,
    resetEQ,
    resumeContext,
    seek,
    setEQBand,
    setEQEnabled,
  }
}
