<script setup lang="ts">
  import { Loader2 } from 'lucide-vue-next'
  import { computed, nextTick, ref, watch } from 'vue'

  import { commands, Song } from '@/bindings'
  import { apiLogger } from '@/lib/logger'
  import { withCustomState } from '@/lib/result'

  interface LyricLine {
    text: string
    time: number
  }

  const props = defineProps<{
    currentTime: number
    duration:    number
    song:        null | Song
    visible:     boolean
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

  const { getLyrics } = commands

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

  watch(() => props.song, async newSong => {
    if (newSong) {
      lyrics.value = null
      error.value = null

      if (newSong.artists && newSong.artists.length > 0) {
        await withCustomState(
          () => getLyrics(
            newSong.id,
            newSong.artists![0],
            newSong.name,
            null,
          ),
          {
            onError: errorString => {
              error.value = errorString
              apiLogger.error('Failed to fetch lyrics:', errorString)
              isLoading.value = false
              emit('lyrics-loaded', false)
            },
            onStart: () => {
              isLoading.value = true
            },
            onSuccess: lyricsData => {
              lyrics.value = lyricsData
              if (areLyricsSynced.value && lyricsData) {
                parsedLyrics.value = parseLrc(lyricsData)
              }
              isLoading.value = false
              emit('lyrics-loaded', !!lyricsData)
            },
          },
        )
      } else {
        error.value = 'Artist not available'
        apiLogger.error('Lyrics loading error: Artist not available')
        isLoading.value = false
        emit('lyrics-loaded', false)
      }
    }
  }, { immediate: true })

  const formattedLyrics = computed(() => {
    if (!lyrics.value) return ''
    return lyrics.value.replace(/\[.*?\]/g, '').replace(/\n/g, '<br />')
  })

  const currentLineIndex = computed(() => {
    if (!areLyricsSynced.value || parsedLyrics.value.length === 0) {
      return -1
    }

    const tolerance = 0.01 // 10ms tolerance for floating point precision
    for (let i = parsedLyrics.value.length - 1; i >= 0; i--) {
      if (parsedLyrics.value[i].time <= props.currentTime + tolerance) {
        return i
      }
    }

    return -1
  })

  watch(currentLineIndex, async (newIndex, oldIndex) => {
    if (newIndex !== oldIndex && newIndex !== -1) {
      await nextTick()
      activeLineRef.value?.scrollIntoView({
        behavior: 'smooth',
        block:    'center',
      })
    }
  })

  watch(() => props.visible, async (isVisible, wasVisible) => {
    if (isVisible && !wasVisible && areLyricsSynced.value && currentLineIndex.value !== -1) {
      await nextTick()
      activeLineRef.value?.scrollIntoView({
        block: 'center',
      })
    }
  })

  watch(parsedLyrics, async newLyrics => {
    if (newLyrics && newLyrics.length > 0 && currentLineIndex.value === -1) {
      await nextTick()
      const firstLineRef = getFirstLineRef()
      if (firstLineRef) {
        firstLineRef.scrollIntoView({
          behavior: 'smooth',
          block:    'center',
        })
      }
    }
  }, { immediate: true })

  const getFirstLineRef = (): HTMLParagraphElement | null => {
    const lyricsContainer = document.querySelector('.lyrics-content')
    if (lyricsContainer) {
      return lyricsContainer.querySelector('p.lyric-line') as HTMLParagraphElement
    }
    return null
  }
</script>

<template>
  <div class='h-full w-full flex flex-col overflow-hidden'>
    <div
      v-if='isLoading'
      class='flex-grow flex items-center justify-center'
    >
      <Loader2 class='w-8 h-8 animate-spin' />
    </div>
    <div
      v-else-if='error'
      class='flex-grow flex items-center justify-center text-red-500'
    >
      {{ error }}
    </div>
    <div
      v-else-if='lyrics && areLyricsSynced'
      ref='lyricsContainerRef'
      class='lyrics-container flex-grow overflow-y-auto'
    >
      <div class='lyrics-content'>
        <p
          v-for='(line, index) in parsedLyrics'
          @click='handleLineClick(line.time)'
          :key='line.time + line.text'
          :ref='(el) => { if (index === currentLineIndex) activeLineRef = el as HTMLParagraphElement }'
          :class="['lyric-line', { 'active': index === currentLineIndex }]"
        >
          {{ line.text }}
        </p>
      </div>
    </div>
    <div
      v-else-if='lyrics'
      class='prose prose-invert max-w-none flex-grow overflow-y-auto'
      v-html='formattedLyrics'
    />
    <div
      v-else
      class='flex-grow flex items-center justify-center text-muted-foreground'
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

.lyrics-content {
  padding: 20vh 0;
  min-height: 60vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
}

/* For Chrome, Safari, and Opera */
.lyrics-container::-webkit-scrollbar {
  display: none;
}

.lyric-line {
  padding: 8px 0;
  transition: all 0.3s ease;
  opacity: 0.5;
  font-size: 2rem;
  filter: blur(2px);
  cursor: pointer;
}

.lyric-line.active {
  opacity: 1;
  font-weight: bold;
  transform: scale(1.15);
  color: var(--accent);
  filter: blur(0);
}

.prose {
  text-align: center;
}
</style>
