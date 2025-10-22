<script setup lang="ts">
  import { refDebounced } from '@vueuse/core'
  import Fuse from 'fuse.js'
  import { computed, ref, watch } from 'vue'

  import type { Credentials, Song } from '@/bindings'

  import SongList from '@/components/shared/SongList.vue'
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
  import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs'
  import { useLayoutPreference, useSortPreference } from '@/composables/useLayoutPreference'
  import { useAuthStore } from '@/stores/auth'
  import { useLibraryStore } from '@/stores/library'

  defineProps<{
    credentials: Credentials
  }>()

  const emit = defineEmits<{
    'play-instant-mix': [song: Song]
    'play-song':        [song: Song]
    'toggle-favorite':  [song: Song]
  }>()

  const authStore = useAuthStore()
  const libraryStore = useLibraryStore()

  // Create computed properties from stores
  const allSongs = computed(() => libraryStore.allSongs as Song[])
  const libraryLoading = computed(() => libraryStore.isLoading)
  const serverUrl = computed(() => authStore.serverUrl)
  const token = computed(() => authStore.token)

  const searchQuery = ref('')
  const debouncedSearchQuery = refDebounced(searchQuery, 300)

  const { layout: viewLayout } = useLayoutPreference('songlist-layout', 'comfy')
  const { sort: sortOption } = useSortPreference('songlist-sort', 'Title')

  const sortingOptions = ['Title', 'Artist', 'Album', 'Date Added', 'Play Count']

  const songFuse = ref<Fuse<Song>>()

  watch(() => allSongs.value, newSongs => {
    if (newSongs && newSongs.length > 0) {
      songFuse.value = new Fuse(newSongs as Song[], {
        includeScore: true,
        keys:         [
          { name: 'name', weight: 0.5 },
          { name: 'artists', weight: 0.3 },
          { name: 'album', weight: 0.2 },
        ],
        minMatchCharLength: 2,
        threshold:          0.2,
      })
    }
  })

  const filteredSongs = computed(() =>
    debouncedSearchQuery.value && debouncedSearchQuery.value.length >= 2 && songFuse.value
      ? songFuse.value.search(debouncedSearchQuery.value).map(result => result.item)
      : allSongs.value as Song[],
  )

  const sortedSongs = ref<Song[]>([])

  watch([filteredSongs, sortOption], ([newFilteredSongs, newSortOption]) => {
    const songsToSort = [...newFilteredSongs]
    switch (newSortOption) {
      case 'Album':
        songsToSort.sort((a, b) => (a.album || '').localeCompare(b.album || ''))
        break
      case 'Artist':
        songsToSort.sort((a, b) => (a.artists?.[0] || '').localeCompare(b.artists?.[0] || ''))
        break
      case 'Date Added':
        songsToSort.sort((a, b) => (b.dateCreated || '').localeCompare(a.dateCreated || ''))
        break
      case 'Play Count':
        songsToSort.sort((a, b) => (b.playCount || 0) - (a.playCount || 0))
        break
      case 'Title':
        songsToSort.sort((a, b) => a.name.localeCompare(b.name))
        break
    }
    sortedSongs.value = songsToSort
  }, { immediate: true })

  const playSong = (song: Song): void => {
    emit('play-song', song)
  }

  const handleToggleFavorite = (song: Song): void => {
    emit('toggle-favorite', song)
  }
</script>

<template>
  <div class='h-full flex flex-col'>
    <div class='max-w-7xl mx-auto p-4 w-full'>
      <div class='w-full'>
        <div class='mb-8'>
          <div class='flex justify-between items-start mb-4'>
            <h1 class='text-4xl font-bold'>
              Songs
            </h1>
            <Tabs v-model='viewLayout'>
              <TabsList>
                <TabsTrigger value='comfy'>
                  Comfy
                </TabsTrigger>
                <TabsTrigger value='compact'>
                  Compact
                </TabsTrigger>
              </TabsList>
            </Tabs>
          </div>
          <div class='flex flex-col sm:flex-row gap-4 items-start sm:items-center justify-between'>
            <Input
              v-model='searchQuery'
              class='max-w-sm focus-visible:ring-1 focus-visible:ring-accent border-0 focus-visible:border-accent'
              placeholder='Search songs...'
              type='text'
            />

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
          </div>
        </div>

        <div class='w-full'>
          <SongList
            @play-instant-mix='$emit("play-instant-mix", $event)'
            @play-song='playSong'
            @toggle-favorite='handleToggleFavorite'
            :layout='viewLayout'
            :loading='libraryLoading'
            :server-url='serverUrl'
            :show-album='true'
            :show-album-art='true'
            :show-artist='true'
            :show-duration='true'
            :show-track-number='true'
            :show-year='true'
            :songs='sortedSongs'
            :token='token'
          />
        </div>
      </div>
    </div>
  </div>
</template>