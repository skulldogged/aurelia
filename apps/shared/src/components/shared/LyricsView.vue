<script setup lang="ts">
  import { AlertTriangle, Loader2 } from 'lucide-vue-next'
  import { computed, nextTick, ref, watch } from 'vue'

  import type { ParsedLyrics, Song } from '../../lib/api/types'

  import { ApiError, runAureliaEffect } from '../../effect'
  import { getLyricsEffect, getParsedLyricsEffect } from '../../effect/services/api'
  import { logger } from '../../lib/logger'

  /** Map agent IDs to their type for quick lookup. */
  type AgentMap = Record<string, string>

  interface LyricLine {
    agentId: null | string
    endTime: null | number
    text:    string
    time:    number
    words:   LyricWord[] | null
  }

  interface LyricWord {
    endTime: null | number
    time:    number
    word:    string
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
  const parsedLyricsResponse = ref<null | ParsedLyrics>(null)
  const error = ref<null | string>(null)
  const activeLineRef = ref<HTMLParagraphElement | null>(null)
  const lyricsContainerRef = ref<HTMLDivElement | null>(null)
  const currentLyricsRequestToken = ref<null | symbol>(null)

  /** Build a lookup from agent ID to agent type (e.g. "person" | "other"). */
  const agentMap = computed<AgentMap>(() => {
    const map: AgentMap = {}
    for (const agent of parsedLyricsResponse.value?.agents ?? [])
      map[agent.id] = agent.agentType
    return map
  })

  /** Section labels indexed by line start time for display as dividers. */
  const sectionLabels = computed<Record<number, string>>(() => {
    const labels: Record<number, string> = {}
    for (const section of parsedLyricsResponse.value?.sections ?? []) {
      if (section.name && section.lines.length > 0)
        labels[section.lines[0].timeMs] = section.name
    }
    return labels
  })

  const parsedLyrics = computed<LyricLine[]>(() =>
    parsedLyricsResponse.value?.synced?.map(line => ({
      agentId: line.agentId ?? null,
      endTime: line.endTimeMs != null ? line.endTimeMs / 1000 : null,
      text:    line.line,
      time:    line.timeMs / 1000,
      words:   line.words?.map(w => ({
        endTime: w.endTimeMs != null ? w.endTimeMs / 1000 : null,
        time:    w.timeMs / 1000,
        word:    w.word,
      })) ?? null,
    })) ?? [],
  )
  const areLyricsSynced = computed(() => parsedLyrics.value.length > 0)
  const hasLyrics = computed(() => {
    const hasFallback = !!lyrics.value
    const hasParsedPlain = (parsedLyricsResponse.value?.plain?.length ?? 0) > 0
    return hasFallback || hasParsedPlain || areLyricsSynced.value
  })

  const handleLineClick = (time: number): void => {
    if (props.duration > 0)
      emit('seek', time)
  }

  /** Check if an agent ID refers to a background/other voice. */
  const isBackgroundVocal = (agentId: null | string): boolean => {
    if (!agentId) return false
    return agentMap.value[agentId] === 'other'
  }

  /** Get the section label for a given line, if it starts a new section. */
  const getSectionLabel = (timeMs: number): string | undefined =>
    sectionLabels.value[timeMs]

  watch(() => props.song?.id, async (newId, oldId) => {
    if (newId === oldId && props.song) return
    const newSong = props.song
    if (newSong) {
      lyrics.value = null
      parsedLyricsResponse.value = null
      error.value = null

      if (newSong.artists && newSong.artists.length > 0) {
        const requestToken = Symbol('lyrics-request')
        currentLyricsRequestToken.value = requestToken

        if (currentLyricsRequestToken.value === requestToken)
          isLoading.value = true

        try {
          try {
            const parsedData = await runAureliaEffect(getParsedLyricsEffect(
              newSong.id,
              newSong.artists![0],
              newSong.name,
              newSong.path ?? undefined,
            ))
            if (currentLyricsRequestToken.value !== requestToken) return
            parsedLyricsResponse.value = parsedData

            const hasParsedLyrics = (parsedData.synced?.length ?? 0) > 0 || (parsedData.plain?.length ?? 0) > 0
            if (hasParsedLyrics) {
              emit('lyrics-loaded', true)
              return
            }
          } catch {
            // Fallback to raw endpoint for older backends.
          }

          const lyricsData = await runAureliaEffect(getLyricsEffect(
            newSong.id,
            newSong.artists![0],
            newSong.name,
            newSong.path ?? undefined,
          ))
          if (currentLyricsRequestToken.value !== requestToken) return
          lyrics.value = lyricsData
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
    if ((parsedLyricsResponse.value?.plain?.length ?? 0) > 0)
      return parsedLyricsResponse.value!.plain

    if (!lyrics.value)
      return []

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
    const time = props.currentTime + tolerance

    // Use end times when available for more precise detection
    for (let i = parsedLyrics.value.length - 1; i >= 0; i--) {
      const line = parsedLyrics.value[i]
      if (line.time <= time) {
        // If we have an end time, check we haven't passed it
        if (line.endTime != null && time > line.endTime) {
          // We're past this line's end — check if the next line has started
          if (i < parsedLyrics.value.length - 1 && parsedLyrics.value[i + 1].time <= time) {
            continue // Skip, a later line is active
          }
          // We're in a gap between lines — keep this as active for continuity
        }
        return i
      }
    }

    return -1
  })

  /** For the active line, determine which word is currently being sung. */
  const activeWordIndex = computed(() => {
    const lineIdx = currentLineIndex.value
    if (lineIdx === -1) return -1
    const line = parsedLyrics.value[lineIdx]
    if (!line.words || line.words.length === 0) return -1

    const tolerance = 0.01
    const time = props.currentTime + tolerance

    for (let i = line.words.length - 1; i >= 0; i--) {
      if (line.words[i].time <= time)
        return i
    }
    return -1
  })

  /** Check if a specific word in the active line has been sung (is at or past its start time). */
  const isWordSung = (lineIdx: number, wordIdx: number): boolean => {
    if (lineIdx !== currentLineIndex.value) return false
    return wordIdx <= activeWordIndex.value
  }

  /**
   * Compute the progress (0-1) through the currently active word for gradient fill.
   * Returns 1 for fully-sung words, 0 for upcoming words.
   */
  const wordProgress = (lineIdx: number, wordIdx: number): number => {
    if (lineIdx !== currentLineIndex.value) return 0
    if (wordIdx < activeWordIndex.value) return 1
    if (wordIdx > activeWordIndex.value) return 0

    // This is the currently active word — compute partial progress
    const line = parsedLyrics.value[lineIdx]
    if (!line.words) return 0
    const word = line.words[wordIdx]
    const endTime = word.endTime
      ?? line.words[wordIdx + 1]?.time
      ?? line.endTime
      ?? (word.time + 0.5)
    const duration = endTime - word.time
    if (duration <= 0) return 1
    const elapsed = props.currentTime - word.time
    return Math.max(0, Math.min(1, elapsed / duration))
  }

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
      v-else-if='hasLyrics && areLyricsSynced'
      ref='lyricsContainerRef'
      :class="['lyrics-container grow overflow-y-auto', { 'sidebar': isInSidebar }]"
    >
      <div class='lyrics-content'>
        <template v-for='(line, index) in parsedLyrics' :key='line.time + line.text'>
          <div
            v-if='getSectionLabel(line.time * 1000)'
            :class="{ 'sidebar': isInSidebar }"
            class='section-label'
          >
            {{ getSectionLabel(line.time * 1000) }}
          </div>
          <p
            @click='handleLineClick(line.time)'
            :ref='(el) => { if (index === currentLineIndex) activeLineRef = el as HTMLParagraphElement }'
            :class="['lyric-line', {
              'active': index === currentLineIndex,
              'word-synced': index === currentLineIndex && line.words && line.words.length > 0,
              'background-vocal': isBackgroundVocal(line.agentId),
              'sidebar': isInSidebar,
              'large': size === 'large',
              'small': size === 'small'
            }]"
          >
            <template v-if='line.words && line.words.length > 0 && index === currentLineIndex'>
              <span
                v-for='(word, wIdx) in line.words'
                :key='wIdx'
                :class='{
                  "sung": isWordSung(index, wIdx) && wIdx !== activeWordIndex,
                  "filling": wIdx === activeWordIndex,
                }'
                :style='wIdx === activeWordIndex ? {
                  "--fill": `${wordProgress(index, wIdx) * 100}%`,
                } : undefined'
                class='lyric-word'
              >{{ word.word }}</span>
            </template>
            <template v-else>
              {{ line.text }}
            </template>
          </p>
        </template>
      </div>
    </div>
    <div
      v-else-if='hasLyrics'
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

.section-label {
  padding: 16px 0 4px;
  font-size: 0.75rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  opacity: 0.35;
  color: var(--muted-foreground);
}

.section-label.sidebar {
  padding: 12px 0 2px;
  font-size: 0.65rem;
}

.lyric-line {
  padding: 8px 0;
  transition: opacity 0.3s ease, transform 0.3s ease, color 0.3s ease;
  opacity: 0.4;
  font-size: 2rem;
  cursor: pointer;
}

.lyric-line.background-vocal {
  font-style: italic;
  opacity: 0.3;
}

.lyric-line.background-vocal.active {
  opacity: 0.75;
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

/* Word-synced active lines use per-word coloring instead of line-level accent */
.lyric-line.word-synced {
  color: var(--muted-foreground);
}

.lyric-word {
  -webkit-text-fill-color: var(--muted-foreground);
  color: var(--muted-foreground);
}

/* Sung words: fully colored with accent */
.lyric-word.sung {
  -webkit-text-fill-color: var(--accent);
  color: var(--accent);
}

/* Active word: gradient fill from accent to muted based on progress */
.lyric-word.filling {
  background-image: linear-gradient(
    to right,
    var(--accent),
    var(--accent) var(--fill, 0%),
    var(--muted-foreground) var(--fill, 0%),
    var(--muted-foreground)
  );
  background-clip: text;
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  color: transparent;
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
