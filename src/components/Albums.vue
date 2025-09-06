<script setup lang="ts">
  import { computed, ref, watch } from 'vue'
  import { useRouter } from 'vue-router'
  import { MusicItem, AlbumInfo, AlbumWithSongs } from '@/types'
  import { Button } from '@/components/ui/button'
  import { Input } from '@/components/ui/input'
  import { Play } from 'lucide-vue-next'
  import Fuse from 'fuse.js'
  import { useImageCache } from '@/composables/useImageCache'

  const router = useRouter()

  const props = defineProps<{
    songs:  MusicItem[]
    albums: AlbumInfo[]
  }>()

  const emit = defineEmits<{
    'play-songs':   [songs: MusicItem[]]
    'select-album': [album: AlbumInfo]
  }>()

  const searchQuery = ref('')

  // Compute albums with songs
  const albumsWithSongs = computed((): AlbumWithSongs[] => {
    const albumMap = new Map<string, AlbumWithSongs>()

    props.songs.forEach(song => {
      const albumName = song.album || 'Unknown Album'
      const primaryArtistName = song.artists?.[0] || 'Unknown Artist'
      const primaryArtistId = song.artistIds?.[0]

      if (!albumMap.has(albumName)) {
        albumMap.set(albumName, {
          name:        albumName,
          artist:      primaryArtistName,
          artistId:    primaryArtistId,
          songCount:   0,
          albumArtUrl: song.albumArtUrl,
          songs:       [],
        })
      }
      const album = albumMap.get(albumName)
      if (album) {
        album.songs.push(song)
        album.songCount++
      }
    })

    return Array.from(albumMap.values())
      .sort((a, b) => a.name.localeCompare(b.name))
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

  const albumArtUrls = computed(() => {
    return filteredAlbums.value
      .map(album => album.albumArtUrl)
      .filter(url => !!url) as string[]
  })

  const { cachedUrls } = useImageCache(() => albumArtUrls.value)

  const playAlbum = (album: AlbumWithSongs) => {
    const albumSongs = album.songs
      .sort((a: MusicItem, b: MusicItem) => (a.trackNumber || 0) - (b.trackNumber || 0))

    if (albumSongs.length > 0) {
      emit('play-songs', albumSongs)
    }
  }

  const selectAlbum = (album: AlbumWithSongs) => {
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
        class='max-w-sm'
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
          <img
            v-if='album.albumArtUrl'
            :alt='`${album.name} album art`'
            :src='cachedUrls[album.albumArtUrl] || album.albumArtUrl'
            class='
              w-full aspect-square rounded-lg object-cover shadow-lg
              group-hover:opacity-75 transition-opacity
            '
          >
          <div
            v-else
            class='
              w-full aspect-square bg-muted rounded-lg flex items-center
              justify-center shadow-lg group-hover:opacity-75 transition-opacity
            '
          >
            <span class='text-4xl'>💿</span>
          </div>

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
          <h3 class='font-semibold truncate'>
            {{ album.name }}
          </h3>
          <p class='text-sm text-muted-foreground truncate'>
            <router-link
              @click.stop
              v-if='album.artistId'
              :to="{ name: 'artist-detail', params: { artistId: album.artistId } }"
              class='hover:underline'
            >
              {{ album.artist }}
            </router-link>
            <span v-else>{{ album.artist }}</span>
          </p>
          <p class='text-xs text-muted-foreground'>
            {{ album.songCount }} songs
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
