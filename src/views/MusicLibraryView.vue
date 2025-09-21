<template>
  <div class='h-full flex flex-col'>
    <div class='max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8 w-full'>
      <div class='w-full'>
        <div class='mb-8'>
          <h1 class='text-4xl font-bold mb-4'>
            Songs
          </h1>
          <div class='flex flex-col sm:flex-row gap-4 items-start sm:items-center justify-between'>
            <Input
              v-model='searchQuery'
              class='max-w-sm focus-visible:ring-1 focus-visible:ring-accent border-0 focus-visible:border-accent'
              placeholder='Search songs...'
              type='text'
            />
            <div class='flex items-center gap-4'>
              <!-- Dev toggle for skeleton adjustment -->
              <Button
                @click='showSkeleton = !showSkeleton'
                :variant='showSkeleton ? "default" : "outline"'
                size='sm'
              >
                {{ showSkeleton ? 'Hide' : 'Show' }} Skeleton (dev)
              </Button>
              <!-- Sort Controls -->
              <Select v-model='sortOption'>
                <SelectTrigger class='w-[180px]'>
                  <SelectValue placeholder='Sort by' />
                </SelectTrigger>
                <SelectContent>
                  <SelectGroup>
                    <SelectLabel>Sort by</SelectLabel>
                    <SelectItem v-for='option in sortingOptions' :key='option' :value='option'>
                      {{ option }}
                    </SelectItem>
                  </SelectGroup>
                </SelectContent>
              </Select>

              <!-- Layout Controls -->
              <div class='flex items-center gap-2'>
                <div class='flex rounded-md border'>
                  <Button
                    @click='viewLayout = "comfy"'
                    :variant='viewLayout === "comfy" ? "default" : "ghost"'
                    class='rounded-r-none border-r-0'
                    size='sm'
                  >
                    Comfy
                  </Button>
                  <Button
                    @click='viewLayout = "compact"'
                    :variant='viewLayout === "compact" ? "default" : "ghost"'
                    class='rounded-l-none'
                    size='sm'
                  >
                    Compact
                  </Button>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div
          v-if='(hasError || songsState.error) && !showSkeleton && !loading && !dataLoading'
          class='text-center py-12'
        >
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
            :songs='sortedSongs'
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
  import {
    Select,
    SelectContent,
    SelectGroup,
    SelectItem,
    SelectLabel,
    SelectTrigger,
    SelectValue,
  } from '@/components/ui/select'
  import type { Song, Credentials } from '@/bindings'
  import Fuse from 'fuse.js'
  import { useDataFetching } from '@/composables/useDataFetching'
  import { useLayoutPreference, useSortPreference } from '@/composables/useLayoutPreference'

  const props = defineProps<{
    currentSong: Song | null
    isPlaying:   boolean
    credentials: Credentials
  }>()

  const emit = defineEmits<{
    'play-song':       [song: Song]
    'toggle-favorite': [song: Song]
  }>()

  const { songs, isLoading: dataLoading, hasError, songsState, fetchSongs } = useDataFetching(props.credentials)

  const searchQuery = ref('')

  const { layout: viewLayout } = useLayoutPreference('songlist-layout', 'comfy')
  const { sort: sortOption } = useSortPreference('songlist-sort', 'Title')

  const sortingOptions = ['Title', 'Artist', 'Album', 'Date Added', 'Play Count']

  const songFuse = ref<Fuse<Song>>()

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

  const filteredSongs = computed(() => {
    if (!searchQuery.value || searchQuery.value.length < 2 || !songFuse.value) {
      return songs.value
    }
    return songFuse.value.search(searchQuery.value).map(result => result.item)
  })

  const sortedSongs = computed(() => {
    const songsToSort = [...filteredSongs.value]
    switch (sortOption.value) {
      case 'Title':
        return songsToSort.sort((a, b) => a.name.localeCompare(b.name))
      case 'Artist':
        return songsToSort.sort((a, b) => (a.artists?.[0] || '').localeCompare(b.artists?.[0] || ''))
      case 'Album':
        return songsToSort.sort((a, b) => (a.album || '').localeCompare(b.album || ''))
      case 'Date Added':
        return songsToSort.sort((a, b) => (b.dateCreated || '').localeCompare(a.dateCreated || ''))
      case 'Play Count':
        return songsToSort.sort((a, b) => (b.playCount || 0) - (a.playCount || 0))
      default:
        return songsToSort
    }
  })

  const loading = ref(false)
  const showSkeleton = ref(false) // Temporary dev toggle for adjusting skeleton sizes

  const fetchMusicLibrary = async () => {
    loading.value = true
    try {
      await fetchSongs()
    } catch (err) {
      console.error('Failed to fetch music library:', err)
    } finally {
      loading.value = false
    }
  }

  onMounted(fetchMusicLibrary)

  watch(() => props.credentials, newCredentials => {
    if (newCredentials) {
      fetchMusicLibrary()
    }
  }, { immediate: false })

  const playSong = (song: Song) => {
    emit('play-song', song)
  }

  const handleToggleFavorite = (song: Song) => {
    emit('toggle-favorite', song)
  }
</script>
