import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { computed, type ComputedRef, onUnmounted, ref, type Ref, watch } from 'vue'

import { commands, NowPlayingPayload } from '@/bindings'
import { Song } from '@/bindings'
import { useRustAudioPlayer } from '@/composables/useRustAudioPlayer'
import { logger } from '@/lib/logger'
import { usePlayerStore } from '@/stores'

interface AudioPositionEvent {
  isFinished: boolean
  position:   number
}

interface AudioStreamErrorEvent {
  position: number
  reason:   string
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
  const streamErrorUnlisten = ref<null | UnlistenFn>(null)
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

    // Listen for audio stream errors (device disconnected, etc.)
    if (streamErrorUnlisten.value) {
      streamErrorUnlisten.value()
      streamErrorUnlisten.value = null
    }

    streamErrorUnlisten.value = await listen<AudioStreamErrorEvent>('audio:stream-error', async event => {
      const { reason, position } = event.payload
      logger.warn(`Audio stream error: ${reason} at position ${position}`)

      // Pause playback on the UI side
      playerStore.pause()

      // Stop the backend player to clean up the dead stream
      await rustAudioPlayer.stop()

      // Store the position so we can resume from here if the user plays again
      playerStore.setCurrentTime(position)

      // Mark that we need to reload the audio when user presses play
      playerStore.setNeedsReload(true)

      logger.info('Playback paused due to audio stream error - press play to resume')
    })

    logger.debug('Audio stream error listener registered')
  }

  // Subscribe to media control events from backend (OS media keys)
  const setupMediaEventListeners = async (): Promise<void> => {
    // Clean up existing listeners
    for (const unlisten of mediaEventUnlisteners.value) {
      unlisten()
    }
    mediaEventUnlisteners.value = []

    // Listen for play/pause events from OS media keys
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
        logger.debug('Media key: Next - received event')
        logger.debug(`Current index before: ${playerStore.currentIndex}, hasNext: ${hasNext.value}`)
        nextSong()
        logger.debug(`Current index after: ${playerStore.currentIndex}, currentSong: ${playerStore.currentSong?.name}`)
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
    try {
      const payload: NowPlayingPayload = {
        album:        song.album ?? null,
        artist:       song.artists?.join(', ') ?? null,
        coverUrl:     song.albumArtUrl ?? null,
        durationSecs: song.duration ?? null,
        title:        song.name,
      }
      await commands.mediaUpdateNowPlaying(payload)
      logger.debug(`Updated OS Now Playing: ${song.name}`)
    } catch (error) {
      logger.error('Failed to update Now Playing:', error)
    }
  }

  // Update OS media control button states based on queue position
  const updateMediaButtonStates = async (): Promise<void> => {
    try {
      const canGoNext = hasNext.value || playerStore.repeatMode === 'all'
      const canGoPrevious = hasPrevious.value

      await Promise.all([
        commands.mediaSetButtonEnabled('next', canGoNext),
        commands.mediaSetButtonEnabled('previous', canGoPrevious),
      ])
      logger.debug(`Updated media buttons: next=${canGoNext}, previous=${canGoPrevious}`)
    } catch (error) {
      logger.error('Failed to update media button states:', error)
    }
  }

  // Cleanup on unmount
  onUnmounted(() => {
    if (eventUnlisten.value) {
      eventUnlisten.value()
      eventUnlisten.value = null
    }
    if (streamErrorUnlisten.value) {
      streamErrorUnlisten.value()
      streamErrorUnlisten.value = null
    }
    // Cleanup media event listeners
    for (const unlisten of mediaEventUnlisteners.value) {
      unlisten()
    }
    mediaEventUnlisteners.value = []
    // Clear Now Playing on unmount
    commands.mediaClearNowPlaying().catch(() => {})
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
    // loadSong is triggered by the watcher on currentSong.id
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

      const success = await rustAudioPlayer.play(streamResult.data, props.token, {
        title:      song.name,
        artist:     song.artists?.join(', ') ?? null,
        album:      song.album ?? null,
        artworkUrl: song.albumArtUrl ?? null,
      })

      if (success) {
        playerStore.setAudioReady(true)
        playerStore.setDuration(song.duration || 0)
        playerStore.setCurrentTime(0)
        logger.info(`Now playing: ${song.name}`)

        // Update OS Now Playing
        await updateNowPlaying(song)

        // Update media button states based on queue position
        await updateMediaButtonStates()

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

      // Setup media control event listeners (OS media keys)
      await setupMediaEventListeners()
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

    // Sync playback status to OS Now Playing widget
    commands.mediaSetPlaybackStatus(isPlaying, playerStore.currentTime).catch(() => {})
  })

  // Watch for EQ enabled changes
  watch(() => playerStore.eqEnabled, async enabled => {
    await rustAudioPlayer.setEQEnabled(enabled)
  })

  // Watch for queue position changes to update media button states
  watch([hasNext, hasPrevious, () => playerStore.repeatMode], () => {
    // Only update if we have a current song (player is active)
    if (playerStore.currentSong) {
      updateMediaButtonStates()
    }
  })

  // Watch for song changes and auto-load (handles media keys, queue clicks, etc.)
  watch(() => playerStore.currentSong?.id, async (newId, oldId) => {
    logger.debug(`Song watcher triggered: ${oldId} -> ${newId}, isGaplessTransition: ${isGaplessTransition.value}`)
    if (newId === oldId) return
    // Skip if we're in a gapless transition (handleTrackEnded manages this)
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
