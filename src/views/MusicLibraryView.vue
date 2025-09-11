<template>
  <div class='h-full flex flex-col'>
    <div class='max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8'>
      <div>
        <div class='mb-8'>
          <h1 class='text-4xl font-bold mb-4'>
            Songs
          </h1>
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
        </div>

        <div v-if='loading || showSkeleton' class='rounded-md border'>
          <Table class='table-fixed w-full'>
            <TableHeader>
              <TableRow>
                <TableHead class='w-14' name='album-art-header' />
                <TableHead class='w-14' name='heart-button-header' />
                <TableHead class='w-8 text-right'>
                  #
                </TableHead>
                <TableHead>
                  Title
                </TableHead>
                <TableHead class='w-[20%]'>
                  Artist
                </TableHead>
                <TableHead class='w-[20%]'>
                  Album
                </TableHead>
                <TableHead class='w-20 text-right'>
                  Year
                </TableHead>
                <TableHead class='w-24 text-right'>
                  Plays
                </TableHead>
                <TableHead class='w-24 text-right'>
                  Duration
                </TableHead>
                <TableHead class='w-12' />
              </TableRow>
            </TableHeader>
            <TableBody>
              <TableRow
                v-for='n in 20'
                :key='`skeleton-row-${n}`'
                class='cursor-pointer group hover:bg-sidebar transition-colors'
              >
                <TableCell>
                  <Skeleton :name='`album-art-${n}`' class='w-10 h-10 rounded-md' />
                </TableCell>
                <TableCell class='text-center'>
                  <Button size='icon' variant='ghost' disabled>
                    <Heart class='w-5 h-5 text-muted-foreground' />
                  </Button>
                </TableCell>
                <TableCell class='font-medium text-muted-foreground text-right'>
                  {{ n }}
                </TableCell>
                <TableCell class='font-medium overflow-hidden'>
                  <Skeleton :name='`title-${n}`' class='h-4 w-full' />
                </TableCell>
                <TableCell class='min-w-[150px] overflow-hidden'>
                  <Skeleton :name='`artist-${n}`' class='h-4 w-full' />
                </TableCell>
                <TableCell class='min-w-[150px] overflow-hidden'>
                  <Skeleton :name='`album-${n}`' class='h-4 w-full' />
                </TableCell>
                <TableCell class='hidden md:table-cell text-right'>
                  <Skeleton :name='`year-${n}`' class='h-4 w-9 ml-auto' />
                </TableCell>
                <TableCell class='text-right'>
                  <Skeleton :name='`plays-${n}`' class='h-4 w-4 ml-auto' />
                </TableCell>
                <TableCell class='text-right'>
                  <Skeleton :name='`duration-${n}`' class='h-4 w-8 ml-auto' />
                </TableCell>
                <TableCell />
              </TableRow>
            </TableBody>
          </Table>
        </div>
        <div v-else-if='error && !showSkeleton' class='text-center py-12'>
          <p class='text-destructive mb-4'>
            {{ error }}
          </p>
          <Button @click='fetchMusicLibrary' variant='destructive'>
            Try Again
          </Button>
        </div>
        <div v-else-if='!showSkeleton'>
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
  const showSkeleton = ref(false) // Temporary dev toggle for adjusting skeleton sizes
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
