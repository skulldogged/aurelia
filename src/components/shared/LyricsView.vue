<template>
  <div class='h-full flex flex-col'>
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
      <p
        v-for='(line, index) in parsedLyrics'
        :key='line.time + line.text'
        :ref='(el) => { if (index === currentLineIndex) activeLineRef = el as HTMLParagraphElement }'
        :class="['lyric-line', { 'active': index === currentLineIndex }]"
      >
        {{ line.text }}
      </p>
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

<script setup lang="ts">
  import { ref, watch, computed, nextTick } from 'vue'
  import { invoke } from '@tauri-apps/api/core'
  import { Loader2 } from 'lucide-vue-next'
  import { MusicItem } from '@/types'

  interface LyricLine {
    time: number
    text: string
  }

  const props = defineProps<{
    song:        MusicItem | null
    currentTime: number
  }>()

  const isLoading = ref(false)
  const lyrics = ref<string | null>(null)
  const error = ref<string | null>(null)
  const parsedLyrics = ref<LyricLine[]>([])
  const activeLineRef = ref<HTMLParagraphElement | null>(null)

  const areLyricsSynced = computed(() => {
    return lyrics.value ? /\[\d{2}:\d{2}\.\d{2,3}\]/.test(lyrics.value) : false
  })

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
        if (text) {
          result.push({ time, text })
        }
      }
    }
    return result
  }

  watch(() => props.song, async newSong => {
    if (newSong) {
      lyrics.value = null
      error.value = null
      isLoading.value = true
      try {
        if (newSong.artists && newSong.artists.length > 0) {
          const fetchedLyrics = await invoke<string>('get_lyrics', {
            artist: newSong.artists[0],
            title:  newSong.name,
          })
          lyrics.value = fetchedLyrics
          if (areLyricsSynced.value && fetchedLyrics) {
            parsedLyrics.value = parseLrc(fetchedLyrics)
          }
        } else {
          throw new Error('Artist not available')
        }
      } catch (err) {
        if (typeof err === 'string') {
          error.value = err
        } else if (err instanceof Error) {
          error.value = err.message
        } else {
          error.value = 'An unknown error occurred'
        }
      } finally {
        isLoading.value = false
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
    const index = parsedLyrics.value.findIndex(line => line.time > props.currentTime)
    if (index === -1) {
      return parsedLyrics.value.length - 1
    }
    return Math.max(0, index - 1)
  })

  watch(currentLineIndex, async (newIndex, oldIndex) => {
    if (newIndex !== oldIndex) {
      await nextTick()
      activeLineRef.value?.scrollIntoView({
        behavior: 'smooth',
        block:    'center',
      })
    }
  })
</script>

<style scoped>
.lyrics-container {
  text-align: center;
  overflow-x: hidden;
  /* For Firefox */
  scrollbar-width: none;
  /* For IE and Edge */
  -ms-overflow-style: none;
}

/* For Chrome, Safari, and Opera */
.lyrics-container::-webkit-scrollbar {
  display: none;
}

.lyric-line {
  padding: 8px 0;
  transition: all 0.3s ease;
  opacity: 0.5;
  font-size: 1.1rem;
}

.lyric-line.active {
  opacity: 1;
  font-weight: bold;
  transform: scale(1.1);
  color: var(--primary);
}

.prose {
  text-align: center;
}
</style>
