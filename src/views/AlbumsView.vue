<script setup lang="ts">
  import { computed, ref, watch, onMounted } from 'vue'
  import { useRouter } from 'vue-router'
  import { Song, Album } from '@/bindings'
  import { Button } from '@/components/ui/button'
  import { Input } from '@/components/ui/input'
  import { Play } from 'lucide-vue-next'
  import Fuse from 'fuse.js'
  import ImagePlaceholder from '@/components/shared/ImagePlaceholder.vue'
  import ImageLoader from '@/components/shared/ImageLoader.vue'
  import { useTauri } from '@/composables/useTauri'

  const router = useRouter()
  const { getMusicLibrary } = useTauri()

  const props = defineProps<{
    serverUrl: string,
    token:     string,
  }>()

  const emit = defineEmits<{
    'play-songs':   [songs: Song[]]
    'select-album': [album: Album]
  }>()

  const searchQuery = ref('')
  const allSongs = ref<Song[]>([])

  onMounted(async () => {
    allSongs.value = await getMusicLibrary(props.serverUrl, props.token)
  })

  const albumsWithSongs = computed(() => {
    const albumsMap = new Map<string, Album>()

    allSongs.value.forEach(song => {
      if (song.album && song.albumId) {
        if (!albumsMap.has(song.albumId)) {
          albumsMap.set(song.albumId, {
            id:          song.albumId,
            name:        song.album,
            artist:      song.artists?.[0] || 'Unknown Artist',
            artistId:    song.artistIds?.[0] || null,
            albumArtUrl: song.albumArtUrl,
            songCount:   0,
            songs:       [],
          })
        }
        const album = albumsMap.get(song.albumId)!
        album.songs!.push(song)
        album.songCount = album.songs!.length
      }
    })

    return Array.from(albumsMap.values())
  })

  // Fuzzy search setup (Fuse.js)
  const albumsFuse = ref(new Fuse(albumsWithSongs.value, {
    keys: [
      { name: 'name', weight: 0.6 },
      { name: 'artist', weight: 0.4 },
    ],
    includeScore:       true,
    threshold:          0.2,
    minMatchCharLength: 2,
  }))

  watch(albumsWithSongs, newAlbums => {
    albumsFuse.value.setCollection(newAlbums)
  })

  const filteredAlbums = computed(() => {
    if (!searchQuery.value || searchQuery.value.length < 2) return albumsWithSongs.value
    return albumsFuse.value.search(searchQuery.value).map(result => result.item)
  })

  const playAlbum = (album: Album) => {
    if (album.songs && album.songs.length > 0) {
      emit('play-songs', album.songs)
    }
  }

  const selectAlbum = (album: Album) => {
    router.push(`/songs/album/${encodeURIComponent(album.name)}`)
  }
</script>

<template>
  <div class='p-8 max-w-7xl mx-auto'>
    <div class='mb-8'>
      <h1 class='text-4xl font-bold mb-4'>
        Albums
      </h1>
      <Input
        v-model='searchQuery'
        class='max-w-sm focus-visible:ring-1 focus-visible:ring-accent border-0 focus-visible:border-accent'
        placeholder='Search albums...'
        type='text'
      />
    </div>

    <div class='grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-6'>
      <div
        v-for='album in filteredAlbums'
        @click='selectAlbum(album)'
        :key='album.name'
        class='cursor-pointer group'
      >
        <div class='relative mb-4'>
          <ImageLoader
            :alt='`${album.name} album art`'
            :item-id='album.id || album.name'
            :server-url='serverUrl'
            :token='token'
            class='
              w-full aspect-square rounded-lg object-cover shadow-lg
              group-hover:opacity-75 transition-opacity
            '
          >
            <template #fallback>
              <ImagePlaceholder
                class='w-full aspect-square shadow-lg group-hover:opacity-75 transition-opacity'
                size='large'
                type='album'
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
              @click.stop='playAlbum(album)'
              class='
                bg-white/20 hover:bg-white/30 backdrop-blur-sm text-white border
                border-white/20
              '
              size='icon'
            >
              <Play class='h-4 w-4' />
            </Button>
          </div>
        </div>

        <div>
          <p class='font-semibold truncate'>
            {{ album.name }}
          </p>
          <p class='text-sm text-muted-foreground truncate'>
            {{ album.artist }}
          </p>
          <p v-if='album.songs' class='text-sm text-muted-foreground truncate'>
            {{ album.songs.length }} songs
          </p>
        </div>
      </div>
    </div>

    <div v-if='filteredAlbums.length === 0' class='text-center py-12'>
      <p class='text-muted-foreground'>
        No albums found
      </p>
    </div>
  </div>
</template>
