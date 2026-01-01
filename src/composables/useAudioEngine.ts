import { computed, type ComputedRef, ref, type Ref, watch } from 'vue'

import { commands } from '@/bindings'
import { Song } from '@/bindings'
import { useRustAudioPlayer } from '@/composables/useRustAudioPlayer'
import { logger } from '@/lib/logger'
import { usePlayerStore } from '@/stores'

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
  const pollInterval = ref<null | number>(null)

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

  // Start polling for playback state and position
  const startPolling = (): void => {
    if (pollInterval.value)
      clearInterval(pollInterval.value)

    pollInterval.value = window.setInterval(async () => {
      // Update current position from Rust backend (skip while user is seeking)
      if (playerStore.isPlaying && !playerStore.isSeeking) {
        const position = await rustAudioPlayer.getPosition()
        playerStore.setCurrentTime(position)
      }

      // Check if track finished
      const finished = await rustAudioPlayer.isFinished()
      if (finished && playerStore.isPlaying)
        await handleTrackEnded()
    }, 250) // Poll more frequently for smooth progress bar
  }

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
      // Advance to next song
      if (nextSongInQueue.value) {
        isGaplessTransition.value = true
        const success = await rustAudioPlayer.advanceGapless()
        if (success) {
          playerStore.setCurrentSong(nextSongInQueue.value)
          playerStore.setCurrentTime(0)
          playerStore.setDuration(nextSongInQueue.value.duration || 0)
          playerStore.play()

          // Prepare next track for gapless
          await prepareNextTrack()
        } else {
          // Fallback to regular next song
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
    if (!next) return

    try {
      const streamResult = await getAudioStreamUrl(
        props.serverUrl,
        props.token,
        next.id,
        next.container,
      )

      if (streamResult.status === 'ok') {
        await rustAudioPlayer.prepareNext(streamResult.data, props.token)
        logger.debug(`Prepared next track: ${next.name}`)
      }
    } catch (error) {
      logger.error('Failed to prepare next track:', error)
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

      // Start polling for track end
      startPolling()
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
