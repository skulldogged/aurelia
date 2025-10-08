<script setup lang="ts">
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

  const props = defineProps<{
    allSongs:       Song[]
    credentials:    Credentials
    currentSong:    null | Song
    isPlaying:      boolean
    libraryLoaded:  boolean
    libraryLoading: boolean
  }>()

  const emit = defineEmits<{
    'play-song':       [song: Song]
    'toggle-favorite': [song: Song]
  }>()

  const searchQuery = ref('')

  const { layout: viewLayout } = useLayoutPreference('songlist-layout', 'comfy')
  const { sort: sortOption } = useSortPreference('songlist-sort', 'Title')

  const sortingOptions = ['Title', 'Artist', 'Album', 'Date Added', 'Play Count']

  const songFuse = ref<Fuse<Song>>()

  watch(() => props.allSongs, newSongs => {
    if (newSongs && newSongs.length > 0) {
      songFuse.value = new Fuse(newSongs, {
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
  }, { immediate: true })

  const filteredSongs = computed(() =>
    searchQuery.value && searchQuery.value.length >= 2 && songFuse.value
      ? songFuse.value.search(searchQuery.value).map(result => result.item)
      : props.allSongs,
  )

  const sortedSongs = computed(() => {
    switch (sortOption.value) {
      case 'Album':
        return [...filteredSongs.value].sort((a, b) => (a.album || '').localeCompare(b.album || ''))
      case 'Artist':
        return [...filteredSongs.value].sort((a, b) => (a.artists?.[0] || '').localeCompare(b.artists?.[0] || ''))
      case 'Date Added':
        return [...filteredSongs.value].sort((a, b) => (b.dateCreated || '').localeCompare(a.dateCreated || ''))
      case 'Play Count':
        return [...filteredSongs.value].sort((a, b) => (b.playCount || 0) - (a.playCount || 0))
      case 'Title':
        return [...filteredSongs.value].sort((a, b) => a.name.localeCompare(b.name))
      default:
        return [...filteredSongs.value]
    }
  })

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
            @play-song='playSong'
            @toggle-favorite='handleToggleFavorite'
            :current-song='props.currentSong'
            :is-playing='props.isPlaying'
            :layout='viewLayout'
            :loading='libraryLoading'
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
