<script setup lang="ts">
  import Fuse from 'fuse.js'
  import { Shuffle } from 'lucide-vue-next'
  import { computed, ref } from 'vue'
  import { useRouter } from 'vue-router'

  import { Artist, Song } from '@/bindings'
  import ImageLoader from '@/components/shared/ImageLoader.vue'
  import ImagePlaceholder from '@/components/shared/ImagePlaceholder.vue'
  import Button from '@/components/ui/Button.vue'
  import { Input } from '@/components/ui/input'
  import { Skeleton } from '@/components/ui/skeleton'
  import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs'
  import { useAuthStore } from '@/stores/auth'
  import { useLibraryStore } from '@/stores/library'

  const router = useRouter()

  const artistMode = ref<'album' | 'all'>('album')

  const emit = defineEmits<{
    'play-song':     [song: Song]
    'play-songs':    [songs: Song[]]
    'select-artist': [artist: Artist]
  }>()

  const authStore = useAuthStore()
  const libraryStore = useLibraryStore()

  // Create computed properties from stores
  const allArtists = computed(() => libraryStore.allArtistsWithSongs as Artist[])
  const libraryLoading = computed(() => libraryStore.isLoading)
  const serverUrl = computed(() => authStore.serverUrl)
  const token = computed(() => authStore.token)

  const searchQuery = ref('')

  // Artists who appear as an "album artist" on at least one song
  const albumArtists = computed(() => allArtists.value.filter(artist =>
    artist.songs?.some(song =>
      song.albumArtists?.some(albumArtist => albumArtist.id === artist.id),
    ),
  ))

  const artistsToDisplay = computed(() => {
    const mode = artistMode.value
    return mode === 'all' ? allArtists.value : (albumArtists.value?.length ? albumArtists.value : allArtists.value)
  })

  // Deduplicate artists by name (not ID) to handle Jellyfin duplicate artist entries
  // For duplicates, keep the entry with the most songs
  const artistsWithSongs = computed(() => {
    const uniqueArtistsByName = new Map<string, Artist>()

    for (const artist of artistsToDisplay.value) {
      const normalizedName = artist.name.toLowerCase()
      const existing = uniqueArtistsByName.get(normalizedName)

      // Keep the artist with more songs, or the first one if equal
      if (!existing || (artist.songs?.length || 0) > (existing.songs?.length || 0)) {
        uniqueArtistsByName.set(normalizedName, artist)
      }
    }

    return Array.from(uniqueArtistsByName.values()).sort((a, b) =>
      a.name.toLowerCase().localeCompare(b.name.toLowerCase()),
    )
  })

  // Fuzzy search setup (Fuse.js)
  // Recreate the Fuse instance when artists change to avoid duplication issues
  const artistsFuse = computed(() => new Fuse(artistsWithSongs.value, {
    includeScore:       true,
    keys:               ['name'],
    minMatchCharLength: 2,
    threshold:          0.2,
  }))

  const filteredArtists = computed(() =>
    searchQuery.value && searchQuery.value.length >= 2
      ? artistsFuse.value.search(searchQuery.value).map(result => result.item)
      : artistsWithSongs.value,
  )

  const playArtistShuffle = (artist: Artist): void => {
    const artistSongs = artist.songs

    if (artistSongs && artistSongs.length > 0)
      emit('play-songs', [...artistSongs].sort(() => 0.5 - Math.random()))
  }

  const selectArtist = (artist: Artist): void => {
    if (artist.id)
      router.push(`/artists/${artist.id}`)
  }
</script>

<template>
  <div class='px-4 pb-4' style='padding-top: env(safe-area-inset-top)'>
    <div class='mb-6'>
      <h1 class='text-3xl font-bold mb-4'>
        Artists
      </h1>
      <Input
        v-model='searchQuery'
        class='w-full focus-visible:ring-1 focus-visible:ring-accent border-0 focus-visible:border-accent mb-4'
        placeholder='Search artists...'
        type='text'
      />

      <!-- Artist Mode Tabs -->
      <Tabs v-model='artistMode'>
        <TabsList>
          <TabsTrigger value='album'>
            Album Artists
          </TabsTrigger>
          <TabsTrigger value='all'>
            All Artists
          </TabsTrigger>
        </TabsList>
      </Tabs>
    </div>

    <div v-if='libraryLoading' class='grid grid-cols-2 gap-4'>
      <!-- Skeleton loading grid -->
      <div
        v-for='n in 10'
        :key='`skeleton-${n}`'
        class='flex flex-col gap-3'
      >
        <!-- Artist image skeleton -->
        <Skeleton class='w-full aspect-square rounded-lg' />
        <!-- Text content skeleton -->
        <div class='flex flex-col items-center gap-1'>
          <!-- Artist name skeleton -->
          <Skeleton class='h-5 w-3/4' />
          <!-- Song count skeleton -->
          <Skeleton class='h-4 w-1/2' />
        </div>
      </div>
    </div>

    <div v-else class='grid grid-cols-2 gap-4'>
      <div
        v-for='artist in filteredArtists'
        @click='selectArtist(artist)'
        :key='artist.id'
        class='cursor-pointer group'
      >
        <div class='relative mb-3'>
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
          <p class='font-medium truncate text-sm'>
            {{ artist.name }}
          </p>
          <p
            v-if='artist.songs'
            class='text-xs text-muted-foreground truncate'
          >
            {{ artist.songs.length }} songs
          </p>
        </div>
      </div>
    </div>

    <div
      v-if='!libraryLoading && filteredArtists && filteredArtists.length === 0'
      class='text-center py-12'
    >
      <p class='text-muted-foreground'>
        No artists found
      </p>
    </div>
  </div>
</template>