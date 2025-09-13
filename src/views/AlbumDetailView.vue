<script setup lang="ts">
  import { computed, ref, onMounted } from 'vue'
  import { useRoute } from 'vue-router'
  import SongList from '@/components/shared/SongList.vue'
  import ImageLoader from '@/components/shared/ImageLoader.vue'
  import { Song, Album, commands } from '@/bindings'
  import { uiLogger } from '@/lib/logger'

  const props = defineProps<{
    currentSong: Song | null
    isPlaying:   boolean
    serverUrl:   string
    token:       string
  }>()

  defineEmits<{
    'play-song':       [song: Song]
    'toggle-favorite': [song: Song]
  }>()

  const route = useRoute()
  const { getSongs } = commands
  const album = ref<Album | null>(null)
  const allSongs = ref<Song[]>([])

  onMounted(async () => {
    if (!props.serverUrl || !props.token) {
      uiLogger.error('Missing serverUrl or token props')
      return
    }

    try {
      const albumName = decodeURIComponent(route.params.albumName as string)

      const songsResult = await getSongs(props.serverUrl, props.token, null, null, null, null)
      if (songsResult.status === 'error') {
        uiLogger.error('Failed to fetch songs:', songsResult.error)
        throw new Error(songsResult.error)
      }
      allSongs.value = songsResult.data

      const albumsMap = new Map<string, Album>()
      allSongs.value.forEach(song => {
        if (song.album && song.albumId) {
          if (!albumsMap.has(song.albumId)) {
            albumsMap.set(song.albumId, {
              id:          song.albumId,
              name:        song.album,
              artist:      song.albumArtists?.[0]?.name || song.artists?.[0] || 'Unknown Artist',
              artistId:    song.albumArtists?.[0]?.id || song.artistIds?.[0] || null,
              albumArtUrl: song.albumArtUrl,
              songCount:   0,
              songs:       [],
            })
          }
          const currentAlbum = albumsMap.get(song.albumId)!
          currentAlbum.songs!.push(song)
          currentAlbum.songCount = currentAlbum.songs!.length
        }
      })

      const allAlbums = Array.from(albumsMap.values())
      album.value = allAlbums.find(a => a.name === albumName) || null
    } catch (error) {
      uiLogger.error('Error fetching albums:', error)
    }
  })

  const albumSongs = computed(() => {
    if (!album.value) return []
    return [...album.value.songs || []].sort((a, b) => (a.trackNumber ?? 0) - (b.trackNumber ?? 0))
  })

  // Determine if any song in the album has multiple artists
  const hasMultipleArtists =
    computed(() =>
      albumSongs.value.length > 1 &&
      albumSongs.value.some(song =>
        song.artists?.length &&
        song.artists.length > 1,
      ),
    )
</script>

<template>
  <div v-if='album' class='space-y-8 max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8'>
    <!-- Header -->
    <div class='flex items-center space-x-6'>
      <ImageLoader
        :item-id='album.id || album.name'
        :server-url='serverUrl'
        :token='token'
        alt='Album art'
        class='w-32 h-32 rounded-md'
      />
      <div>
        <h1 class='text-5xl font-bold text-foreground select-text'>
          {{ album.name }}
        </h1>
        <p class='text-2xl text-muted-foreground mt-2 select-text'>
          {{ album?.artist || 'Unknown Artist' }}
        </p>
      </div>
    </div>

    <!-- Songs -->
    <div>
      <h2 class='text-2xl font-semibold text-foreground mb-4'>
        Songs
      </h2>
      <SongList
        @play-song="(song) => $emit('play-song', song)"
        @toggle-favorite="(song) => $emit('toggle-favorite', song)"
        :current-song='props.currentSong'
        :is-playing='props.isPlaying'
        :server-url='props.serverUrl'
        :show-album-art='false'
        :show-artist='hasMultipleArtists'
        :show-duration='true'
        :show-track-number='true'
        :songs='albumSongs'
        :token='props.token'
      />
    </div>
  </div>
  <div v-else class='text-center py-12 text-muted-foreground'>
    Album not found.
  </div>
</template>
