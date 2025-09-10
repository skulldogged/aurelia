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

        <div v-if='loading' class='flex justify-center items-center py-12'>
          <div class='text-muted-foreground'>
            Loading songs...
          </div>
        </div>
        <div v-else-if='error' class='text-center py-12'>
          <p class='text-destructive mb-4'>
            {{ error }}
          </p>
          <Button @click='fetchMusicLibrary' variant='destructive'>
            Try Again
          </Button>
        </div>
        <div v-else>
          <SongList
            @play-song='playSong'
            @toggle-favorite='handleToggleFavorite'
            :current-song='props.currentSong'
            :is-playing='props.isPlaying'
            :server-url='props.serverUrl'
            :show-album='true'
            :show-album-art='true'
            :show-artist='true'
            :show-duration='true'
            :show-track-number='true'
            :show-year='true'
            :songs='filteredSongs'
            :token='props.token'
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
  import { ref, computed, watch, onMounted } from 'vue'
  import Fuse from 'fuse.js'
  import SongList from '@/components/shared/SongList.vue'
  import { Button } from '@/components/ui/button'
  import { Input } from '@/components/ui/input'
  import { Song } from '@/bindings'
  import { useTauri } from '@/composables/useTauri'

  // Define props from parent
  const props = defineProps<{
    currentSong: Song | null
    isPlaying:   boolean
    serverUrl:   string
    token:       string
  }>()

  // Define emits for parent
  const emit = defineEmits<{
    'play-song':       [song: Song]
    'toggle-favorite': [song: Song]
  }>()

  // Component state
  const searchQuery = ref('')
  const songs = ref<Song[]>([])
  const loading = ref(false)
  const error = ref('')
  const { getMusicLibrary } = useTauri()

  const fetchMusicLibrary = async () => {
    loading.value = true
    error.value = ''
    try {
      songs.value = await getMusicLibrary(props.serverUrl, props.token)
    } catch (e) {
      error.value = e as string
    } finally {
      loading.value = false
    }
  }

  onMounted(fetchMusicLibrary)

  // Fuzzy Search setup
  const songFuse = ref(new Fuse(songs.value, {
    keys: [
      { name: 'name', weight: 0.5 },
      { name: 'artists', weight: 0.3 },
      { name: 'album', weight: 0.2 },
    ],
    includeScore:       true,
    threshold:          0.2,
    minMatchCharLength: 2,
  }))

  watch(songs, newSongs => songFuse.value.setCollection(newSongs))

  // Computed properties for filtering
  const filteredSongs = computed(() => {
    if (!searchQuery.value || searchQuery.value.length < 2) return songs.value
    return songFuse.value.search(searchQuery.value).map(result => result.item)
  })

  // Methods
  const playSong = (song: Song) => {
    emit('play-song', song)
  }

  const handleToggleFavorite = (song: Song) => {
    emit('toggle-favorite', song)
  }
</script>
