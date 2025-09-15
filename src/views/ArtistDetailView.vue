<template>
  <div v-bind='$attrs'>
    <div class='max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 pt-4 flex justify-end'>
      <Button
        @click='showSkeleton = !showSkeleton'
        :disabled='artistLoading'
        size='sm'
        variant='outline'
      >
        {{ showSkeleton ? 'Hide' : 'Show' }} Skeleton (dev)
      </Button>
    </div>
    <div v-if='artistLoading || showSkeleton' class='max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8'>
      <div class='flex items-center mb-4 pt-4'>
        <Skeleton class='w-48 h-48 rounded-lg' />
        <div class='ml-6 space-y-3 flex-1'>
          <Skeleton class='h-10 w-48' />
          <Skeleton class='h-6 w-52' />
          <Skeleton class='h-6 w-72' />
          <div class='flex items-center gap-2 pt-1'>
            <Button disabled>
              <Shuffle class='w-4 h-4 mr-2' />
              Shuffle All
            </Button>
          </div>
        </div>
      </div>
      <!-- Provider links skeleton (below header, like normal view) -->
      <div class='flex flex-wrap gap-4 mt-4 mb-4'>
        <Skeleton class='h-5 w-24 rounded-full' />
        <Skeleton class='h-5 w-28 rounded-full' />
        <Skeleton class='h-5 w-20 rounded-full' />
        <Skeleton class='h-5 w-16 rounded-full' />
      </div>
      <div class='md:flex md:space-x-8 items-stretch'>
        <div class='md:w-2/3 flex flex-col space-y-4 mb-8 md:mb-0'>
          <div class='flex justify-between items-center mb-4'>
            <h3 class='text-2xl font-semibold'>
              Albums
            </h3>
            <div class='space-x-2 z-10'>
              <Button :disabled='true' size='icon' variant='outline'>
                <ChevronLeft class='h-4 w-4' />
              </Button>
              <Button :disabled='true' size='icon' variant='outline'>
                <ChevronRight class='h-4 w-4' />
              </Button>
            </div>
          </div>
          <div
            :style="{ '--left-fade-opacity': 0, '--right-fade-opacity': 1 }"
            class='flex-grow relative carousel-container'
          >
            <div class='h-full overflow-x-auto scrollbar-hide'>
              <div class='grid grid-rows-1 grid-flow-col auto-cols-[18rem] gap-6'>
                <div
                  v-for='i in 10'
                  :key='`albums-skeleton-${i}`'
                  class='cursor-pointer group'
                >
                  <div class='relative mb-4'>
                    <Skeleton class='w-full aspect-square rounded-lg shadow-lg' />
                  </div>
                  <Skeleton class='h-6 w-3/4 mb-1' />
                  <Skeleton class='h-4 w-1/2 mb-1' />
                  <Skeleton class='h-3 w-1/3' />
                </div>
              </div>
            </div>
          </div>
        </div>
        <div class='md:w-1/3 space-y-4 mt-8 md:mt-0 flex flex-col min-h-0'>
          <h3 class='text-2xl font-semibold'>
            Top Songs
          </h3>
          <div class='flex flex-col justify-between flex-1 min-h-0 overflow-y-auto'>
            <div v-for='i in 5' :key='`top-song-skeleton-${i}`' class='flex items-center py-2.5 px-2 rounded-md'>
              <Skeleton class='w-10 h-10 rounded-md mr-3' />
              <div class='flex-1 space-y-2'>
                <Skeleton class='h-4 w-3/4' />
                <Skeleton class='h-3 w-1/2' />
              </div>
              <Skeleton class='h-4 w-12 ml-3' />
            </div>
          </div>
        </div>
      </div>

      <!-- Related Artists Skeleton -->
      <div class='mt-8'>
        <h3 class='text-2xl font-semibold mb-4'>
          Related Artists
        </h3>
        <div class='grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-6'>
          <div
            v-for='i in 5'
            :key='`related-artist-skeleton-${i}`'
            class='flex flex-col items-center text-center bg-card
                   hover:bg-muted/50 rounded-md p-4 cursor-pointer
                   transition-all group space-y-2 border hover:shadow-lg'
          >
            <Skeleton class='w-32 h-32 rounded-lg' />
            <div class='w-32'>
              <Skeleton class='h-6 w-full' />
            </div>
          </div>
        </div>
      </div>
    </div>
    <div v-else-if='artist' class='max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8'>
      <div class='flex items-center mb-4 pt-4'>
        <div class='flex-shrink-0'>
          <ImageLoader
            :item-id='artist.id'
            :server-url='serverUrl'
            :token='token'
            alt='Artist art'
            class='w-48 h-48 rounded-lg object-cover'
          >
            <template #fallback>
              <div class='w-48 h-48 rounded-lg bg-muted flex items-center justify-center'>
                <Music class='w-24 h-24 text-muted-foreground' />
              </div>
            </template>
          </ImageLoader>
        </div>
        <div class='ml-6'>
          <h2 class='text-4xl font-bold'>
            {{ artist.name }}
          </h2>
          <div v-if='artist.communityRating' class='flex items-center gap-1 mt-2 text-muted-foreground'>
            <Star class='w-4 h-4 text-yellow-500' />
            <span>{{ artist.communityRating.toFixed(1) }} / 10</span>
          </div>
          <p v-if='isFeaturedOnlyArtist' class='text-muted-foreground mt-2'>
            Featured on {{ featuredSongs.length }} {{ featuredSongs.length === 1 ? 'song' : 'songs' }}
          </p>
          <p v-else class='text-muted-foreground mt-2'>
            {{ artistSongs.length }} songs across {{ artistAlbums.length }} albums
          </p>
          <div v-if='artistGenres.length > 0' class='flex flex-wrap gap-2 mt-4'>
            <span
              v-for='genre in artistGenres'
              :key='genre'
              class='px-2 py-1 text-xs font-semibold rounded-full bg-secondary text-secondary-foreground'
            >
              {{ genre }}
            </span>
          </div>
          <div class='flex items-center gap-2 mt-4'>
            <Button @click='playArtistShuffle'>
              <Shuffle class='w-4 h-4 mr-2' />
              Shuffle All
            </Button>
          </div>
        </div>
      </div>

      <!-- Provider Links -->
      <div v-if='artist.providerIds' class='flex flex-wrap gap-4 mt-4 mb-4'>
        <a
          v-for='(providerId, provider) in artist.providerIds'
          :key='provider'
          :href='providerId ? getProviderUrl(provider, providerId) : "#"'
          class='text-sm font-medium text-blue-500 hover:underline flex items-center gap-1'
          rel='noopener noreferrer'
          target='_blank'
        >
          {{ provider }}
          <ExternalLink class='w-3 h-3' />
        </a>
      </div>

      <!-- Overview -->
      <div v-if='artist.overview' class='prose dark:prose-invert max-w-none mt-4'>
        <p :class="{ 'line-clamp-3': !showFullOverview }" v-html='artist.overview' />
        <Button
          @click='showFullOverview = !showFullOverview'
          v-if='artist.overview.length > 200'
          class='px-0'
          variant='link'
        >
          {{ showFullOverview ? 'Show Less' : 'Read More' }}
        </Button>
      </div>

      <div class='md:flex md:space-x-8 items-stretch'>
        <!-- Albums -->
        <div v-if='artistAlbums.length > 0' class='md:w-2/3 flex flex-col mb-8 md:mb-0'>
          <div class='flex justify-between items-center mb-4'>
            <h3 class='text-2xl font-semibold'>
              {{ isFeaturedOnlyArtist ? 'Appears On' : 'Albums' }}
            </h3>
            <div class='space-x-2 z-10'>
              <Button
                @click='scrollLeft'
                :disabled='artistLoading || !canScrollLeft'
                size='icon'
                variant='outline'
              >
                <ChevronLeft class='h-4 w-4' />
              </Button>
              <Button
                @click='scrollRight'
                :disabled='artistLoading || !canScrollRight'
                size='icon'
                variant='outline'
              >
                <ChevronRight class='h-4 w-4' />
              </Button>
            </div>
          </div>
          <div
            :style="{
              '--left-fade-opacity': canScrollLeft ? 1 : 0,
              '--right-fade-opacity': canScrollRight ? 1 : 0,
            }"
            class='flex-grow relative carousel-container'
          >
            <div @scroll='updateScrollButtons' ref='scrollContainer' class='h-full overflow-x-auto scrollbar-hide'>
              <div class='grid grid-rows-1 grid-flow-col auto-cols-[18rem] gap-6 h-full'>
                <div
                  v-for='album in artistAlbums'
                  @click="$emit('select-album', album)"
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
                    <h3 class='font-semibold truncate'>
                      {{ album.name }}
                      <span
                        v-if='isAlbumSingle(album)'
                        class='ml-2 inline-flex items-center rounded-full
                               px-2 py-0.5 text-xs font-medium
                               bg-secondary text-secondary-foreground'
                      >
                        Single
                      </span>
                    </h3>
                    <p v-if='albumCollaboratorsFor(album).length' class='text-sm text-muted-foreground truncate'>
                      with
                      <template v-for='(pair, idx) in albumCollaboratorsFor(album)' :key='pair.id || pair.name'>
                        <router-link
                          @click.stop
                          v-if='pair.id'
                          :to="{ name: 'artist-detail', params: { artistId: pair.id } }"
                          class='hover:underline'
                        >
                          {{ pair.name }}
                        </router-link>
                        <span v-else>{{ pair.name }}</span>
                        <span v-if='idx < albumCollaboratorsFor(album).length - 1'>, </span>
                      </template>
                    </p>
                    <p class='text-xs text-muted-foreground'>
                      {{ album.songCount }} songs
                    </p>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Top Songs by Artist -->
        <div class='md:w-1/3 flex flex-col min-h-0'>
          <h3 class='text-2xl font-semibold mb-4'>
            {{ isFeaturedOnlyArtist ? 'Features' : 'Top Songs' }}
          </h3>
          <div class='flex flex-col justify-between flex-1 min-h-0 overflow-y-auto'>
            <div
              v-for='song in artistSongs.slice(0, 5)'
              @click='playSong(song)'
              :key='song.id'
              class='flex items-center p-2 hover:bg-muted/50 rounded-md cursor-pointer transition-colors group'
            >
              <div class='relative mr-4'>
                <img
                  v-if='song.albumArtUrl'
                  :src='song.albumArtUrl'
                  alt='Album art'
                  class='w-10 h-10 rounded-md'
                >
                <div v-else class='w-10 h-10 rounded-md bg-muted flex-shrink-0' />
                <div
                  :class="[
                    'absolute inset-0 w-10 h-10 flex items-center justify-center transition-opacity',
                    currentSong?.id === song.id && isPlaying
                      ? 'opacity-100'
                      : 'opacity-0 group-hover:opacity-100',
                  ]"
                >
                  <Button
                    class='w-8 h-8 rounded-full bg-background/75 text-foreground hover:bg-background'
                    size='icon'
                    variant='ghost'
                  >
                    <Pause v-if='currentSong?.id === song.id && isPlaying' class='w-5 h-5' />
                    <Play v-else class='w-5 h-5' />
                  </Button>
                </div>
              </div>
              <div class='flex-1 min-w-0'>
                <p class='text-foreground font-medium truncate'>
                  {{ song.name }}
                </p>
                <p v-if='isFeaturedOnSong(song)' class='text-muted-foreground text-sm truncate'>
                  with
                  <template v-for='(collab, idx) in collaboratorsFor(song)' :key='collab.id || collab.name'>
                    <router-link
                      @click.stop
                      v-if='collab.id'
                      :to="{ name: 'artist-detail', params: { artistId: collab.id } }"
                      class='hover:underline'
                    >
                      {{ collab.name }}
                    </router-link>
                    <span v-else>{{ collab.name }}</span>
                    <span v-if='idx < collaboratorsFor(song).length - 1'>, </span>
                  </template>
                </p>
                <p v-if='song.album && !isSingle(song)' class='text-muted-foreground text-sm truncate'>
                  <router-link
                    @click.stop
                    :to="{ name: 'album-detail', params: { albumName: song.album } }"
                    class='hover:underline'
                  >
                    {{ song.album }}
                  </router-link>
                </p>
              </div>
              <p class='text-muted-foreground text-sm'>
                {{ song.playCount ?? 0 }} {{ (song.playCount ?? 0) === 1 ? 'play'
                  : 'plays' }}
              </p>
            </div>
          </div>
        </div>
      </div>

      <!-- Related Artists -->
      <div v-if='relatedArtists.length > 0' class='mt-8'>
        <h3 class='text-2xl font-semibold mb-4'>
          Related Artists
        </h3>
        <div class='grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-6'>
          <div
            v-for='relatedArtist in relatedArtists'
            @click="$emit('select-artist', relatedArtist)"
            :key='relatedArtist.name'
            class='
            flex flex-col items-center text-center bg-card
            hover:bg-muted/50 rounded-md p-4 cursor-pointer transition-all
            group space-y-2 border hover:shadow-lg
          '
          >
            <ImageLoader
              :item-id='relatedArtist.id'
              :server-url='serverUrl'
              :token='token'
              alt='Artist art'
              class='w-32 h-32 rounded-lg object-cover'
            >
              <template #fallback>
                <div class='w-32 h-32 rounded-lg bg-muted flex-shrink-0 flex items-center justify-center'>
                  <Music class='w-16 h-16 text-muted-foreground' />
                </div>
              </template>
            </ImageLoader>
            <div class='w-32'>
              <h3 class='text-foreground font-medium truncate'>
                {{ relatedArtist.name }}
              </h3>
            </div>
          </div>
        </div>
      </div>
    </div>
    <div v-else class='text-center py-12 text-muted-foreground'>
      Artist not found.
    </div>
  </div>
</template>

<script setup lang="ts">
  import { computed, ref, onMounted, onUnmounted, nextTick, watch } from 'vue'
  import { useRoute } from 'vue-router'
  import { Button } from '@/components/ui/button'
  import { Music, ChevronLeft, ChevronRight, Play, Pause, Shuffle, Star, ExternalLink } from 'lucide-vue-next'
  import { Song, Album, Artist, commands } from '@/bindings'
  import ImagePlaceholder from '@/components/shared/ImagePlaceholder.vue'
  import ImageLoader from '@/components/shared/ImageLoader.vue'
  import { uiLogger } from '@/lib/logger'
  import { Skeleton } from '@/components/ui/skeleton'

  const props = defineProps<{
    currentSong: Song | null,
    isPlaying:   boolean,
    serverUrl:   string,
    token:       string,
    userId:      string,
  }>()

  const emit = defineEmits<{
    'play-song':     [song: Song],
    'select-album':  [album: Album],
    'play-songs':    [songs: Song[]],
    'select-artist': [artist: Artist],
  }>()

  const route = useRoute()
  const artistId = computed(() => route.params.artistId as string)

  const { getSongs, getArtists, getArtist } = commands

  const scrollContainer = ref<HTMLElement | null>(null)
  const canScrollLeft = ref(false)
  const canScrollRight = ref(false)
  let scrollResizeObserver: ResizeObserver | null = null
  const artist = ref<Artist | null>(null)
  const artistLoading = ref(false)
  const showSkeleton = ref(false) // dev toggle for skeleton preview
  const allSongs = ref<Song[]>([])
  const allArtists = ref<Artist[]>([])
  const showFullOverview = ref(false)

  const artistSongs = computed(() => {
    if (!artist.value) return []
    return allSongs.value.filter(song => song.artists && song.artists.includes(artist.value!.name))
      .sort((a, b) => (b.playCount ?? 0) - (a.playCount ?? 0))
  })

  const artistAlbums = computed(() => {
    if (!artist.value) return []
    const albumsMap = new Map<string, Album>()

    artistSongs.value.forEach(song => {
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

  const relatedArtists = computed(() => {
    if (!artist.value) return []
    // Configurable weights for scoring
    const COLLABORATION_SCORE = 10
    const SHARED_GENRE_SCORE = 5
    const SHARED_ALBUM_SCORE = 2 // For artists on the same compilation/album

    // 1. Get all data for the current artist
    const currentArtistName = artist.value.name
    const currentArtistSongs = allSongs.value.filter(s => s.artists?.includes(currentArtistName))
    const currentArtistGenres = new Set(currentArtistSongs.flatMap(s => s.genres || []))
    const currentArtistAlbums = new Set(currentArtistSongs.map(s => s.album).filter(Boolean))

    const artistScores = new Map<string, number>()

    // 2. Iterate over every *other* artist to calculate a similarity score
    allArtists.value.forEach(otherArtist => {
      if (otherArtist.name === currentArtistName) return

      let score = 0
      const otherArtistSongs = allSongs.value.filter(s => s.artists?.includes(otherArtist.name))
      if (otherArtistSongs.length === 0) return

      // 3. Calculate score based on collaborations, genres, and albums
      // Score for direct collaborations
      const collaborations = currentArtistSongs.filter(s => s.artists?.includes(otherArtist.name))
      score += collaborations.length * COLLABORATION_SCORE

      // Score for shared genres
      const otherArtistGenres = new Set(otherArtistSongs.flatMap(s => s.genres || []))
      for (const genre of otherArtistGenres) {
        if (currentArtistGenres.has(genre)) {
          score += SHARED_GENRE_SCORE
        }
      }

      // Score for shared albums (compilations)
      const otherArtistAlbums = new Set(otherArtistSongs.map(s => s.album).filter(Boolean))
      for (const album of otherArtistAlbums) {
        if (currentArtistAlbums.has(album!) && collaborations.length === 0) {
          // Only score shared albums if it's not a direct collaboration album
          const songsOnAlbumByOther = otherArtistSongs.filter(s => s.album === album)
          const songsOnAlbumByCurrent = currentArtistSongs.filter(s => s.album === album)
          if (songsOnAlbumByOther.length > 0 && songsOnAlbumByCurrent.length > 0) {
            score += SHARED_ALBUM_SCORE
          }
        }
      }

      if (score > 0) {
        artistScores.set(otherArtist.name, score)
      }
    })

    // 4. Sort artists by score and return the top 6
    const sortedArtists = [...artistScores.entries()].sort((a, b) => b[1] - a[1])

    const allArtistsMap = new Map(allArtists.value.map(a => [a.name, a]))

    return sortedArtists.slice(0, 6).map(([name]) => {
      const artistInfo = allArtistsMap.get(name)
      return {
        id:              artistInfo?.id || '',
        name,
        imageUrl:        artistInfo?.imageUrl,
        songCount:       artistInfo?.songCount || 0,
        imageTags:       null,
        overview:        null,
        providerIds:     null,
        communityRating: null,
        songs:           null,
      } as Artist
    })
  })

  const primarySongs = computed(() => {
    return artistSongs.value.filter(song => song.artists?.[0] === artist.value?.name)
  })

  const featuredSongs = computed(() => {
    return artistSongs.value.filter(song => song.artists?.[0] !== artist.value?.name)
  })

  const isFeaturedOnlyArtist = computed(() => {
    return primarySongs.value.length === 0 && featuredSongs.value.length > 0
  })

  const fetchArtistData = async () => {
    artistLoading.value = true
    if (!artistId.value) {
      artist.value = null
      artistLoading.value = false
      return
    }
    try {
      const [artistsResult, songsResult] = await Promise.all([
        getArtists(props.serverUrl, props.token, true, false, null, null),
        getSongs(props.serverUrl, props.token, null, null, null, null),
      ])

      if (artistsResult.status === 'error') {
        uiLogger.error('Failed to fetch artists:', artistsResult.error)
        throw new Error(artistsResult.error)
      }
      if (songsResult.status === 'error') {
        uiLogger.error('Failed to fetch songs:', songsResult.error)
        throw new Error(songsResult.error)
      }

      uiLogger.debug('Fetched artists count:', artistsResult.data.length)
      uiLogger.debug('Fetched songs count:', songsResult.data.length)

      const foundArtist = artistsResult.data.find(a => a.id === artistId.value)
      if (foundArtist) {
        artist.value = foundArtist
        allArtists.value = artistsResult.data
        allSongs.value = songsResult.data
        uiLogger.debug('Artist detail (from list):', foundArtist)
      } else {
        uiLogger.error('Artist not found in library:', artistId.value)
        uiLogger.debug('Attempting direct artist fetch for id:', artistId.value)
        // If the artist is not found in the main list (e.g., a featured artist),
        // fetch their details directly.
        try {
          const directFetchResult = await getArtist(artistId.value, false, false)
          if (directFetchResult.status === 'error') {
            throw new Error(directFetchResult.error)
          }
          if (directFetchResult.data) {
            artist.value = directFetchResult.data
            uiLogger.debug('Artist detail (direct):', directFetchResult.data)
            // We might not have all songs here, but the detail view will render.
            // The song list for this artist might be incomplete.
            allSongs.value = songsResult.data // Keep the songs from the main library
            allArtists.value = artistsResult.data
          } else {
            artist.value = null
            uiLogger.warn('Direct artist fetch returned no data for id:', artistId.value)
          }
        } catch (directFetchError) {
          uiLogger.error('Failed to fetch artist details directly:', directFetchError)
          artist.value = null
        }
      }
    } catch (error) {
      uiLogger.error('Failed to fetch artist details:', error)
      artist.value = null
    } finally {
      artistLoading.value = false
    }
  }

  const playArtistShuffle = () => {
    if (artistSongs.value.length > 0) {
      const shuffledSongs = [...artistSongs.value].sort(() => 0.5 - Math.random())
      emit('play-songs', shuffledSongs)
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

  const isFeaturedOnSong = (song: Song) =>
    song.artists?.[0] !== artist.value?.name && !!song.artists?.includes(artist.value?.name || '')

  type SimpleArtist = { id: string | null, name: string }

  const collaboratorsFor = (song: Song): SimpleArtist[] => {
    const current = artist.value?.name
    const artists = song.artists || []
    const ids = song.artistIds || []

    // Build pairs for as many mapped entries as possible
    const pairs: SimpleArtist[] = []
    for (let i = 0; i < artists.length; i++) {
      const name = artists[i]
      const id = ids[i] || null
      if (name && name !== current) {
        pairs.push({ id, name })
      }
    }
    return pairs
  }

  // Determine if a song is a single: album name equals song name and album has only this track
  const albumTrackCountsById = computed(() => {
    const counts = new Map<string, number>()
    for (const s of allSongs.value) {
      if (s.albumId) counts.set(s.albumId, (counts.get(s.albumId) || 0) + 1)
    }
    return counts
  })

  const isSingle = (song: Song): boolean => {
    const sameName = (song.album || '').trim().toLowerCase() === (song.name || '').trim().toLowerCase()
    const trackCount = song.albumId ? (albumTrackCountsById.value.get(song.albumId) || 0) : 0
    return !!song.album && sameName && trackCount <= 1
  }

  const isAlbumSingle = (album: Album): boolean => {
    if (!album) return false
    const tracks = album.songs || []
    if (tracks.length === 1) {
      const only = tracks[0]
      const sameName = (album.name || '').trim().toLowerCase() === (only.name || '').trim().toLowerCase()
      return sameName
    }
    // Fallback using counts map if songs not populated
    if (album.id) {
      const count = albumTrackCountsById.value.get(album.id) || 0
      if (count === 1) return true
    }
    return false
  }

  type NameId = { id: string | null, name: string }

  const albumArtistPairsFor = (album: Album): NameId[] => {
    const pairs = new Map<string, string>()
    const tracks = album.songs || []
    for (const s of tracks) {
      if (s.albumArtists) {
        for (const p of s.albumArtists) {
          if (p.id && p.name) pairs.set(p.id, p.name)
        }
      }
    }
    if (pairs.size === 0 && tracks.length) {
      const first = tracks[0]
      if (first.artistIds && first.artists && first.artistIds.length === first.artists.length) {
        first.artistIds.forEach((id, i) => {
          const name = first.artists![i]
          if (id && name) pairs.set(id, name)
        })
      } else if (album.artistId && album.artist) {
        pairs.set(album.artistId, album.artist)
      }
    }
    return Array.from(pairs, ([id, name]) => ({ id, name }))
  }

  // Album carousel: collaborators excluding the current artist only
  const albumCollaboratorsFor = (album: Album): NameId[] => {
    const allPairs = albumArtistPairsFor(album)
    const current = artist.value?.name
    const filtered = allPairs.filter(p => p.name !== current)
    // Show only if there is at least one collaborator AND the album isn't solely by the current artist
    return filtered
  }

  const updateScrollButtons = () => {
    if (scrollContainer.value) {
      const { scrollLeft, scrollWidth, clientWidth } = scrollContainer.value
      canScrollLeft.value = scrollLeft > 0
      canScrollRight.value = scrollLeft < scrollWidth - clientWidth - 1 // -1 for precision issues
    }
  }

  const scrollLeft = () => {
    scrollContainer.value?.scrollBy({ left: -256, behavior: 'smooth' })
  }

  const scrollRight = () => {
    scrollContainer.value?.scrollBy({ left: 256, behavior: 'smooth' })
  }

  watch(artistId, fetchArtistData, { immediate: true })

  // Ensure scroll buttons/fade update when the album list changes/render completes
  watch(() => artistAlbums.value.length, async () => {
    await nextTick()
    updateScrollButtons()
  })

  onMounted(async () => {
    await nextTick()
    updateScrollButtons()
    window.addEventListener('resize', updateScrollButtons)
    if (scrollContainer.value) {
      scrollResizeObserver = new ResizeObserver(() => updateScrollButtons())
      scrollResizeObserver.observe(scrollContainer.value)
    }
  })

  onUnmounted(() => {
    window.removeEventListener('resize', updateScrollButtons)
    if (scrollResizeObserver) {
      scrollResizeObserver.disconnect()
      scrollResizeObserver = null
    }
  })

  const getProviderUrl = (provider: string, providerId: string): string => {
    const providerUrls: Record<string, (id: string) => string> = {
      'MusicBrainzArtist': id => `https://musicbrainz.org/artist/${id}`,
      'SpotifyArtist':     id => `https://open.spotify.com/artist/${id}`,
      'AppleMusicArtist':  id => `https://music.apple.com/artist/${id}`,
      'YouTubeArtist':     id => `https://www.youtube.com/channel/${id}`,
      'SoundCloudArtist':  id => `https://soundcloud.com/${id}`,
      'BandcampArtist':    id => `https://bandcamp.com/artist/${id}`,
      'DiscogsArtist':     id => `https://www.discogs.com/artist/${id}`,
      'LastFmArtist':      id => `https://www.last.fm/music/${encodeURIComponent(id)}`,
      'WikipediaArtist':   id => `https://en.wikipedia.org/wiki/${encodeURIComponent(id)}`,
      // Fallback mappings for generic names
      'MusicBrainz':       id => `https://musicbrainz.org/artist/${id}`,
      'Spotify':           id => `https://open.spotify.com/artist/${id}`,
      'AppleMusic':        id => `https://music.apple.com/artist/${id}`,
      'YouTube':           id => `https://www.youtube.com/channel/${id}`,
      'SoundCloud':        id => `https://soundcloud.com/${id}`,
      'Bandcamp':          id => `https://bandcamp.com/artist/${id}`,
      'Discogs':           id => `https://www.discogs.com/artist/${id}`,
      'LastFm':            id => `https://www.last.fm/music/${encodeURIComponent(id)}`,
      'Wikipedia':         id => `https://en.wikipedia.org/wiki/${encodeURIComponent(id)}`,
    }

    const urlGenerator = providerUrls[provider]
    return urlGenerator ? urlGenerator(providerId) : `https://www.google.com/search?q=${artist.value?.name}+${provider}`
  }

  const playSong = (song: Song) => {
    emit('play-song', song)
  }

  const playAlbum = (album: Album) => {
    if (album.songs && album.songs.length > 0)
      emit('play-songs', [...album.songs].sort((a, b) => (a.trackNumber ?? 0) - (b.trackNumber ?? 0)))
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
  background-image: linear-gradient(to right, var(--background), transparent);
  opacity: var(--left-fade-opacity, 0);
}

.carousel-container::after {
  right: 0;
  background-image: linear-gradient(to left, var(--background), transparent);
  opacity: var(--right-fade-opacity, 0);
}
</style>

