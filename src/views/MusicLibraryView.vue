<template>
  <div class='h-full flex flex-col'>
    <div class='max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8'>
      <div>
        <div class='mb-8'>
          <h1 class='text-4xl font-bold mb-4'>
            Songs
          </h1>
          <Input
            v-model='searchQuery'
            class='max-w-sm focus-visible:ring-1 focus-visible:ring-accent border-0 focus-visible:border-accent'
            placeholder='Search songs...'
            type='text'
          />
        </div>

        <div v-if='props.loading' class='flex justify-center items-center py-12'>
          <div class='text-muted-foreground'>
            Loading songs...
          </div>
        </div>
        <div v-else-if='props.error' class='text-center py-12'>
          <p class='text-destructive mb-4'>
            {{ props.error }}
          </p>
          <Button @click="$emit('reload-library')" variant='destructive'>
            Try Again
          </Button>
        </div>
        <div v-else>
          <SongList
            @play-song='playSong'
            @toggle-favorite='handleToggleFavorite'
            :current-song='props.currentSong'
            :is-playing='props.isPlaying'
            :show-album='true'
            :show-album-art='true'
            :show-artist='true'
            :show-duration='true'
            :show-track-number='true'
            :show-year='true'
            :songs='filteredSongs'
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
  import { ref, computed, watch } from 'vue'
  import Fuse from 'fuse.js'
  import SongList from '@/components/shared/SongList.vue'
  import { Button } from '@/components/ui/button'
  import { Input } from '@/components/ui/input'
  import { MusicItem, ArtistInfo, AlbumInfo } from '@/types'

  // Define props from parent
  const props = defineProps<{
    songs:       MusicItem[]
    artists:     ArtistInfo[]
    albums:      AlbumInfo[]
    loading:     boolean
    error:       string
    currentSong: MusicItem | null
    isPlaying:   boolean
  }>()

  // Define emits for parent
  const emit = defineEmits<{
    'play-song':       [song: MusicItem]
    'toggle-favorite': [song: MusicItem]
    'reload-library':  []
  }>()

  // Component state
  const searchQuery = ref('')

  // Fuzzy Search setup
  const songFuse = ref(new Fuse(props.songs, {
    keys: [
      { name: 'name', weight: 0.5 },
      { name: 'artists', weight: 0.3 },
      { name: 'album', weight: 0.2 },
    ],
    includeScore:       true,
    threshold:          0.2,
    minMatchCharLength: 2,
  }))

  watch(() => props.songs, newSongs => songFuse.value.setCollection(newSongs))

  // Computed properties for filtering
  const filteredSongs = computed(() => {
    if (!searchQuery.value || searchQuery.value.length < 2) return props.songs
    return songFuse.value.search(searchQuery.value).map(result => result.item)
  })

  // Methods
  const playSong = (song: MusicItem) => {
    emit('play-song', song)
  }

  const handleToggleFavorite = (song: MusicItem) => {
    emit('toggle-favorite', song)
  }
</script>
