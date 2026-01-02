import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { computed, type ComputedRef, onUnmounted, ref, type Ref, watch } from 'vue'

import { commands } from '@/bindings'
import { Song } from '@/bindings'
import { useRustAudioPlayer } from '@/composables/useRustAudioPlayer'
import { logger } from '@/lib/logger'
import { usePlayerStore } from '@/stores'

interface AudioPositionEvent {
  isFinished: boolean
  position:   number
}

interface UseAudioEngineReturn {
  initializePlayer:        () => Promise<void>;
  isGaplessTransition:     Ref<boolean>;
  loadSong:                (song: null | Song) => Promise<void>;
  nextSong:                () => void;
  nextSongInQueue:         ComputedRef<null | Song>;
  playManuallyChangedSong: (song: Song) => void;
  playSongAtIndex:         (index: number) => void;
  rustAudioPlayer:         ReturnType<typeof useRustAudioPlayer>;
}

export const useAudioEngine = (
  props: { serverUrl: string; token: string },
): UseAudioEngineReturn => {
  const playerStore = usePlayerStore()
  const { getAudioStreamUrl } = commands

  // Rust audio player
  const rustAudioPlayer = useRustAudioPlayer()

  // State
  const isGaplessTransition = ref(false)
  const eventUnlisten = ref<null | UnlistenFn>(null)

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

  // Subscribe to audio position events from Rust backend
  // This replaces frontend polling with push-based events
  const setupEventListener = async (): Promise<void> => {
    // Clean up existing listener
    if (eventUnlisten.value) {
      eventUnlisten.value()
      eventUnlisten.value = null
    }

    eventUnlisten.value = await listen<AudioPositionEvent>('audio:position', async event => {
      const { isFinished, position } = event.payload

      // Update current position (skip while user is seeking)
      if (!playerStore.isSeeking) {
        playerStore.setCurrentTime(position)
      }

      // Handle track end
      if (isFinished && playerStore.isPlaying) {
        await handleTrackEnded()
      }
    })

    logger.debug('Audio position event listener registered')
  }

  // Cleanup on unmount
  onUnmounted(() => {
    if (eventUnlisten.value) {
      eventUnlisten.value()
      eventUnlisten.value = null
    }
  })

  const handleTrackEnded = async (): Promise<void> => {
    logger.debug('Track ended')

    if (playerStore.repeatMode === 'one') {
      // Replay current song
      const song = playerStore.currentSong
      if (song) {
        await loadSong(song)
        playerStore.play()
      }
    } else if (playerStore.repeatMode === 'all' || hasNext.value) {
      // Capture the next song ONCE to avoid re-evaluation (important for shuffle mode)
      const upcomingSong = nextSongInQueue.value
      logger.debug(`[Gapless] upcomingSong: ${upcomingSong?.name} (id: ${upcomingSong?.id}), currentIndex: ${playerStore.currentIndex}`)
      if (upcomingSong) {
        isGaplessTransition.value = true
        logger.debug('[Gapless] Set isGaplessTransition = true, calling advanceGapless')
        const success = await rustAudioPlayer.advanceGapless()
        logger.debug(`[Gapless] advanceGapless returned: ${success}`)
        if (success) {
          logger.debug(`[Gapless] Setting currentSong to: ${upcomingSong.name}`)

          // Update both song and index so prepareNextTrack gets the correct next song
          const newIndex = playerStore.playlist.findIndex(s => s.id === upcomingSong.id)
          playerStore.setCurrentSong(upcomingSong)
          if (newIndex !== -1) {
            playerStore.setCurrentIndex(newIndex)
          }

          playerStore.setCurrentTime(0)
          playerStore.setDuration(upcomingSong.duration || 0)
          playerStore.play()

          // Prepare next track for gapless
          logger.debug(`[Gapless] Calling prepareNextTrack (currentIndex is now ${playerStore.currentIndex})`)
          await prepareNextTrack()
        } else {
          // Fallback to regular next song - reset flag first so watcher loads the song
          logger.debug('[Gapless] advanceGapless failed, falling back to nextSong')
          isGaplessTransition.value = false
          nextSong()
        }
        isGaplessTransition.value = false
      } else if (playerStore.repeatMode === 'all') {
        playSongAtIndex(0)
      }
    } else {
      playerStore.pause()
    }
  }

  const prepareNextTrack = async (): Promise<void> => {
    const next = nextSongInQueue.value
    logger.debug(`[PrepareNext] nextSongInQueue: ${next?.name} (id: ${next?.id}), currentIndex: ${playerStore.currentIndex}`)
    if (!next) {
      logger.debug('[PrepareNext] No next song to prepare')
      return
    }

    try {
      const streamResult = await getAudioStreamUrl(
        props.serverUrl,
        props.token,
        next.id,
        next.container,
      )

      if (streamResult.status === 'ok') {
        await rustAudioPlayer.prepareNext(streamResult.data, props.token)
        logger.debug(`[PrepareNext] Successfully prepared: ${next.name} (id: ${next.id})`)
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
    loadSong(playerStore.playlist[index])
  }

  const loadSong = async (song: null | Song): Promise<void> => {
    if (!song) {
      await rustAudioPlayer.stop()
      playerStore.setAudioReady(false)
      return
    }

    try {
      playerStore.setAudioReady(false)
      playerStore.setBuffering(true)

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

      const success = await rustAudioPlayer.play(streamResult.data, props.token)

      if (success) {
        playerStore.setAudioReady(true)
        playerStore.setDuration(song.duration || 0)
        playerStore.setCurrentTime(0)
        logger.info(`Now playing: ${song.name}`)

        // Prepare next track for gapless playback
        await prepareNextTrack()
      } else {
        throw new Error('Failed to play audio via Rust backend')
      }
    } catch (error) {
      logger.error(`Failed to load audio for song ${song.name} (ID: ${song.id}):`, error)
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

      if (await rustAudioPlayer.isPlaying()) playerStore.play()
      else playerStore.pause()
    }
    execute()
  }

  const initializePlayer = async (): Promise<void> => {
    logger.info('Initializing Rust audio player...')

    const initialized = await rustAudioPlayer.init()
    if (initialized) {
      logger.info('Rust audio player initialized successfully')

      // Set initial volume
      await rustAudioPlayer.setVolume(playerStore.volume)

      // Restore EQ settings from store (bands first, then enabled state)
      // Always sync bands so toggling EQ on/off is instant
      for (let i = 0; i < playerStore.eqBands.length; i++) {
        await rustAudioPlayer.setEQBand(i, playerStore.eqBands[i].gain)
      }
      await rustAudioPlayer.setEQEnabled(playerStore.eqEnabled)
      logger.debug(`EQ restored: enabled=${playerStore.eqEnabled}, bands synced`)

      // Setup event listener for position updates from Rust backend
      await setupEventListener()
    } else {
      logger.error('Failed to initialize Rust audio player')
    }
  }

  // Throttle volume updates to backend to prevent audio issues during slider drags
  let volumeThrottleTimer: null | ReturnType<typeof setTimeout> = null
  let pendingVolume: null | number = null
  const VOLUME_THROTTLE_MS = 50

  watch(() => playerStore.volume, newVolume => {
    pendingVolume = newVolume

    if (volumeThrottleTimer) return

    volumeThrottleTimer = setTimeout(async () => {
      if (pendingVolume !== null) {
        await rustAudioPlayer.setVolume(pendingVolume)
        pendingVolume = null
      }
      volumeThrottleTimer = null
    }, VOLUME_THROTTLE_MS)
  })

  // Watch for play/pause from store
  watch(() => playerStore.isPlaying, async isPlaying => {
    const currentlyPlaying = await rustAudioPlayer.isPlaying()
    if (isPlaying && !currentlyPlaying)
      await rustAudioPlayer.resume()
    else if (!isPlaying && currentlyPlaying)
      await rustAudioPlayer.pause()
  })

  // Watch for EQ enabled changes
  watch(() => playerStore.eqEnabled, async enabled => {
    await rustAudioPlayer.setEQEnabled(enabled)
  })

  return {
    initializePlayer,
    isGaplessTransition,
    loadSong,
    nextSong,
    nextSongInQueue,
    playManuallyChangedSong,
    playSongAtIndex,
    rustAudioPlayer,
  }
}
