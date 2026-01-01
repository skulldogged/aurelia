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
  import { computed, nextTick, onBeforeUnmount, onMounted, onUnmounted, ref, watch } from 'vue'

  import { commands } from '@/bindings'
  import { Song } from '@/bindings'
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
    initializePlayer,
    isGaplessTransition,
    loadSong,
    nextSong,
    playManuallyChangedSong,
    rustAudioPlayer,
  } = useAudioEngine(props)

  // Track whether audio has been loaded (vs just restored from session)
  const audioLoaded = ref(false)

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
      // Rust audio player handles seeking internally
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

    // EQ is always available with Rust backend
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
    // EQ is always available with Rust backend
    ['favorite', 'fullscreen', 'volume', 'equalizer', 'lyrics', 'queue']
      .some(icon => !visibleIcons.value.includes(icon)),
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
      if (playerStore.isPlaying) {
        await rustAudioPlayer.pause()
        playerStore.pause()
      } else {
        // If audio hasn't been loaded yet (restored session), load it first
        if (!audioLoaded.value && playerStore.currentSong) {
          await loadSong(playerStore.currentSong)
          audioLoaded.value = true
          playerStore.play()
        } else {
          await rustAudioPlayer.resume()
          playerStore.play()
        }
      }
    } catch (error) {
      logger.error('Playback error:', error)
    }
  }

  // Throttle seek updates to prevent audio crackling during slider drags
  const SEEK_THROTTLE_MS = 100
  let pendingSeekTime: null | number = null
  let seekThrottleTimer: null | ReturnType<typeof setTimeout> = null

  const onSeek = async (value: number[] | undefined): Promise<void> => {
    if (!value || !playerStore.audioReady) return

    const progressValue = value[0]
    const seekTime = (progressValue / 100) * playerStore.duration

    if (isFinite(seekTime)) {
      // Update UI immediately for responsiveness, mark as seeking to pause polling
      playerStore.setCurrentTime(seekTime, true)

      // Store the pending seek position
      pendingSeekTime = seekTime

      // If there's already a pending timer, let it handle the seek
      if (seekThrottleTimer) return

      // Set up throttled backend seek
      seekThrottleTimer = setTimeout(async () => {
        if (pendingSeekTime !== null) {
          await rustAudioPlayer.seek(pendingSeekTime)
          pendingSeekTime = null
        }
        // Clear seeking state after backend seek completes
        playerStore.setIsSeeking(false)
        seekThrottleTimer = null
      }, SEEK_THROTTLE_MS)
    }
  }

  // Volume updates go through the store, which has a throttled watcher in useAudioEngine
  const onVolumeInput = (value: number[] | undefined): void =>
    void(value?.length && playerStore.setVolume(value[0] / 100))

  // Cleanup throttle timer
  onBeforeUnmount(() => {
    if (seekThrottleTimer) clearTimeout(seekThrottleTimer)
  })

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
        // Gapless transition is handled by Rust backend
        audioLoaded.value = true
        playerStore.play()
      } else {
        playManuallyChangedSong(newSong)
        audioLoaded.value = true
      }
    } else if (!playerStore.currentSong) {
      rustAudioPlayer.stop()
      playerStore.pause()
      audioLoaded.value = false
    }
  })

  watch(
    () => playerStore.playlist,
    newPlaylist => {
      if (playerStore.currentSong) {
        const index = newPlaylist.findIndex(song => song.id === playerStore.currentSong!.id)
        playerStore.setCurrentIndex(index)
        // Next song loading is handled by useAudioEngine
      } else {
        playerStore.setCurrentIndex(-1)
      }
    },
  )

  onMounted(async () => {
    await initializePlayer()

    // If there's a restored session, sync UI state with backend
    if (playerStore.currentSong) {
      const index = playerStore.playlist.findIndex(song => song.id === playerStore.currentSong!.id)
      if (index !== -1) {
        playerStore.setCurrentIndex(index)
      }
      // Set duration from the restored song for proper seekbar display
      playerStore.setDuration(playerStore.currentSong.duration || 0)

      // Query backend for current playback state
      const [isPlaying, position] = await Promise.all([
        rustAudioPlayer.isPlaying(),
        rustAudioPlayer.getPosition(),
      ])

      // If backend has audio loaded (position > 0 or is playing), sync state
      if (isPlaying || position > 0) {
        audioLoaded.value = true
        playerStore.setCurrentTime(position)
        playerStore.setAudioReady(true)
        if (isPlaying) {
          playerStore.play()
        } else {
          playerStore.pause()
        }
      } else {
        // Backend has no audio loaded, mark as ready for lazy loading on first play
        playerStore.setAudioReady(true)
      }
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

  // Watch active view changes to update visible icons
  watch(activeView, () => {
    updateVisibleIcons()
  })

  onUnmounted(() => {
    rustAudioPlayer.stop()

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
    class='player-bar'
  >
    <div ref='containerRef' class='player-bar-inner'>
      <!-- Mobile layout -->
      <template v-if='isMobilePortraitMode'>
        <div class='flex items-center gap-3 flex-1 min-w-0'>
          <!-- Album art -->
          <div @click="emit('toggle-fullscreen')" class='shrink-0 player-album-art'>
            <ImageLoader
              :item-id='playerStore.currentSong.albumId || playerStore.currentSong.id'
              :server-url='serverUrl'
              :token='token'
              alt='Album art'
              class='size-12 rounded-lg'
            >
              <template #fallback>
                <div class='size-12 bg-muted rounded-lg flex items-center justify-center'>
                  <Music2 class='size-5 text-muted-foreground' />
                </div>
              </template>
            </ImageLoader>
          </div>

          <!-- Song info -->
          <div class='flex-1 min-w-0' @click="emit('toggle-fullscreen')">
            <p class='font-medium text-sm truncate'>{{ playerStore.currentSong.name }}</p>
            <p class='text-xs text-muted-foreground truncate'>
              {{ playerStore.currentSong.artists?.join(', ') || 'Unknown Artist' }}
            </p>
          </div>

          <!-- Play button -->
          <Button
            @click.stop='togglePlayPause'
            :disabled='!playerStore.audioReady || playerStore.isBuffering'
            class='shrink-0 rounded-full! size-11'
            size='icon'
            variant='default'
          >
            <Loader2 v-if='playerStore.isBuffering' class='size-5 animate-spin' />
            <Play v-else-if='!playerStore.isPlaying' class='size-5 ml-0.5' />
            <Pause v-else class='size-5' />
          </Button>
        </div>
      </template>

      <!-- Desktop layout -->
      <template v-else>
        <!-- Left: Song info -->
        <div
          @mouseenter='onTitleMouseEnter'
          @mouseleave='onTitleMouseLeave'
          class='flex items-center gap-3 min-w-0 flex-1'
        >
          <div @click="emit('toggle-fullscreen')" class='shrink-0 player-album-art'>
            <ImageLoader
              :item-id='playerStore.currentSong.albumId || playerStore.currentSong.id'
              :server-url='serverUrl'
              :token='token'
              alt='Album art'
              class='size-12 rounded-lg'
            >
              <template #fallback>
                <div class='size-12 bg-muted rounded-lg flex items-center justify-center'>
                  <Music2 class='size-5 text-muted-foreground' />
                </div>
              </template>
            </ImageLoader>
          </div>

          <div class='min-w-0 flex-1'>
            <div ref='titleContainerRef' class='overflow-hidden'>
              <div
                @animationiteration='handleMarqueeIteration'
                ref='marqueeTrackRef'
                :class="['marquee-track', isMarqueePaused && 'marquee-paused']"
                :style='marqueeStyle'
              >
                <span ref='titleTextRef' class='font-medium text-sm whitespace-nowrap'>
                  {{ playerStore.currentSong.name }}
                </span>
                <span v-if='shouldMarquee' aria-hidden='true' class='font-medium text-sm whitespace-nowrap'>
                  {{ playerStore.currentSong.name }}
                </span>
              </div>
            </div>
            <p class='text-xs text-muted-foreground truncate'>
              <template v-if='playerStore.currentSong.artists?.length'>
                <template v-for='(artist, i) in playerStore.currentSong.artists' :key='i'>
                  <RouterLink
                    v-if='playerStore.currentSong.artistIds?.[i]'
                    :to='`/artists/${playerStore.currentSong.artistIds[i]}`'
                    class='hover:underline'
                  >{{ artist }}</RouterLink>
                  <span v-else>{{ artist }}</span>
                  <span v-if='i < playerStore.currentSong.artists.length - 1'>, </span>
                </template>
              </template>
              <template v-if='playerStore.currentSong.album'>
                <span class='mx-1 opacity-50'>·</span>
                <RouterLink
                  v-if='playerStore.currentSong.albumId'
                  :to='`/albums/${playerStore.currentSong.albumId}`'
                  class='hover:underline'
                >{{ playerStore.currentSong.album }}</RouterLink>
                <span v-else>{{ playerStore.currentSong.album }}</span>
              </template>
            </p>
            <p v-if='songFormatInfo' class='text-[10px] text-muted-foreground/60 truncate'>
              {{ songFormatInfo }}
            </p>
          </div>
        </div>

        <!-- Center: Controls & Progress -->
        <div class='flex-1 max-w-lg px-4'>
          <div class='flex items-center justify-center gap-1'>
            <Button
              @click='playerStore.toggleShuffle'
              :class="['player-control-btn', playerStore.isShuffled && 'is-active']"
              size='icon'
              variant='ghost'
            >
              <Shuffle class='size-4' />
            </Button>

            <Button
              @click='previousSong'
              :disabled='!hasPrevious'
              class='player-control-btn'
              size='icon'
              variant='ghost'
            >
              <SkipBack class='size-4' />
            </Button>

            <Button
              @click='togglePlayPause'
              :disabled='!playerStore.audioReady || playerStore.isBuffering'
              class='player-play-btn'
              size='icon'
              variant='default'
            >
              <Loader2 v-if='playerStore.isBuffering' class='size-4 animate-spin' />
              <Play v-else-if='!playerStore.isPlaying' class='size-4 ml-0.5' />
              <Pause v-else class='size-4' />
            </Button>

            <Button
              @click='nextSong'
              :disabled='!hasNext'
              class='player-control-btn'
              size='icon'
              variant='ghost'
            >
              <SkipForward class='size-4' />
            </Button>

            <Button
              @click='playerStore.cycleRepeatMode'
              :class="['player-control-btn', playerStore.repeatMode !== 'none' && 'is-active']"
              size='icon'
              variant='ghost'
            >
              <Repeat1 v-if="playerStore.repeatMode === 'one'" class='size-4' />
              <Repeat v-else class='size-4' />
            </Button>
          </div>

          <div class='flex items-center gap-2 mt-1.5'>
            <span class='text-[10px] text-muted-foreground tabular-nums w-8 text-right'>
              {{ formatTime(playerStore.currentTime) }}
            </span>
            <Slider
              @update:model-value='onSeek'
              :max='100'
              :model-value='[playerStore.progress]'
              :step='0.1'
              class='flex-1'
            />
            <span class='text-[10px] text-muted-foreground tabular-nums w-8'>
              {{ formatTime(playerStore.duration) }}
            </span>
          </div>
        </div>

        <!-- Right: Secondary controls -->
        <div class='flex items-center justify-end gap-1 flex-1'>
          <Button
            @click="emit('toggle-queue')"
            v-if="visibleIcons.includes('queue')"
            :class="['player-control-btn', activeView === 'queue' && 'is-active']"
            size='icon'
            variant='ghost'
          >
            <ListMusic class='size-4' />
          </Button>

          <Button
            @click="emit('toggle-lyrics')"
            v-if="visibleIcons.includes('lyrics')"
            :class="['player-control-btn', activeView === 'lyrics' && 'is-active']"
            :disabled='!hasLyrics'
            size='icon'
            variant='ghost'
          >
            <Mic2 class='size-4' />
          </Button>

          <Button
            @click="emit('toggle-favorite', playerStore.currentSong)"
            v-if="visibleIcons.includes('favorite')"
            :class="['player-control-btn', playerStore.currentSong.isFavorite && 'is-active']"
            size='icon'
            variant='ghost'
          >
            <Heart :class="['size-4', playerStore.currentSong.isFavorite && 'fill-current']" />
          </Button>

          <Button
            @click="emit('toggle-fullscreen')"
            v-if="visibleIcons.includes('fullscreen')"
            class='player-control-btn'
            size='icon'
            variant='ghost'
          >
            <Expand class='size-4' />
          </Button>

          <Button
            @click="emit('toggle-equalizer')"
            v-if="visibleIcons.includes('equalizer')"
            :class="['player-control-btn', activeView === 'equalizer' && 'is-active']"
            size='icon'
            variant='ghost'
          >
            <Sliders class='size-4' />
          </Button>

          <!-- Volume -->
          <div v-if="visibleIcons.includes('volume')" class='relative'>
            <Button
              @click='handleVolumeClick'
              :class="['player-control-btn', isVolumePopupVisible && 'is-active']"
              size='icon'
              variant='ghost'
              data-volume-button
            >
              <Volume2 v-if='playerStore.volume > 0.5' class='size-4' />
              <Volume1 v-else-if='playerStore.volume > 0' class='size-4' />
              <VolumeX v-else class='size-4' />
            </Button>
            <Transition name='pop'>
              <div v-if='isVolumePopupVisible' ref='volumePopupRef' class='volume-popup'>
                <span class='text-xs text-muted-foreground tabular-nums'>
                  {{ Math.round(playerStore.volume * 100) }}%
                </span>
                <Slider
                  @update:model-value='onVolumeInput'
                  :max='100'
                  :model-value='[playerStore.volume * 100]'
                  :step='1'
                  class='h-20 w-1.5'
                  orientation='vertical'
                />
                <button
                  @click.stop='playerStore.toggleMute'
                  class='p-1 text-muted-foreground hover:text-foreground transition-colors'
                >
                  <VolumeX v-if='playerStore.volume === 0' class='size-4' />
                  <Volume2 v-else class='size-4' />
                </button>
              </div>
            </Transition>
          </div>

          <!-- Overflow menu -->
          <DropdownMenu v-if='hasHiddenIcons'>
            <DropdownMenuTrigger as-child>
              <Button class='player-control-btn' size='icon' variant='ghost'>
                <MoreHorizontal class='size-4' />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align='end'>
              <DropdownMenuItem v-if="!visibleIcons.includes('volume')">
                <div class='flex items-center gap-2 w-full'>
                  <button @click='playerStore.toggleMute' class='text-muted-foreground hover:text-foreground'>
                    <Volume2 v-if='playerStore.volume > 0.5' class='size-4' />
                    <Volume1 v-else-if='playerStore.volume > 0' class='size-4' />
                    <VolumeX v-else class='size-4' />
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
              <DropdownMenuItem @click="emit('toggle-equalizer')" v-if="!visibleIcons.includes('equalizer')">
                <Sliders class='size-4 mr-2' />
                Equalizer
              </DropdownMenuItem>
              <DropdownMenuItem @click="emit('toggle-fullscreen')" v-if="!visibleIcons.includes('fullscreen')">
                <Expand class='size-4 mr-2' />
                Fullscreen
              </DropdownMenuItem>
              <DropdownMenuItem
                @click="emit('toggle-favorite', playerStore.currentSong)"
                v-if="!visibleIcons.includes('favorite')"
              >
                <Heart :class="['size-4 mr-2', playerStore.currentSong.isFavorite && 'fill-current']" />
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
              <DropdownMenuItem @click="emit('toggle-queue')" v-if="!visibleIcons.includes('queue')">
                <ListMusic class='size-4 mr-2' />
                Queue
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </template>
    </div>
  </div>
</template>
<style scoped>
/* Player bar container */
.player-bar {
  height: 88px;
  padding: 0.625rem 0.75rem;
  position: relative;
}

.player-bar-inner {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  max-width: 100%;
  margin: 0 auto;
  height: 100%;
}

/* Album art with hover effect */
.player-album-art {
  cursor: pointer;
  transition: transform 0.2s ease, box-shadow 0.2s ease;
  border-radius: 0.5rem;
}

.player-album-art:hover {
  transform: scale(1.03);
}

.player-album-art:active {
  transform: scale(0.98);
}

/* Control buttons */
.player-control-btn {
  transition: all 0.15s ease;
}

.player-control-btn:hover:not(:disabled) {
  background: var(--accent);
  color: var(--accent-foreground);
}

.player-control-btn.is-active {
  background: var(--accent);
  color: var(--accent-foreground);
}

.player-control-btn:disabled {
  opacity: 0.4;
}

/* Play button */
.player-play-btn {
  border-radius: 9999px !important;
  width: 2.5rem;
  height: 2.5rem;
  transition: all 0.15s ease;
}

.player-play-btn:hover:not(:disabled) {
  transform: scale(1.05);
}

.player-play-btn:active:not(:disabled) {
  transform: scale(0.97);
}

/* Volume popup */
.volume-popup {
  position: absolute;
  bottom: 100%;
  left: 50%;
  transform: translateX(-50%);
  margin-bottom: 0.375rem;
  padding: 0.625rem;
  background: var(--card);
  border: 1px solid var(--border);
  border-radius: 0.5rem;
  box-shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.2);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.375rem;
  z-index: 50;
}

/* Pop transition */
.pop-enter-active,
.pop-leave-active {
  transition: all 0.15s cubic-bezier(0.32, 0.72, 0, 1);
}

.pop-enter-from,
.pop-leave-to {
  opacity: 0;
  transform: translateX(-50%) scale(0.9) translateY(0.25rem);
}

/* Marquee animation */
.marquee-track {
  display: inline-flex;
  align-items: center;
  gap: var(--marquee-gap, 24px);
  will-change: transform;
}

.marquee-track:hover {
  animation: marquee-scroll var(--marquee-duration, 10s) linear infinite;
}

.marquee-paused {
  animation-play-state: paused;
}

@keyframes marquee-scroll {
  0% { transform: translateX(0); }
  100% { transform: translateX(calc(-1 * var(--scroll-distance, 300px))); }
}
</style>
