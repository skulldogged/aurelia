<script setup lang="ts">
  import { computed, ref, watch } from 'vue'
  import { useRouter } from 'vue-router'
  import { MusicItem, AlbumInfo } from '@/types'
  import Carousel from '@/components/shared/Carousel.vue'
  import { Button } from '@/components/ui/button'
  import { ChevronLeft, ChevronRight, Play } from 'lucide-vue-next'

  const router = useRouter()

  const props = defineProps<{
    songs:  MusicItem[],
    albums: AlbumInfo[]
  }>()

  const emit = defineEmits<{
    'play-songs':   [songs: MusicItem[]],
    'select-album': [album: AlbumInfo]
  }>()

  const mostPlayed = computed(() => {
    return [...props.songs]
      .sort((a, b) => (b.playCount || 0) - (a.playCount || 0))
      .slice(0, 10)
  })

  const recentlyPlayed = computed(() => {
    return [...props.songs]
      .filter(s => s.datePlayed)
      .sort((a, b) => new Date(b.datePlayed!).getTime() - new Date(a.datePlayed!).getTime())
      .slice(0, 10)
  })

  const recentlyAdded = computed(() => {
    // Get albums that were recently added to the user's Jellyfin server
    // Sort by the most recently added song in each album
    return props.albums
      .filter(album => {
        // Find songs from this album
        const albumSongs = props.songs.filter(s => s.album === album.name)
        return albumSongs.length > 0
      })
      .sort((a, b) => {
        // Sort by the most recent dateCreated of songs in each album
        const albumASongs = props.songs.filter(s => s.album === a.name)
        const albumBSongs = props.songs.filter(s => s.album === b.name)

        const getMostRecentDateCreated = (songs: typeof props.songs) => {
          const withDates = songs.filter(s => s.dateCreated)
          if (withDates.length > 0) {
            return Math.max(...withDates.map(s => new Date(s.dateCreated!).getTime()))
          }
          return 0 // Fallback for albums without dateCreated data
        }

        const dateA = getMostRecentDateCreated(albumASongs)
        const dateB = getMostRecentDateCreated(albumBSongs)

        if (dateA === dateB) {
          // If dates are equal or both 0, sort alphabetically
          return a.name.localeCompare(b.name)
        }

        return dateB - dateA
      })
      .slice(0, 10)
  })

  const randomAlbums = computed(() => {
    return [...props.albums].sort(() => 0.5 - Math.random()).slice(0, 10)
  })

  const featuredAlbums = ref<AlbumInfo[]>([])
  const currentFeaturedIndex = ref(0)

  const featuredAlbum = computed(() => {
    return featuredAlbums.value[currentFeaturedIndex.value] || null
  })

  // Initialize featured albums with a randomized list
  const initializeFeaturedAlbums = () => {
    featuredAlbums.value = [...props.albums].sort(() => 0.5 - Math.random())
  }

  const nextFeaturedAlbum = () => {
    if (featuredAlbums.value.length > 1) {
      currentFeaturedIndex.value = (currentFeaturedIndex.value + 1) % featuredAlbums.value.length
    }
  }

  const prevFeaturedAlbum = () => {
    if (featuredAlbums.value.length > 1) {
      currentFeaturedIndex.value = currentFeaturedIndex.value === 0
        ? featuredAlbums.value.length - 1
        : currentFeaturedIndex.value - 1
    }
  }

  // Watch for changes in albums and reinitialize
  watch(() => props.albums, () => {
    initializeFeaturedAlbums()
  }, { immediate: true })

  const playSongs = (songs: MusicItem[], startWith?: MusicItem) => {
    if (startWith) {
      const startIndex = songs.indexOf(startWith)
      if (startIndex === -1) {
        emit('play-songs', songs)
        return
      }
      const reorderedSongs = [...songs.slice(startIndex), ...songs.slice(0, startIndex)]
      emit('play-songs', reorderedSongs)
    } else {
      emit('play-songs', songs)
    }
  }

  const playFeaturedAlbum = () => {
    if (!featuredAlbum.value) return

    // Get all songs from the featured album
    const albumSongs = props.songs
      .filter(song => song.album === featuredAlbum.value.name)
      .sort((a, b) => (a.trackNumber || 0) - (b.trackNumber || 0))

    if (albumSongs.length > 0) {
      // Play the songs and navigate to the album
      emit('play-songs', albumSongs)
      router.push(`/songs/album/${encodeURIComponent(featuredAlbum.value.name)}`)
    }
  }

  const playAlbumSongs = (album: AlbumInfo) => {
    // Get all songs from the album
    const albumSongs = props.songs
      .filter(song => song.album === album.name)
      .sort((a, b) => (a.trackNumber || 0) - (b.trackNumber || 0))

    if (albumSongs.length > 0) {
      // Play the songs
      emit('play-songs', albumSongs)
    }
  }
</script>

<template>
  <div class='p-8 space-y-12'>
    <!-- Featured Album Section -->
    <div v-if='featuredAlbum' class='relative isolate rounded-2xl p-8 mb-8 overflow-hidden blur-card'>
      <!-- Blurred Background -->
      <div
        v-if='featuredAlbum.albumArtUrl'
        :style='{ backgroundImage: `url(${featuredAlbum.albumArtUrl})` }'
        class='absolute inset-0 bg-cover bg-center bg-no-repeat rounded-2xl blur-md scale-105'
      >
        <div class='absolute inset-0 bg-black/60 rounded-2xl' />
      </div>
      <div v-else class='absolute inset-0 bg-gradient-to-r from-muted/50 to-muted/20 rounded-2xl' />

      <!-- Content -->
      <div class='relative z-10 flex items-center space-x-6'>
        <div class='flex-shrink-0'>
          <img
            v-if='featuredAlbum.albumArtUrl'
            :alt='`${featuredAlbum.name} album art`'
            :src='featuredAlbum.albumArtUrl'
            class='w-48 h-48 rounded-xl shadow-2xl object-cover'
          >
          <div
            v-else
            class='w-48 h-48 bg-muted/80 backdrop-blur-sm rounded-xl shadow-2xl flex items-center justify-center'
          >
            <span class='text-4xl'>🎵</span>
          </div>
        </div>
        <div class='flex-1 min-w-0'>
          <h1 class='text-4xl font-bold mb-2 text-white drop-shadow-lg truncate'>
            <router-link
              :to="{ name: 'album-detail', params: { albumName: featuredAlbum.name } }"
            >
              {{ featuredAlbum.name }}
            </router-link>
          </h1>
          <p class='text-xl text-white/90 mb-4 drop-shadow-md'>
            <router-link
              v-if='featuredAlbum.artistId'
              :to="{ name: 'artist-detail', params: { artistId: featuredAlbum.artistId } }"
            >
              {{ featuredAlbum.artist }}
            </router-link>
            <span v-else>{{ featuredAlbum.artist }}</span>
          </p>
          <p class='text-sm text-white/80 mb-6 drop-shadow-md'>
            {{ featuredAlbum.songCount }} songs
          </p>
          <button
            @click='playFeaturedAlbum'
            class='
              bg-white/20 hover:bg-white/30 backdrop-blur-sm text-white px-8
              py-3 rounded-full font-semibold transition-colors border
              border-white/20
            '
          >
            Play Album
          </button>
        </div>
      </div>

      <!-- Navigation Arrows -->
      <div v-if='featuredAlbums.length > 1' class='absolute bottom-4 right-4 z-20 flex space-x-2'>
        <button
          @click='prevFeaturedAlbum'
          class='
            flex items-center justify-center bg-white/20 p-2 text-white
            backdrop-blur-sm transition-colors hover:bg-white/30
            border border-white/20 rounded-full
          '
        >
          <ChevronLeft class='h-5 w-5' />
        </button>
        <button
          @click='nextFeaturedAlbum'
          class='
            flex items-center justify-center bg-white/20 p-2 text-white
            backdrop-blur-sm transition-colors hover:bg-white/30
            border border-white/20 rounded-full
          '
        >
          <ChevronRight class='h-5 w-5' />
        </button>
      </div>
    </div>

    <Carousel title='Most Played'>
      <div
        v-for='song in mostPlayed'
        @click='playSongs(mostPlayed, song)'
        :key='song.id'
        class='cursor-pointer group'
      >
        <div class='relative mb-2'>
          <img
            v-if='song.albumArtUrl'
            :src='song.albumArtUrl'
            alt='Album art'
            class='album-art-image'
          >
          <div v-else class='w-full h-48 bg-muted rounded-lg' />

          <!-- Play button overlay -->
          <div
            class='
              absolute inset-0 bg-black/50 rounded-lg opacity-0
              group-hover:opacity-100 transition-opacity flex items-center
              justify-center
            '
          >
            <Button
              @click.stop='playSongs(mostPlayed, song)'
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
        <p class='font-semibold truncate'>
          {{ song.name }}
        </p>
        <p class='text-sm text-muted-foreground truncate'>
          <template v-if='song.artists && song.artistIds && song.artists.length === song.artistIds.length'>
            <template v-for='(artist, index) in song.artists' :key='song.artistIds[index]'>
              <router-link
                @click.stop
                :to="{ name: 'artist-detail', params: { artistId: song.artistIds[index] } }"
                class='hover:underline'
              >
                {{ artist }}
              </router-link>
              <span v-if='index < song.artists.length - 1'>, </span>
            </template>
          </template>
          <template v-else>
            {{ song.artists?.join(', ') }}
          </template>
        </p>
      </div>
    </Carousel>

    <Carousel title='Recently Played'>
      <div
        v-for='song in recentlyPlayed'
        @click='playSongs(recentlyPlayed, song)'
        :key='song.id'
        class='cursor-pointer group'
      >
        <div class='relative mb-2'>
          <img
            v-if='song.albumArtUrl'
            :src='song.albumArtUrl'
            alt='Album art'
            class='album-art-image'
          >
          <div v-else class='w-full h-48 bg-muted rounded-lg' />

          <!-- Play button overlay -->
          <div
            class='
              absolute inset-0 bg-black/50 rounded-lg opacity-0
              group-hover:opacity-100 transition-opacity flex items-center
              justify-center
            '
          >
            <Button
              @click.stop='playSongs(recentlyPlayed, song)'
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
        <p class='font-semibold truncate'>
          {{ song.name }}
        </p>
        <p class='text-sm text-muted-foreground truncate'>
          <template v-if='song.artists && song.artistIds && song.artists.length === song.artistIds.length'>
            <template v-for='(artist, index) in song.artists' :key='song.artistIds[index]'>
              <router-link
                @click.stop
                :to="{ name: 'artist-detail', params: { artistId: song.artistIds[index] } }"
                class='hover:underline'
              >
                {{ artist }}
              </router-link>
              <span v-if='index < song.artists.length - 1'>, </span>
            </template>
          </template>
          <template v-else>
            {{ song.artists?.join(', ') }}
          </template>
        </p>
      </div>
    </Carousel>

    <Carousel title='Recently Added'>
      <div
        v-for='album in recentlyAdded'
        @click="$emit('select-album', album)"
        :key='album.name'
        class='cursor-pointer group'
      >
        <div class='relative mb-2'>
          <img
            v-if='album.albumArtUrl'
            :src='album.albumArtUrl'
            alt='Album art'
            class='album-art-image'
          >
          <div v-else class='w-full h-48 bg-muted rounded-lg' />

          <!-- Play button overlay -->
          <div
            class='
              absolute inset-0 bg-black/50 rounded-lg opacity-0
              group-hover:opacity-100 transition-opacity flex items-center
              justify-center
            '
          >
            <Button
              @click.stop='playAlbumSongs(album)'
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
        <p class='font-semibold truncate'>
          {{ album.name }}
        </p>
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
      </div>
    </Carousel>

    <Carousel title='From Your Library'>
      <div
        v-for='album in randomAlbums'
        @click="$emit('select-album', album)"
        :key='album.name'
        class='cursor-pointer group'
      >
        <div class='relative mb-2'>
          <img
            v-if='album.albumArtUrl'
            :src='album.albumArtUrl'
            alt='Album art'
            class='album-art-image'
          >
          <div v-else class='w-full h-48 bg-muted rounded-lg' />

          <!-- Play button overlay -->
          <div
            class='
              absolute inset-0 bg-black/50 rounded-lg opacity-0
              group-hover:opacity-100 transition-opacity flex items-center
              justify-center
            '
          >
            <Button
              @click.stop='playAlbumSongs(album)'
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
        <p class='font-semibold truncate'>
          {{ album.name }}
        </p>
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
      </div>
    </Carousel>
  </div>
</template>

<style scoped>
@reference "tailwindcss";

.album-art-image {
  @apply w-full h-auto rounded-lg shadow-lg group-hover:opacity-75 aspect-square object-cover transition-opacity;
}
</style>
