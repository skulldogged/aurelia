/**
 * Audio Engine Composable
 *
 * Unified audio orchestration layer using the AudioPlayer abstraction.
 * Platform-specific logic is handled internally by the AudioPlayer implementations.
 */

import { computed, type ComputedRef, onUnmounted, ref, type Ref, watch } from 'vue'

import type { Song } from '../lib/api/types'

import { type AudioPlayer, type AudioPosition, getAudioPlayer } from '../audio'
import { runAureliaEffect } from '../effect'
import { getAudioStreamUrlEffect } from '../effect/services/api'
import { logger } from '../lib/logger'
import { isElectron } from '../lib/platform'
import { usePlayerStore } from '../stores'

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
  const lastTrackEndedId = ref<null | string>(null)

  const hasNext = computed(() =>
    playerStore.canGoNext(),
  )

  const nextSongInQueue = computed(() => {
    const nextIndex = playerStore.getNextSongIndex(playerStore.repeatMode === 'all')
    if (nextIndex === -1) return null
    return playerStore.playlist[nextIndex] ?? null
  })

  const setupMediaSession = (): void => {
    if (typeof navigator === 'undefined' || !('mediaSession' in navigator)) return

    const session = navigator.mediaSession
    session.setActionHandler('play', () => {
      playerStore.play()
    })
    session.setActionHandler('pause', () => {
      playerStore.pause()
    })
    session.setActionHandler('previoustrack', () => {
      if (playerStore.canGoPrevious()) playerStore.previousSong()
    })
    session.setActionHandler('nexttrack', () => {
      nextSong()
    })
    session.setActionHandler('seekto', details => {
      if (typeof details.seekTime === 'number') {
        void audioPlayer.seek(details.seekTime)
      }
    })
  }

  const updateMediaSession = (song: null | Song): void => {
    if (typeof navigator === 'undefined' || !('mediaSession' in navigator)) return

    if (!song) {
      navigator.mediaSession.metadata = null
      navigator.mediaSession.playbackState = 'none'
      return
    }

    navigator.mediaSession.metadata = new MediaMetadata({
      album:    song.album ?? '',
      artist:   song.artists?.join(', ') ?? 'Unknown Artist',
      artwork:  song.albumArtUrl ? [{ src: song.albumArtUrl }] : [],
      title:    song.name,
    })
    navigator.mediaSession.playbackState = playerStore.isPlaying ? 'playing' : 'paused'
  }

  // Cleanup on unmount
  onUnmounted(() => {
    audioPlayer.destroy()
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
        if (isElectron()) {
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
    if (!isElectron()) return

    const next = nextSongInQueue.value
    logger.debug(`[PrepareNext] nextSongInQueue: ${next?.name}, currentIndex: ${playerStore.currentIndex}`)
    if (!next) {
      logger.debug('[PrepareNext] No next song to prepare')
      return
    }

    try {
      const streamUrl = await runAureliaEffect(getAudioStreamUrlEffect(
        next.id,
        props.serverUrl,
        props.token,
        next.container ?? undefined,
      ))
      await audioPlayer.prepareNext(streamUrl, props.token)
      logger.debug(`[PrepareNext] Successfully prepared: ${next.name}`)
    } catch (error) {
      logger.error('[PrepareNext] Failed to prepare next track:', error)
    }
  }

  const nextSong = (): void => playerStore.nextSong()

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
    // Only reinitialize (resume AudioContext) on web.
    // On desktop, reinit is too heavy to call on every play/pause as it restarts the audio thread.
    if (!isElectron()) {
      await audioPlayer.reinitialize()
    }
  }

  const loadSong = async (song: null | Song): Promise<void> => {
    if (!song) {
      await audioPlayer.stop()
      playerStore.setAudioReady(false)
      updateMediaSession(null)
      return
    }

    try {
      playerStore.setAudioReady(false)
      playerStore.setBuffering(true)

      const streamUrl = await runAureliaEffect(getAudioStreamUrlEffect(
        song.id,
        props.serverUrl,
        props.token,
        song.container ?? undefined,
      ))

      const loadResult = await audioPlayer.load(streamUrl, props.token, {
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

        updateMediaSession(song)
        if (isElectron()) {
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
    logger.info(`Initializing audio player (${isElectron() ? 'Electron/Rust' : 'Web'})...`)

    const initialized = await audioPlayer.initialize()
    if (!initialized) {
      logger.error('Failed to initialize audio player')
      return
    }

    logger.info('Audio player initialized successfully')

    // Set initial volume
    await audioPlayer.setVolume(playerStore.volume)

    // Restore EQ settings
    for (let i = 0; i < playerStore.eqBands.length; i++) {
      await audioPlayer.setEQBand(i, playerStore.eqBands[i].gain)
    }
    await audioPlayer.setEQEnabled(playerStore.eqEnabled)
    logger.debug(`EQ restored: enabled=${playerStore.eqEnabled}`)

    // Setup event listeners
    audioPlayer.onPositionUpdate((event: AudioPosition) => {
      const { didAutoAdvance, isFinished, position } = event as AudioPosition & { didAutoAdvance?: boolean }

      if (!playerStore.isSeeking) {
        playerStore.setCurrentTime(position)
      }

      // Handle auto-advance from Rust (gapless transition already happened)
      if (didAutoAdvance) {
        logger.debug('[Gapless] Auto-advance detected from Rust backend')
        const upcomingSong = nextSongInQueue.value
        if (upcomingSong) {
          // Mark as gapless transition to prevent song watcher from triggering loadSong
          isGaplessTransition.value = true

          // Update store to reflect the track that just started playing
          const newIndex = playerStore.playlist.findIndex(s => s.id === upcomingSong.id)
          playerStore.setCurrentSong(upcomingSong)
          if (newIndex !== -1) {
            playerStore.setCurrentIndex(newIndex)
          }
          playerStore.setCurrentTime(0)
          playerStore.setDuration(upcomingSong.duration || 0)
          playerStore.play()

          // Prepare the next track for seamless continuation (fire and forget)
          prepareNextTrack().catch(error => {
            logger.error('[Gapless] Failed to prepare next track after auto-advance:', error)
          })

          // Defer resetting the flag to next tick to ensure song watcher sees it as true
          setTimeout(() => {
            isGaplessTransition.value = false
          }, 0)
        }
        return
      }

      if (isFinished && lastTrackEndedId.value !== playerStore.currentSong?.id) {
        lastTrackEndedId.value = playerStore.currentSong?.id || null
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

    setupMediaSession()

    if (!isElectron() && playerStore.currentSong) {
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

    if (typeof navigator !== 'undefined' && 'mediaSession' in navigator) {
      navigator.mediaSession.playbackState = isPlaying ? 'playing' : 'paused'
    }
  })

  // Watch for EQ enabled changes
  watch(() => playerStore.eqEnabled, async enabled => {
    await audioPlayer.setEQEnabled(enabled)
  })

  // Watch for song changes and auto-load
  watch(() => playerStore.currentSong?.id, async (newId, oldId) => {
    logger.debug(`Song watcher triggered: ${oldId} -> ${newId}, isGaplessTransition: ${isGaplessTransition.value}`)
    if (newId === oldId) return

    // Reset last track ended ID when song changes
    lastTrackEndedId.value = null

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
