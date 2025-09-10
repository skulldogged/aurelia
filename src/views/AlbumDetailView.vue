<script setup lang="ts">
  import { computed, ref, onMounted } from 'vue'
  import { useRoute } from 'vue-router'
  import SongList from '@/components/shared/SongList.vue'
  import ImageLoader from '@/components/shared/ImageLoader.vue'
  import { Song, Album } from '@/bindings'
  import { useTauri } from '@/composables/useTauri'

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
  const { getMusicLibrary } = useTauri()
  const album = ref<Album | null>(null)
  const allSongs = ref<Song[]>([])

  onMounted(async () => {
    console.log('AlbumDetailView: onMounted')
    console.log('AlbumDetailView: serverUrl:', props.serverUrl)
    console.log('AlbumDetailView: token present:', !!props.token)

    if (!props.serverUrl || !props.token) {
      console.error('AlbumDetailView: Missing serverUrl or token props')
      return
    }

    try {
      const albumName = decodeURIComponent(route.params.albumName as string)
      console.log('AlbumDetailView: Fetching album:', albumName)

      const songs = await getMusicLibrary(props.serverUrl, props.token)
      allSongs.value = songs
      console.log('AlbumDetailView: Fetched', songs.length, 'total songs')

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
          const currentAlbum = albumsMap.get(song.albumId)!
          currentAlbum.songs!.push(song)
          currentAlbum.songCount = currentAlbum.songs!.length
        }
      })

      const allAlbums = Array.from(albumsMap.values())
      album.value = allAlbums.find(a => a.name === albumName) || null
      console.log('AlbumDetailView: Found album:', album.value?.name || 'null')
      console.log('AlbumDetailView: Album art URL:', album.value?.albumArtUrl || 'null')
    } catch (error) {
      console.error('AlbumDetailView: Error fetching albums:', error)
    }
  })

  const albumSongs = computed(() => {
    if (!album.value) return []
    return [...album.value.songs || []].sort((a, b) => (a.trackNumber ?? 0) - (b.trackNumber ?? 0))
  })

  const displayedArtist = computed(() => {
    // Find the most common artist for this album's songs
    if (!albumSongs.value.length) return album.value?.artist // Fallback

    const artistCounts = new Map<string, number>()
    albumSongs.value.forEach(song => {
      song.artists?.forEach(artist => {
        artistCounts.set(artist, (artistCounts.get(artist) || 0) + 1)
      })
    })

    if (artistCounts.size === 0) return album.value?.artist // Fallback

    // Get the artist with the highest count
    let maxCount = 0
    let primaryArtist = ''
    for (const [artist, count] of artistCounts.entries()) {
      if (count > maxCount) {
        maxCount = count
        primaryArtist = artist
      }
    }
    return primaryArtist
  })
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
          {{ displayedArtist }}
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
