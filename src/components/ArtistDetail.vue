<template>
  <div>
    <div class="flex items-center mb-4 pt-4">
      <div class="flex-shrink-0">
        <img v-if="artist.imageUrl" :src="artist.imageUrl" alt="Artist art"
          class="w-32 h-32 rounded-full object-cover" />
        <div v-else class="w-32 h-32 rounded-full bg-muted flex items-center justify-center">
          <Music class="w-16 h-16 text-muted-foreground" />
        </div>
      </div>
      <div class="ml-6">
        <h2 class="text-4xl font-bold">{{ artist.name }}</h2>
        <div v-if="artistDetails?.communityRating" class="flex items-center gap-1 mt-2 text-muted-foreground">
          <Star class="w-4 h-4 text-yellow-500" />
          <span>{{ artistDetails.communityRating.toFixed(1) }} / 10</span>
        </div>
        <p v-if="isFeaturedOnlyArtist" class="text-muted-foreground mt-2">
          Featured on {{ featuredSongs.length }} {{ featuredSongs.length === 1 ? 'song' : 'songs' }}
        </p>
        <p v-else class="text-muted-foreground mt-2">
          {{ artistSongs.length }} songs across {{ artistAlbums.length }} albums
        </p>
        <div v-if="artistGenres.length > 0" class="flex flex-wrap gap-2 mt-4">
          <span v-for="genre in artistGenres" :key="genre"
            class="px-2 py-1 text-xs font-semibold rounded-full bg-secondary text-secondary-foreground">
            {{ genre }}
          </span>
        </div>
        <Button @click="$emit('play-artist-shuffle', artist)" class="mt-4">
          <Shuffle class="w-4 h-4 mr-2" />
          Shuffle All
        </Button>
      </div>
    </div>

    <!-- Provider Links -->
    <div v-if="artistDetails?.providerIds" class="flex flex-wrap gap-4 mt-4 mb-4">
      <a v-for="(id, provider) in artistDetails.providerIds" :key="provider"
        :href="`https://www.google.com/search?q=${artist.name}+${provider}`" target="_blank"
        class="text-sm font-medium text-blue-500 hover:underline flex items-center gap-1">
        {{ provider }}
        <ExternalLink class="w-3 h-3" />
      </a>
    </div>

    <!-- Overview -->
    <div v-if="artistDetails?.overview" class="prose dark:prose-invert max-w-none mt-4">
      <p :class="{ 'line-clamp-3': !showFullOverview }" v-html="artistDetails.overview"></p>
      <Button v-if="artistDetails.overview.length > 200" @click="showFullOverview = !showFullOverview" variant="link"
        class="px-0">
        {{ showFullOverview ? 'Show Less' : 'Read More' }}
      </Button>
    </div>

    <div class="md:flex md:space-x-8">
      <!-- Albums -->
      <div v-if="artistAlbums.length > 0" class="md:w-2/3 flex flex-col mb-8 md:mb-0">
        <div class="flex justify-between items-center mb-4">
          <h3 class="text-2xl font-semibold">{{ isFeaturedOnlyArtist ? 'Appears On' : 'Albums' }}</h3>
          <div class="space-x-2 z-10">
            <Button @click="scrollLeft" :disabled="!canScrollLeft" variant="outline" size="icon">
              <ChevronLeft class="h-4 w-4" />
            </Button>
            <Button @click="scrollRight" :disabled="!canScrollRight" variant="outline" size="icon">
              <ChevronRight class="h-4 w-4" />
            </Button>
          </div>
        </div>
        <div class="flex-grow relative carousel-container" :style="{
          '--left-fade-opacity': canScrollLeft ? 1 : 0,
          '--right-fade-opacity': canScrollRight ? 1 : 0,
        }">
          <div class="h-full overflow-x-auto scrollbar-hide" ref="scrollContainer" @scroll="updateScrollButtons">
            <div class="grid grid-rows-1 grid-flow-col auto-cols-[18rem] gap-6 h-full">
              <div v-for="album in artistAlbums" :key="album.name"
                class="bg-card hover:bg-muted/50 rounded-md p-4 cursor-pointer transition-all group h-full flex flex-col">
                <div class="relative" @click="$emit('select-album', album)">
                  <img v-if="album.albumArtUrl" :src="album.albumArtUrl" alt="Album art"
                    class="w-full object-cover rounded-md mb-4 aspect-square" />
                  <div v-else class="w-full rounded-md mb-4 bg-muted aspect-square"></div>
                  <div
                    class="absolute inset-0 flex items-center justify-center transition-opacity opacity-0 group-hover:opacity-100">
                    <Button @click.stop="$emit('play-album', album)" variant="ghost" size="icon"
                      class="w-12 h-12 rounded-full bg-background/75 text-foreground hover:bg-background">
                      <Play class="w-8 h-8" />
                    </Button>
                  </div>
                </div>
                <h3 class="text-foreground font-medium truncate mt-auto">{{ album.name }}</h3>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Top Songs by Artist -->
      <div class="md:w-1/3 flex flex-col">
        <h3 class="text-2xl font-semibold mb-4">{{ isFeaturedOnlyArtist ? 'Features' : 'Top Songs' }}</h3>
        <div class="rounded-md border">
          <div class="p-2 space-y-1">
            <div v-for="song in artistSongs.slice(0, 5)" :key="song.id" @click="playSong(song)"
              class="flex items-center p-2 hover:bg-muted/50 rounded-md cursor-pointer transition-colors group">
              <div class="relative mr-4">
                <img v-if="song.albumArtUrl" :src="song.albumArtUrl" alt="Album art" class="w-10 h-10 rounded-md" />
                <div v-else class="w-10 h-10 rounded-md bg-muted flex-shrink-0"></div>
                <div :class="[
                  'absolute inset-0 w-10 h-10 flex items-center justify-center transition-opacity',
                  currentSong?.id === song.id && isPlaying
                    ? 'opacity-100'
                    : 'opacity-0 group-hover:opacity-100',
                ]">
                  <Button variant="ghost" size="icon"
                    class="w-8 h-8 rounded-full bg-background/75 text-foreground hover:bg-background">
                    <Pause v-if="currentSong?.id === song.id && isPlaying" class="w-5 h-5" />
                    <Play v-else class="w-5 h-5" />
                  </Button>
                </div>
              </div>
              <div class="flex-1 min-w-0">
                <p class="text-foreground font-medium truncate">{{ song.name }}</p>
                <p v-if="isFeaturedOnSong(song)" class="text-muted-foreground text-sm truncate">
                  by {{ song.artists?.[0] }}
                </p>
                <p class="text-muted-foreground text-sm truncate">{{ song.album }}</p>
              </div>
              <p class="text-muted-foreground text-sm">{{ song.playCount ?? 0 }} {{ (song.playCount ?? 0) === 1 ? 'play'
                : 'plays' }}</p>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Related Artists -->
    <div v-if="relatedArtists.length > 0" class="mt-8">
      <h3 class="text-2xl font-semibold mb-4">Related Artists</h3>
      <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-6">
        <div v-for="relatedArtist in relatedArtists" :key="relatedArtist.name"
          @click="$emit('select-artist', relatedArtist)"
          class="flex flex-col items-center text-center bg-card hover:bg-muted/50 rounded-md p-4 cursor-pointer transition-all group space-y-2 border hover:shadow-lg">
          <img v-if="relatedArtist.imageUrl" :src="relatedArtist.imageUrl" alt="Artist art"
            class="w-32 h-32 rounded-full object-cover" />
          <div v-else class="w-32 h-32 rounded-full bg-muted flex-shrink-0 flex items-center justify-center">
            <Music class="w-16 h-16 text-muted-foreground" />
          </div>
          <div class="w-32">
            <h3 class="text-foreground font-medium truncate">{{ relatedArtist.name }}</h3>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { Button } from '@/components/ui/button'
import { Music, ChevronLeft, ChevronRight, Play, Pause, Shuffle, Star, ExternalLink } from 'lucide-vue-next'
import { MusicItem } from '@/types'

interface ArtistInfo {
  id: string
  name: string
  imageUrl?: string
  overview?: string
  providerIds?: Record<string, string>
  communityRating?: number
}

const props = defineProps<{
  artist: {
    id: string;
    name: string;
    imageUrl?: string;
  },
  songs: MusicItem[],
  albums: any[],
  allArtists: { id: string; name: string; imageUrl?: string; songCount: number }[],
  currentSong: MusicItem | null,
  isPlaying: boolean,
  serverUrl: string,
  token: string,
  userId: string,
}>()

const emit = defineEmits<{
  'play-song': [song: MusicItem],
  'select-album': [album: any],
  'play-album': [album: any],
  'play-artist-shuffle': [artist: { name: string }],
  'select-artist': [artist: { id: string, name: string; imageUrl?: string }],
}>()

const scrollContainer = ref<HTMLElement | null>(null)
const canScrollLeft = ref(false)
const canScrollRight = ref(false)
const artistDetails = ref<ArtistInfo | null>(null)
const showFullOverview = ref(false)

const artistSongs = computed(() => {
  return props.songs.filter(song => song.artists?.includes(props.artist.name))
    .sort((a, b) => (b.playCount ?? 0) - (a.playCount ?? 0))
})

const primarySongs = computed(() => {
  return artistSongs.value.filter(song => song.artists?.[0] === props.artist.name)
})

const featuredSongs = computed(() => {
  return artistSongs.value.filter(song => song.artists?.[0] !== props.artist.name)
})

const isFeaturedOnlyArtist = computed(() => {
  return primarySongs.value.length === 0 && featuredSongs.value.length > 0
})

const fetchArtistDetails = async () => {
  if (!props.artist.id) {
    console.log('No artist ID, skipping fetch.');
    return
  }
  try {
    const details: ArtistInfo = await invoke('get_artist_details', {
      artistId: props.artist.id,
      serverUrl: props.serverUrl,
      token: props.token,
      userId: props.userId,
    })
    artistDetails.value = details
  } catch (error) {
    console.error('Failed to fetch artist details:', error)
  }
}

const artistGenres = computed(() => {
  if (!artistSongs.value.length) {
    return []
  }

  const genreCounts = new Map<string, number>()
  artistSongs.value.forEach(song => {
    song.genres?.forEach(genre => {
      genreCounts.set(genre, (genreCounts.get(genre) || 0) + 1)
    })
  })

  if (genreCounts.size === 0) {
    return []
  }

  // Sort genres by count, descending
  const sortedGenres = [...genreCounts.entries()].sort((a, b) => b[1] - a[1])

  // Return the top 5 genre names
  return sortedGenres.slice(0, 5).map(([genre]) => genre)
})

const artistAlbums = computed(() => {
  const albumsForArtist = props.albums.filter(album => {
    // We need to find if any song in this album is by the current artist
    return props.songs.some(song => song.album === album.name && song.artists?.includes(props.artist.name))
  })
  // Deduplicate albums
  const uniqueAlbums = Array.from(new Map(albumsForArtist.map(album => [album.name, album])).values())
  return uniqueAlbums
})

const relatedArtists = computed(() => {
  // Configurable weights for scoring
  const COLLABORATION_SCORE = 10;
  const SHARED_GENRE_SCORE = 5;
  const SHARED_ALBUM_SCORE = 2; // For artists on the same compilation/album

  // 1. Get all data for the current artist
  const currentArtistName = props.artist.name;
  const currentArtistSongs = props.songs.filter(s => s.artists?.includes(currentArtistName));
  const currentArtistGenres = new Set(currentArtistSongs.flatMap(s => s.genres || []));
  const currentArtistAlbums = new Set(currentArtistSongs.map(s => s.album).filter(Boolean));

  const artistScores = new Map<string, number>();

  // 2. Iterate over every *other* artist to calculate a similarity score
  props.allArtists.forEach(otherArtist => {
    if (otherArtist.name === currentArtistName) return;

    let score = 0;
    const otherArtistSongs = props.songs.filter(s => s.artists?.includes(otherArtist.name));
    if (otherArtistSongs.length === 0) return;

    // 3. Calculate score based on collaborations, genres, and albums
    // Score for direct collaborations
    const collaborations = currentArtistSongs.filter(s => s.artists?.includes(otherArtist.name));
    score += collaborations.length * COLLABORATION_SCORE;

    // Score for shared genres
    const otherArtistGenres = new Set(otherArtistSongs.flatMap(s => s.genres || []));
    for (const genre of otherArtistGenres) {
      if (currentArtistGenres.has(genre)) {
        score += SHARED_GENRE_SCORE;
      }
    }

    // Score for shared albums (compilations)
    const otherArtistAlbums = new Set(otherArtistSongs.map(s => s.album).filter(Boolean));
    for (const album of otherArtistAlbums) {
      if (currentArtistAlbums.has(album!) && collaborations.length === 0) {
        // Only score shared albums if it's not a direct collaboration album
        const songsOnAlbumByOther = otherArtistSongs.filter(s => s.album === album);
        const songsOnAlbumByCurrent = currentArtistSongs.filter(s => s.album === album);
        if (songsOnAlbumByOther.length > 0 && songsOnAlbumByCurrent.length > 0) {
          score += SHARED_ALBUM_SCORE;
        }
      }
    }

    if (score > 0) {
      artistScores.set(otherArtist.name, score);
    }
  });

  // 4. Sort artists by score and return the top 6
  const sortedArtists = [...artistScores.entries()].sort((a, b) => b[1] - a[1]);

  const allArtistsMap = new Map(props.allArtists.map(a => [a.name, a]));

  return sortedArtists.slice(0, 6).map(([name]) => {
    const artistInfo = allArtistsMap.get(name);
    return {
      id: artistInfo?.id || '',
      name,
      imageUrl: artistInfo?.imageUrl,
    };
  });
})

const isFeaturedOnSong = (song: MusicItem) => {
  return song.artists?.[0] !== props.artist.name && !!song.artists?.includes(props.artist.name)
}

function updateScrollButtons() {
  if (scrollContainer.value) {
    const { scrollLeft, scrollWidth, clientWidth } = scrollContainer.value
    canScrollLeft.value = scrollLeft > 0
    canScrollRight.value = scrollLeft < scrollWidth - clientWidth - 1 // -1 for precision issues
  }
}

function scrollLeft() {
  scrollContainer.value?.scrollBy({ left: -248, behavior: 'smooth' })
}

function scrollRight() {
  scrollContainer.value?.scrollBy({ left: 248, behavior: 'smooth' })
}

onMounted(async () => {
  await nextTick()
  updateScrollButtons()
  window.addEventListener('resize', updateScrollButtons)
  fetchArtistDetails()
})

onUnmounted(() => {
  window.removeEventListener('resize', updateScrollButtons)
})

function playSong(song: MusicItem) {
  emit('play-song', song)
}
</script>

<style scoped>
.scrollbar-hide::-webkit-scrollbar {
  display: none;
}

.scrollbar-hide {
  -ms-overflow-style: none;
  scrollbar-width: none;
}

.carousel-container::before,
.carousel-container::after {
  content: '';
  position: absolute;
  top: 0;
  bottom: 0;
  z-index: 2;
  pointer-events: none;
  width: 3rem;
  transition: opacity 0.2s ease-in-out;
}

.carousel-container::before {
  left: 0;
  background-image: linear-gradient(to right, hsl(0 0% 100%), transparent);
  opacity: var(--left-fade-opacity, 0);
}

.dark .carousel-container::before {
  background-image: linear-gradient(to right, hsl(240 10% 3.9%), transparent);
}

.carousel-container::after {
  right: 0;
  background-image: linear-gradient(to left, hsl(0 0% 100%), transparent);
  opacity: var(--right-fade-opacity, 0);
}

.dark .carousel-container::after {
  background-image: linear-gradient(to left, hsl(240 10% 3.9%), transparent);
}
</style>
