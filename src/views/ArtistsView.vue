<script setup lang="ts">
  import { computed, ref, watch } from 'vue'
  import { useRouter } from 'vue-router'
  import { MusicItem, ArtistWithSongs } from '@/types'
  import { Button } from '@/components/ui/button'
  import { Input } from '@/components/ui/input'
  import { Shuffle } from 'lucide-vue-next'
  import Fuse from 'fuse.js'
  import ImagePlaceholder from '@/components/shared/ImagePlaceholder.vue'

  const router = useRouter()
  const showAllArtists = ref(false)

  const props = defineProps<{
    artists:      ArtistWithSongs[]
    albumArtists: ArtistWithSongs[]
  }>()

  const emit = defineEmits<{
    'play-song':     [song: MusicItem]
    'play-songs':    [songs: MusicItem[]]
    'select-artist': [artist: ArtistWithSongs]
  }>()

  const searchQuery = ref('')
  const artistsToDisplay = computed(() => showAllArtists.value ? props.artists : props.albumArtists || [])

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

  const playArtistShuffle = (artist: ArtistWithSongs) => {
    const artistSongs = artist.songs
    if (artistSongs.length > 0) {
      // Shuffle the songs
      const shuffledSongs = [...artistSongs].sort(() => 0.5 - Math.random())
      emit('play-songs', shuffledSongs)
    }
  }

  const selectArtist = (artist: ArtistWithSongs) => {
    if (artist.id)
      router.push(`/songs/artist/${artist.id}`)
  }
</script>

<template>
  <div class='p-8 max-w-7xl mx-auto'>
    <div class='mb-8 flex justify-between items-center'>
      <div>
        <h1 class='text-4xl font-bold mb-4'>
          Artists
        </h1>
        <Input
          v-model='searchQuery'
          class='max-w-sm focus-visible:ring-1 focus-visible:ring-accent border-0 focus-visible:border-accent'
          placeholder='Search artists...'
          type='text'
        />
      </div>
      <Button @click='showAllArtists = !showAllArtists'>
        {{ showAllArtists ? "Show Album Artists" : "Show All Artists" }}
      </Button>
    </div>

    <div class='grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-6'>
      <div
        v-for='artist in filteredArtists'
        @click='selectArtist(artist)'
        :key='artist.name'
        class='cursor-pointer group'
      >
        <div class='relative mb-4'>
          <img
            v-if='artist.imageUrl'
            :alt='`${artist.name} artist image`'
            :src='artist.imageUrl'
            class='w-full aspect-square rounded-lg object-cover shadow-lg group-hover:opacity-75 transition-opacity'
          >
          <ImagePlaceholder
            v-else
            class='w-full aspect-square shadow-lg group-hover:opacity-75 transition-opacity'
            size='large'
            type='artist'
          />

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
          <h3 class='font-semibold truncate'>
            {{ artist.name }}
          </h3>
          <p class='text-sm text-muted-foreground'>
            {{ artist.songCount }} songs
          </p>
        </div>
      </div>
    </div>

    <div v-if='filteredArtists && filteredArtists.length === 0' class='text-center py-12'>
      <p class='text-muted-foreground'>
        No artists found
      </p>
    </div>
  </div>
</template>
