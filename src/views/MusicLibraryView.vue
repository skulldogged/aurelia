<template>
  <div class='h-full flex flex-col'>
    <div class='max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8 w-full'>
      <div class='w-full'>
        <div class='mb-8'>
          <h1 class='text-4xl font-bold mb-4'>
            Songs
          </h1>
          <div class='flex flex-col sm:flex-row gap-4 items-start sm:items-center justify-between'>
            <div class='flex flex-col sm:flex-row gap-4 items-start sm:items-center'>
              <Input
                v-model='searchQuery'
                class='max-w-sm focus-visible:ring-1 focus-visible:ring-accent border-0 focus-visible:border-accent'
                placeholder='Search songs...'
                type='text'
              />
              <!-- Dev toggle for skeleton adjustment -->
              <Button
                @click='showSkeleton = !showSkeleton'
                :variant='showSkeleton ? "default" : "outline"'
                size='sm'
              >
                {{ showSkeleton ? 'Hide' : 'Show' }} Skeleton (dev)
              </Button>
            </div>

            <!-- Layout Controls -->
            <div class='flex items-center gap-2'>
              <span class='text-sm text-muted-foreground'>View:</span>
              <div class='flex rounded-md border'>
                <Button
                  @click='viewLayout = "list"'
                  :variant='viewLayout === "list" ? "default" : "ghost"'
                  class='rounded-r-none border-r-0'
                  size='sm'
                >
                  List
                </Button>
                <Button
                  @click='viewLayout = "compact"'
                  :variant='viewLayout === "compact" ? "default" : "ghost"'
                  class='rounded-none border-r-0'
                  size='sm'
                >
                  Compact
                </Button>
                <Button
                  @click='viewLayout = "grid"'
                  :variant='viewLayout === "grid" ? "default" : "ghost"'
                  class='rounded-l-none'
                  size='sm'
                >
                  Grid
                </Button>
              </div>
            </div>
          </div>
        </div>

        <div v-if='(hasError || songsState.error) && !showSkeleton && !loading && !dataLoading' class='text-center py-12'>
          <p class='text-destructive mb-4'>
            {{ songsState.error || 'Failed to load songs' }}
          </p>
          <Button @click='fetchMusicLibrary' variant='destructive'>
            Try Again
          </Button>
        </div>
        <div v-else class='w-full'>
          <SongList
            @play-song='playSong'
            @toggle-favorite='handleToggleFavorite'
            :current-song='props.currentSong'
            :is-playing='props.isPlaying'
            :layout='viewLayout'
            :loading='loading || dataLoading || showSkeleton'
            :server-url='props.credentials.serverUrl'
            :show-album='true'
            :show-album-art='true'
            :show-artist='true'
            :show-duration='true'
            :show-track-number='true'
            :show-year='true'
            :songs='filteredSongs'
            :token='props.credentials.token'
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
  import { ref, onMounted, watch, computed } from 'vue'
  import SongList from '@/components/shared/SongList.vue'
  import { Button } from '@/components/ui/button'
  import { Input } from '@/components/ui/input'
  import { Skeleton } from '@/components/ui/skeleton'
  import { Heart } from 'lucide-vue-next'
  import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
  } from '@/components/ui/table'
  import type { Song, Credentials } from '@/bindings'
  import Fuse from 'fuse.js'
  import { useDataFetching } from '@/composables/useDataFetching'
  import { useLayoutPreference } from '@/composables/useLayoutPreference'

  // Define props from parent
  const props = defineProps<{
    currentSong: Song | null
    isPlaying:   boolean
    credentials: Credentials
  }>()

  // Define emits for parent
  const emit = defineEmits<{
    'play-song':       [song: Song]
    'toggle-favorite': [song: Song]
  }>()

  // Use data fetching composable
  const { songs, isLoading: dataLoading, hasError, songsState, fetchSongs } = useDataFetching(props.credentials)

  // Search functionality
  const searchQuery = ref('')

  // Layout preference with localStorage persistence
  const { layout: viewLayout } = useLayoutPreference('songlist-layout', 'list')

  // Local search implementation using Fuse
  const songFuse = ref<Fuse<Song>>()

  // Initialize search when songs change
  watch(songs, newSongs => {
    if (newSongs && newSongs.length > 0) {
      songFuse.value = new Fuse(newSongs, {
        keys: [
          { name: 'name', weight: 0.5 },
          { name: 'artists', weight: 0.3 },
          { name: 'album', weight: 0.2 },
        ],
        includeScore:       true,
        threshold:          0.2,
        minMatchCharLength: 2,
      })
    }
  }, { immediate: true })

  // Computed filtered songs
  const filteredSongs = computed(() => {
    if (!searchQuery.value || searchQuery.value.length < 2 || !songFuse.value) {
      return songs.value
    }
    return songFuse.value.search(searchQuery.value).map(result => result.item)
  })

  // Local loading state for initial fetch
  const loading = ref(false)
  const showSkeleton = ref(false) // Temporary dev toggle for adjusting skeleton sizes

  // Fetch data on mount
  const fetchMusicLibrary = async () => {
    loading.value = true
    try {
      await fetchSongs()
    } catch {
      // Error is handled by the composable
    } finally {
      loading.value = false
    }
  }

  // Initialize data on mount
  onMounted(fetchMusicLibrary)

  // Watch for credentials changes
  watch(() => props.credentials, newCredentials => {
    if (newCredentials) {
      fetchMusicLibrary()
    }
  }, { immediate: false })

  // Methods
  const playSong = (song: Song) => {
    emit('play-song', song)
  }

  const handleToggleFavorite = (song: Song) => {
    emit('toggle-favorite', song)
  }
</script>
