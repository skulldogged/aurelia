<script setup lang="ts">
  import { AlertTriangle, Loader2 } from 'lucide-vue-next'
  import { computed, nextTick, ref, watch } from 'vue'

  import type { Song } from '../../lib/api/types'

  import { ApiError, runAureliaEffect } from '../../effect'
  import { getLyricsEffect } from '../../effect/services/api'
  import { logger } from '../../lib/logger'

  interface LyricLine {
    text: string
    time: number
  }

  const props = defineProps<{
    currentTime:  number
    duration:     number
    isInSidebar?: boolean
    size?:        'large' | 'normal' | 'small'
    song:         null | Song
    visible:      boolean
  }>()

  const emit = defineEmits<{
    (e: 'seek', time: number): void
    (e: 'lyrics-loaded', hasLyrics: boolean): void
  }>()

  const isLoading = ref(false)
  const lyrics = ref<null | string>(null)
  const error = ref<null | string>(null)
  const parsedLyrics = ref<LyricLine[]>([])
  const activeLineRef = ref<HTMLParagraphElement | null>(null)
  const lyricsContainerRef = ref<HTMLDivElement | null>(null)
  const currentLyricsRequestToken = ref<null | symbol>(null)

  const areLyricsSynced = computed(() => lyrics.value ? /\[\d{2}:\d{2}\.\d{2,3}\]/.test(lyrics.value) : false)

  const handleLineClick = (time: number): void => {
    if (props.duration > 0)
      emit('seek', time)
  }

  const parseLrc = (lrc: string): LyricLine[] => {
    const lines = lrc.split('\n')
    const result: LyricLine[] = []
    const timeRegex = /\[(\d{2}):(\d{2})\.(\d{2,3})\]/

    for (const line of lines) {
      const match = line.match(timeRegex)
      if (match) {
        const minutes = parseInt(match[1], 10)
        const seconds = parseInt(match[2], 10)
        const milliseconds = parseInt(match[3].padEnd(3, '0'), 10)
        const time = minutes * 60 + seconds + milliseconds / 1000
        const text = line.replace(timeRegex, '').trim()
        if (text)
          result.push({ text, time })
      }
    }

    return result
  }

  watch(() => props.song?.id, async (newId, oldId) => {
    if (newId === oldId && props.song) return
    const newSong = props.song
    if (newSong) {
      lyrics.value = null
      error.value = null
      parsedLyrics.value = []

      if (newSong.artists && newSong.artists.length > 0) {
        const requestToken = Symbol('lyrics-request')
        currentLyricsRequestToken.value = requestToken

        if (currentLyricsRequestToken.value === requestToken)
          isLoading.value = true

        try {
          const lyricsData = await runAureliaEffect(getLyricsEffect(
            newSong.id,
            newSong.artists![0],
            newSong.name,
            undefined,
          ))
          if (currentLyricsRequestToken.value !== requestToken) return
          lyrics.value = lyricsData
          if (areLyricsSynced.value && lyricsData) {
            parsedLyrics.value = parseLrc(lyricsData)
          }
          emit('lyrics-loaded', !!lyricsData)
        } catch (cause) {
          if (currentLyricsRequestToken.value !== requestToken) return
          const errorString = cause instanceof ApiError
            ? cause.message
            : String(cause)
          error.value = errorString
          logger.error('Failed to fetch lyrics:', errorString)
          emit('lyrics-loaded', false)
        } finally {
          if (currentLyricsRequestToken.value === requestToken) {
            isLoading.value = false
          }
        }
      } else {
        error.value = 'Artist not available'
        logger.error('Lyrics loading error: Artist not available')
        isLoading.value = false
        emit('lyrics-loaded', false)
      }
    }
  }, { immediate: true })

  const plainLyrics = computed(() => {
    if (!lyrics.value) return []

    return lyrics.value
      .replace(/\[.*?\]/g, '')
      .split('\n')
      .map(line => line.trim())
      .filter(line => line.length > 0)
  })

  const currentLineIndex = computed(() => {
    if (!areLyricsSynced.value || parsedLyrics.value.length === 0)
      return -1

    const tolerance = 0.01 // 10ms tolerance for floating point precision
    for (let i = parsedLyrics.value.length - 1; i >= 0; i--)
      if (parsedLyrics.value[i].time <= props.currentTime + tolerance)
        return i

    return -1
  })

  // Scroll to center an element within the lyrics container
  const scrollToCenter = (element: HTMLElement, smooth = true): void => {
    const container = lyricsContainerRef.value
    if (!container || !element) return

    const containerRect = container.getBoundingClientRect()
    const elementRect = element.getBoundingClientRect()

    // Calculate the scroll position to center the element
    const elementCenter = elementRect.top + elementRect.height / 2
    const containerCenter = containerRect.top + containerRect.height / 2
    const scrollOffset = elementCenter - containerCenter

    container.scrollBy({
      behavior: smooth ? 'smooth' : 'instant',
      top:      scrollOffset,
    })
  }

  watch(currentLineIndex, async (newIndex, oldIndex) => {
    if (newIndex !== oldIndex && newIndex !== -1) {
      await nextTick()
      if (activeLineRef.value) {
        scrollToCenter(activeLineRef.value, true)
      }
    }
  })

  watch(() => props.visible, async (isVisible, wasVisible) => {
    if (isVisible && !wasVisible && areLyricsSynced.value && currentLineIndex.value !== -1) {
      // Wait for panel animation to complete (350ms) before scrolling
      // Without this delay, getBoundingClientRect returns incorrect values
      // because the container is still animating from width: 0
      await new Promise(resolve => setTimeout(resolve, 400))
      await nextTick()
      if (activeLineRef.value) {
        scrollToCenter(activeLineRef.value, false)
      }
    }
  })

  watch(parsedLyrics, async newLyrics => {
    if (newLyrics && newLyrics.length > 0 && currentLineIndex.value === -1) {
      await nextTick()
      // Scroll to top of lyrics content
      if (lyricsContainerRef.value) {
        lyricsContainerRef.value.scrollTop = 0
      }
    }
  }, { immediate: true })
</script>

<template>
  <div class='size-full flex flex-col overflow-hidden'>
    <div
      v-if='isInSidebar'
      class='h-12 flex items-center justify-between px-4 shrink-0'
      data-tauri-drag-region
    >
      <h2 class='text-base font-semibold tracking-tight text-muted-foreground'>
        Lyrics
      </h2>
    </div>
    <div
      v-if='isLoading'
      class='grow flex items-center justify-center'
    >
      <Loader2 class='size-8 animate-spin' />
    </div>
    <div
      v-else-if='error'
      class='grow flex items-center justify-center px-6'
    >
      <div class='w-full max-w-md bg-card/60 border border-destructive/30 rounded-2xl shadow-lg backdrop-blur-sm p-6'>
        <div class='flex items-start space-x-4'>
          <div class='p-3 rounded-xl bg-destructive/10 text-destructive'>
            <AlertTriangle class='size-6' />
          </div>
          <div class='space-y-2 text-left'>
            <h3 class='text-lg font-semibold'>
              Unable to load lyrics
            </h3>
            <p class='lyrics-error-message text-sm text-muted-foreground'>
              {{ error }}
            </p>
          </div>
        </div>
      </div>
    </div>
    <div
      v-else-if='lyrics && areLyricsSynced'
      ref='lyricsContainerRef'
      :class="['lyrics-container grow overflow-y-auto', { 'sidebar': isInSidebar }]"
    >
      <div class='lyrics-content'>
        <p
          v-for='(line, index) in parsedLyrics'
          @click='handleLineClick(line.time)'
          :key='line.time + line.text'
          :ref='(el) => { if (index === currentLineIndex) activeLineRef = el as HTMLParagraphElement }'
          :class="['lyric-line', {
            'active': index === currentLineIndex,
            'sidebar': isInSidebar,
            'large': size === 'large',
            'small': size === 'small'
          }]"
        >
          {{ line.text }}
        </p>
      </div>
    </div>
    <div
      v-else-if='lyrics'
      :class="['lyrics-container grow overflow-y-auto', { 'sidebar': isInSidebar }]"
    >
      <div class='lyrics-content lyrics-content--static'>
        <p
          v-for='(line, index) in plainLyrics'
          :key='`${index}-${line}`'
          :class="[
            'lyric-line lyric-line--static',
            { 'sidebar': isInSidebar, 'large': size === 'large', 'small': size === 'small' }
          ]"
        >
          {{ line }}
        </p>
      </div>
    </div>
    <div
      v-else
      class='grow flex items-center justify-center text-muted-foreground'
    >
      No lyrics found for this song.
    </div>
  </div>
</template>

<style scoped>
.lyrics-container {
  padding: 0 32px;
  text-align: center;
  overflow-x: hidden;
  /* For Firefox */
  scrollbar-width: none;
  /* For IE and Edge */
  -ms-overflow-style: none;
  mask-image: linear-gradient(to bottom, transparent 0%, black 20%, black 80%, transparent 100%);
  -webkit-mask-image: linear-gradient(to bottom, transparent 0%, black 20%, black 80%, transparent 100%);
}

.lyrics-container.sidebar {
  padding: 0 16px;
}

.lyrics-content {
  padding: 20vh 16px;
  min-height: 60vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
}

.lyrics-content.sidebar {
  padding: 15vh 0;
}

/* For Chrome, Safari, and Opera */
.lyrics-container::-webkit-scrollbar {
  display: none;
}

.lyric-line {
  padding: 8px 0;
  transition: opacity 0.3s ease, transform 0.3s ease, color 0.3s ease;
  opacity: 0.4;
  font-size: 2rem;
  cursor: pointer;
}

.lyric-line.sidebar {
  font-size: 1.5rem;
  padding: 6px 0;
}

.lyric-line.large:not(.sidebar) {
  font-size: 2.5rem;
}

.lyric-line.small:not(.sidebar) {
  font-size: 1.5rem;
}

.lyric-line.sidebar.active {
  transform: scale(1.1);
}

.lyric-line.active {
  opacity: 1;
  font-weight: bold;
  transform: scale(1.15);
  color: var(--accent);
}

.lyrics-content--static {
  padding: 16vh 0;
}

.lyrics-content--static.sidebar {
  padding: 12vh 0;
}

.lyric-line--static {
  opacity: 0.85;
  cursor: default;
}

.lyrics-error-message {
  overflow-wrap: anywhere;
  word-break: break-word;
}
</style>
