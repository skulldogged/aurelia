<script setup lang="ts">
  import { computed, ref, watch, onMounted } from 'vue'
  import { useRouter } from 'vue-router'
  import { Song, Artist } from '@/bindings'
  import { Button } from '@/components/ui/button'
  import { Input } from '@/components/ui/input'
  import { Skeleton } from '@/components/ui/skeleton'
  import { Shuffle } from 'lucide-vue-next'
  import Fuse from 'fuse.js'
  import ImagePlaceholder from '@/components/shared/ImagePlaceholder.vue'
  import ImageLoader from '@/components/shared/ImageLoader.vue'
  import { useTauri } from '@/composables/useTauri'

  const router = useRouter()
  const { getArtistsWithSongs } = useTauri()

  const showAllArtists = ref(false)

  const props = defineProps<{
    serverUrl: string,
    token:     string,
  }>()

  const emit = defineEmits<{
    'play-song':     [song: Song]
    'play-songs':    [songs: Song[]]
    'select-artist': [artist: Artist]
  }>()

  const searchQuery = ref('')
  const artists = ref<Artist[]>([])
  const albumArtists = ref<Artist[]>([])
  const isLoading = ref(true)
  const showSkeleton = ref(false) // Temporary dev toggle for adjusting skeleton sizes

  onMounted(async () => {
    try {
      const [all, albumOnly] = await Promise.all([
        getArtistsWithSongs(props.serverUrl, props.token, false),
        getArtistsWithSongs(props.serverUrl, props.token, true),
      ])
      artists.value = all
      albumArtists.value = albumOnly
    } catch (error) {
      console.error('Failed to load artists:', error)
    } finally {
      isLoading.value = false
    }
  })

  const artistsToDisplay = computed(() => showAllArtists.value ? artists.value : albumArtists.value || [])

  // Use artists directly from props
  const artistsWithSongs = computed(() => artistsToDisplay.value)

  // Fuzzy search setup (Fuse.js)
  const artistsFuse = ref(new Fuse(artistsWithSongs.value, {
    keys:               ['name'],
    includeScore:       true,
    threshold:          0.2,
    minMatchCharLength: 2,
  }))

  watch(artistsWithSongs, newArtists => {
    artistsFuse.value.setCollection(newArtists)
  })

  const filteredArtists = computed(() => {
    if (!searchQuery.value || searchQuery.value.length < 2) return artistsWithSongs.value
    return artistsFuse.value.search(searchQuery.value).map(result => result.item)
  })

  const playArtistShuffle = (artist: Artist) => {
    const artistSongs = artist.songs
    if (artistSongs && artistSongs.length > 0) {
      // Shuffle the songs
      const shuffledSongs = [...artistSongs].sort(() => 0.5 - Math.random())
      emit('play-songs', shuffledSongs)
    }
  }

  const selectArtist = (artist: Artist) => {
    if (artist.id)
      router.push(`/songs/artist/${artist.id}`)
  }
</script>

<template>
  <div class='p-8 max-w-7xl mx-auto'>
    <div class='mb-8'>
      <div class='flex justify-between items-start mb-4'>
        <h1 class='text-4xl font-bold'>
          Artists
        </h1>
        <Button @click='showAllArtists = !showAllArtists'>
          {{ showAllArtists ? "Show Album Artists" : "Show All Artists" }}
        </Button>
      </div>
      <div class='flex flex-col sm:flex-row gap-4 items-start sm:items-center'>
        <Input
          v-model='searchQuery'
          class='max-w-sm focus-visible:ring-1 focus-visible:ring-accent border-0 focus-visible:border-accent'
          placeholder='Search artists...'
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

    <div v-if='isLoading || showSkeleton' class='grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-6'>
      <!-- Skeleton loading grid -->
      <div
        v-for='n in 20'
        :key='`skeleton-${n}`'
        class='flex flex-col gap-4'
      >
        <!-- Artist image skeleton -->
        <Skeleton class='w-full aspect-square rounded-lg' />
        <!-- Text content skeleton -->
        <div class='flex flex-col items-center gap-1'>
          <!-- Artist name skeleton -->
          <Skeleton class='h-6 w-3/4' />
          <!-- Song count skeleton -->
          <Skeleton class='h-4 w-1/2' />
        </div>
      </div>
    </div>
    <div v-else class='grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-6'>
      <div
        v-for='artist in filteredArtists'
        @click='selectArtist(artist)'
        :key='artist.name'
        class='cursor-pointer group'
      >
        <div class='relative mb-4'>
          <ImageLoader
            :alt='`${artist.name} artist image`'
            :item-id='artist.id'
            :server-url='serverUrl'
            :token='token'
            class='w-full aspect-square rounded-lg object-cover shadow-lg group-hover:opacity-75 transition-opacity'
          >
            <template #fallback>
              <ImagePlaceholder
                class='w-full aspect-square shadow-lg group-hover:opacity-75 transition-opacity'
                size='large'
                type='artist'
              />
            </template>
          </ImageLoader>

          <!-- Play button overlay -->
          <div
            class='
              absolute inset-0 bg-black/50 rounded-lg opacity-0
              group-hover:opacity-100 transition-opacity flex items-center
              justify-center
            '
          >
            <Button
              @click.stop='playArtistShuffle(artist)'
              class='
                bg-white/20 hover:bg-white/30 backdrop-blur-sm text-white
                border border-white/20
              '
              size='icon'
            >
              <Shuffle class='h-4 w-4' />
            </Button>
          </div>
        </div>

        <div class='text-center'>
          <p class='font-semibold truncate'>
            {{ artist.name }}
          </p>
          <p v-if='artist.songs' class='text-sm text-muted-foreground truncate'>
            {{ artist.songs.length }} songs
          </p>
        </div>
      </div>
    </div>

    <div
      v-if='!isLoading && !showSkeleton && filteredArtists && filteredArtists.length === 0'
      class='text-center py-12'
    >
      <p class='text-muted-foreground'>
        No artists found
      </p>
    </div>
  </div>
</template>
