<template>
  <div class='p-8 max-w-7xl mx-auto space-y-8'>
    <div class='flex justify-end'>
      <Button
        @click='showSkeleton = !showSkeleton'
        :disabled='libraryLoading || !libraryLoaded || !artist'
        size='sm'
        variant='outline'
      >
        {{ showSkeleton ? 'Hide' : 'Show' }} Skeleton (dev)
      </Button>
    </div>
    <div v-if='libraryLoading || !libraryLoaded || !artist || showSkeleton' class='space-y-8'>
      <!-- Header Skeleton -->
      <div class='flex items-center p-8 blur-card rounded-2xl'>
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

      <!-- Top Songs Skeleton -->
      <div class='space-y-4'>
        <Skeleton class='h-8 w-48' />
        <div class='grid sm:grid-cols-1 md:grid-cols-2 gap-x-8'>
          <div
            v-for='i in topSongsCount'
            :key='`top-song-skeleton-${i}`'
            class='flex items-center py-2.5 px-2 rounded-md'
          >
            <Skeleton class='w-10 h-10 rounded-md mr-3' />
            <div class='flex-1 space-y-2'>
              <Skeleton class='h-4 w-3/4' />
              <Skeleton class='h-3 w-1/2' />
            </div>
            <Skeleton class='h-4 w-12 ml-3' />
          </div>
        </div>
      </div>

      <!-- Albums Skeleton -->
      <div class='space-y-4'>
        <Skeleton class='h-8 w-32' />
        <div class='grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-6'>
          <div v-for='i in 6' :key='`albums-skeleton-${i}`' class='cursor-pointer group'>
            <div class='relative mb-4'>
              <Skeleton class='w-full aspect-square rounded-lg shadow-lg' />
            </div>
            <Skeleton class='h-6 w-3/4 mb-1' />
            <Skeleton class='h-4 w-1/2' />
          </div>
        </div>
      </div>
    </div>
    <div v-else-if='artist' class='space-y-12'>
      <!-- Header -->
      <div
        class='relative flex flex-col md:flex-row items-start p-8 blur-card rounded-2xl shadow-lg gap-8 overflow-hidden'
      >
        <!-- Backdrop Background -->
        <!-- <ImageLoader
          v-if='artist.imageTags?.Backdrop'
          :image-type='"Backdrop"'
          :item-id='artist.id'
          :server-url='serverUrl'
          :token='token'
          alt='Artist backdrop'
          class='absolute inset-0 w-full h-full object-cover opacity-10'
        /> -->
        <div class='flex-shrink-0 mx-auto md:mx-0'>
          <ImageLoader
            :item-id='artist.id'
            :server-url='serverUrl'
            :token='token'
            alt='Artist art'
            class='w-50 h-50 rounded-lg object-cover'
          >
            <template #fallback>
              <div class='w-50 h-50 rounded-lg bg-muted flex items-center justify-center'>
                <Music class='w-25 h-25 text-muted-foreground' />
              </div>
            </template>
          </ImageLoader>
        </div>
        <div class='flex-1 w-full'>
          <div class='flex flex-col lg:flex-row items-start gap-8'>
            <div class='space-y-4 flex-shrink-0'>
              <div>
                <h2 class='text-4xl font-bold'>
                  {{ artist.name }}
                </h2>
                <div class='flex flex-wrap items-center gap-x-4 gap-y-2 mt-2 text-muted-foreground'>
                  <div v-if='artist.communityRating' class='flex items-center gap-1'>
                    <Star class='w-4 h-4 text-yellow-500' />
                    <span>{{ artist.communityRating.toFixed(1) }} / 10</span>
                  </div>
                  <p v-if='isFeaturedOnlyArtist'>
                    Featured on {{ featuredSongs.length }} {{ featuredSongs.length === 1 ? 'song' : 'songs' }}
                  </p>
                  <p v-else>
                    {{ artistSongs.length }} songs across {{ artistAlbums.length }} albums
                  </p>
                </div>
                <div v-if='artistGenres.length > 0' class='flex flex-wrap gap-2 mt-4'>
                  <span
                    v-for='genre in artistGenres'
                    :key='genre'
                    class='px-2 py-1 text-xs font-semibold rounded-full bg-secondary text-secondary-foreground'
                  >
                    {{ genre }}
                  </span>
                </div>
                <!-- Provider Links -->
                <div v-if='validProviderLinks.length > 0' class='flex flex-wrap gap-4 pt-4'>
                  <a
                    v-for='link in validProviderLinks'
                    :key='link.provider'
                    :href='link.url'
                    class='text-sm font-medium text-accent hover:underline flex items-center gap-2'
                    rel='noopener noreferrer'
                    target='_blank'
                  >
                    <span v-if='link.icon' class='w-4 h-4' v-html='link.icon.svg' />
                    <span>{{ link.provider.replace('Artist', '') }}</span>
                  </a>
                </div>
              </div>

              <!-- Actions -->
              <div class='flex items-center gap-2'>
                <Button @click='playArtistShuffle'>
                  <Shuffle class='w-4 h-4 mr-2' />
                  Shuffle All
                </Button>
                <Button @click='showShareDialog = true' variant='outline'>
                  <Share2 class='w-4 h-4 mr-2' />
                  Share
                </Button>
              </div>
            </div>

            <!-- About Section -->
            <div class='space-y-4 flex-1 self-stretch'>
              <div v-if='artist.overview' class='prose dark:prose-invert max-w-none h-full flex flex-col'>
                <p :class="['flex-1', { 'line-clamp-4': !showFullOverview }]" v-html='artist.overview' />
                <Button
                  @click='showFullOverview = !showFullOverview'
                  v-if='artist.overview.length > 200'
                  class='px-0 self-start'
                  variant='link'
                >
                  {{ showFullOverview ? 'Show Less' : 'Read More' }}
                </Button>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Top Songs -->
      <div class='bg-card/50 backdrop-blur-sm border border-border/50 rounded-xl p-6 shadow-lg space-y-4'>
        <h3 class='text-2xl font-semibold'>
          {{ isFeaturedOnlyArtist ? 'Features' : 'Top Songs' }}
        </h3>
        <div class='grid sm:grid-cols-1 md:grid-cols-2 gap-x-8'>
          <div
            v-for='song in artistSongs.slice(0, topSongsCount)'
            @click='playSong(song)'
            :key='song.id'
            class='flex items-center p-2 hover:bg-muted/50 rounded-md cursor-pointer transition-colors group'
          >
            <div class='relative mr-4'>
              <img
                v-if='getSongImageUrl(song)'
                :src='getSongImageUrl(song)'
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
              <p
                v-if='(song.album && !isSingle(song)) || isFeaturedOnSong(song)'
                class='text-muted-foreground text-sm truncate'
              >
                <router-link
                  @click.stop
                  v-if='song.album && !isSingle(song)'
                  :to="{ name: 'album-detail', params: { albumName: song.album } }"
                  class='hover:underline'
                >
                  {{ song.album }}
                </router-link>
                <span v-if='(song.album && !isSingle(song)) && isFeaturedOnSong(song)' class='mx-1'>•</span>
                <span v-if='isFeaturedOnSong(song)'>
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
                </span>
              </p>
            </div>
            <p class='text-muted-foreground text-sm'>
              {{ song.playCount ?? 0 }} play{{ (song.playCount ?? 0) === 1 ? '' : 's' }}
            </p>
          </div>
        </div>
      </div>

      <!-- Albums Carousel -->
      <Carousel
        v-if='artistAlbums.length > 0'
        :disabled='libraryLoading || !libraryLoaded || !artist'
        :title="isFeaturedOnlyArtist ? 'Appears On' : 'Albums'"
      >
        <div
          v-for='album in artistAlbums'
          @click="$emit('select-album', album)"
          :key='album.name'
          class='cursor-pointer group hover:bg-muted/50 rounded-md transition-colors p-2'
        >
          <div class='relative mb-4'>
            <ImageLoader
              :alt='`${album.name} album art`'
              :item-id='album.id || album.name'
              :server-url='serverUrl'
              :token='token'
              class='w-full aspect-square rounded-lg object-cover shadow-lg group-hover:opacity-75 transition-opacity'
            >
              <template #fallback>
                <ImagePlaceholder
                  class='w-full aspect-square shadow-lg group-hover:opacity-75 transition-opacity'
                  size='large'
                  type='album'
                />
              </template>
            </ImageLoader>
            <div
              class='
                absolute inset-0 bg-black/50 rounded-lg opacity-0
                group-hover:opacity-100 transition-opacity
                flex items-center justify-center
              '
            >
              <Button
                @click.stop='playAlbum(album)'
                class='bg-white/20 hover:bg-white/30 backdrop-blur-sm text-white border border-white/20'
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
              <span v-if='isAlbumSingle(album)'>Single</span>
              <span v-else>{{ album.songs?.length || 0 }} songs</span>
            </p>
          </div>
        </div>
      </Carousel>

      <!-- Related Artists -->
      <Carousel v-if='relatedArtists.length > 0' :disabled='libraryLoading || !libraryLoaded || !artist' title='Related Artists'>
        <div
          v-for='relatedArtist in relatedArtists'
          @click="$emit('select-artist', relatedArtist)"
          :key='relatedArtist.name'
          class='cursor-pointer group hover:bg-muted/50 rounded-md transition-colors p-2'
        >
          <div class='relative mb-4'>
            <ImageLoader
              :item-id='relatedArtist.id'
              :server-url='serverUrl'
              :token='token'
              alt='Artist art'
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
          </div>
          <div class='text-center'>
            <h3 class='font-semibold truncate'>
              {{ relatedArtist.name }}
            </h3>
            <p class='text-sm text-muted-foreground truncate'>
              {{ relatedArtist.songs?.length || 0 }} songs
            </p>
          </div>
        </div>
      </Carousel>

      <!-- About Section -->
    </div>
    <div v-else class='text-center py-12 text-muted-foreground'>
      Artist not found.
    </div>

    <ShareDialog
      v-if='artist'
      v-model:open='showShareDialog'
      :item-id='artist.id'
      :item-name='artist.name'
      :item-type="'artist'"
    />
  </div>
</template>

<script setup lang="ts">
  import { computed, ref, watch } from 'vue'
  import { useRoute } from 'vue-router'
  import { useBreakpoints } from '@vueuse/core'
  import { Button } from '@/components/ui/button'
  import { Music, Play, Pause, Shuffle, Star, Share2 } from 'lucide-vue-next'
  import { Song, Album, Artist, commands } from '@/bindings'
  import type { SimpleIcon } from 'simple-icons'
  import {
    siMusicbrainz,
    siSpotify,
    siApplemusic,
    siYoutube,
    siSoundcloud,
    siBandcamp,
    siDiscogs,
    siLastdotfm,
    siWikipedia,
  } from 'simple-icons'
  import ImagePlaceholder from '@/components/shared/ImagePlaceholder.vue'
  import ImageLoader from '@/components/shared/ImageLoader.vue'
  import Carousel from '@/components/shared/Carousel.vue'
  import ShareDialog from '@/components/shared/ShareDialog.vue'
  import { uiLogger } from '@/lib/logger'
  import { Skeleton } from '@/components/ui/skeleton'

  const breakpoints = useBreakpoints({
    tablet: 768,
  })

  const isTabletOrLarger = breakpoints.greaterOrEqual('tablet')

  const topSongsCount = computed(() => isTabletOrLarger.value ? 10 : 5)

  const props = defineProps<{
    currentSong: Song | null,
    isPlaying:   boolean,
    serverUrl:   string,
    token:       string,
    userId:      string,
    libraryLoaded: boolean,
    libraryLoading: boolean,
    allArtists: Artist[],
    allSongs: Song[],
  }>()

  const emit = defineEmits<{
    'play-song':     [song: Song],
    'select-album':  [album: Album],
    'play-songs':    [songs: Song[]],
    'select-artist': [artist: Artist],
  }>()

  const route = useRoute()
  const artistId = computed(() => route.params.artistId as string)
  const showSkeleton = ref(false)
  const showFullOverview = ref(false)
  const showShareDialog = ref(false)

  const artist = computed(() => {
    if (!props.libraryLoaded || !props.allArtists.length) return null
    return props.allArtists.find(a => a.id === artistId.value) || null
  })

  const artistSongs = computed(() =>
    artist.value
      ? props.allSongs.filter(song =>
        song.artists
        && song.artists.includes(artist.value!.name)).sort((a, b) => (b.playCount ?? 0) - (a.playCount ?? 0))
      : [])

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
            songCount:   BigInt(0),
            songs:       [],
            imageTags:   song.imageTags,
            providerIds: null,
          })
        }
        const album = albumsMap.get(song.albumId)!
        album.songs!.push(song)
        album.songCount = BigInt(album.songs!.length)
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
    const currentArtistSongs = props.allSongs.filter(s => s.artists?.includes(currentArtistName))
    const currentArtistGenres = new Set(currentArtistSongs.flatMap(s => s.genres || []))
    const currentArtistAlbums = new Set(currentArtistSongs.map(s => s.album).filter(Boolean))

    const artistScores = new Map<string, number>()

    // 2. Iterate over every *other* artist to calculate a similarity score
    props.allArtists.forEach(otherArtist => {
      if (otherArtist.name === currentArtistName) return

      let score = 0
      const otherArtistSongs = props.allSongs.filter(s => s.artists?.includes(otherArtist.name))
      if (otherArtistSongs.length === 0) return

      // 3. Calculate score based on collaborations, genres, and albums
      // Score for direct collaborations
      const collaborations = currentArtistSongs.filter(s => s.artists?.includes(otherArtist.name))
      score += collaborations.length * COLLABORATION_SCORE

      // Score for shared genres
      const otherArtistGenres = new Set(otherArtistSongs.flatMap(s => s.genres || []))
      for (const genre of otherArtistGenres)
        if (currentArtistGenres.has(genre))
          score += SHARED_GENRE_SCORE

      // Score for shared albums (compilations)
      const otherArtistAlbums = new Set(otherArtistSongs.map(s => s.album).filter(Boolean))
      for (const album of otherArtistAlbums) {
        if (currentArtistAlbums.has(album!) && collaborations.length === 0) {
          // Only score shared albums if it's not a direct collaboration album
          const songsOnAlbumByOther = otherArtistSongs.filter(s => s.album === album)
          const songsOnAlbumByCurrent = currentArtistSongs.filter(s => s.album === album)
          if (songsOnAlbumByOther.length > 0 && songsOnAlbumByCurrent.length > 0)
            score += SHARED_ALBUM_SCORE
        }
      }

      if (score > 0)
        artistScores.set(otherArtist.name, score)
    })

    // 4. Sort artists by score and return the top 6
    const sortedArtists = [...artistScores.entries()].sort((a, b) => b[1] - a[1])

    const allArtistsMap = new Map(props.allArtists.map(a => [a.name, a]))

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


  const playArtistShuffle = () => {
    if (artistSongs.value.length > 0) {
      const shuffledSongs = [...artistSongs.value].sort(() => 0.5 - Math.random())
      emit('play-songs', shuffledSongs)
    }
  }

  const artistGenres = computed(() => {
    if (!artistSongs.value.length)
      return []

    const genreCounts = new Map<string, number>()
    artistSongs.value.forEach(song => {
      song.genres?.forEach(genre => {
        genreCounts.set(genre, (genreCounts.get(genre) || 0) + 1)
      })
    })

    if (!genreCounts.size)
      return []

    const sortedGenres = [...genreCounts.entries()].sort((a, b) => b[1] - a[1])

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
      if (name && name !== current)
        pairs.push({ id, name })
    }
    return pairs
  }

  const albumTrackCountsById = computed(() => {
    const counts = new Map<string, number>()

    for (const s of props.allSongs)
      if (s.albumId)
        counts.set(s.albumId, (counts.get(s.albumId) || 0) + 1)

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

    for (const s of tracks)
      if (s.albumArtists)
        for (const p of s.albumArtists)
          if (p.id && p.name) pairs.set(p.id, p.name)

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
  const albumCollaboratorsFor = (album: Album): NameId[] =>
    albumArtistPairsFor(album).filter(p => p.name !== artist.value?.name)


  const getIconForProvider = (provider: string): SimpleIcon | null => {
    const key = provider.toLowerCase().replace('artist', '')
    const iconMap: Record<string, SimpleIcon> = {
      musicbrainz: siMusicbrainz,
      spotify:     siSpotify,
      applemusic:  siApplemusic,
      youtube:     siYoutube,
      soundcloud:  siSoundcloud,
      bandcamp:    siBandcamp,
      discogs:     siDiscogs,
      lastfm:      siLastdotfm,
      wikipedia:   siWikipedia,
    }
    return iconMap[key] || null
  }

  const getProviderUrl = (provider: string, providerId: string): string | null => {
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
      'AudioDbArtist':     id => `https://www.theaudiodb.com/artist/${id}`,
    }

    const urlGenerator = providerUrls[provider]

    if (urlGenerator) {
      return urlGenerator(providerId)
    } else {
      uiLogger.warn(`No URL generator found for provider: ${provider}`)
      return null
    }
  }

  const validProviderLinks = computed(() => {
    if (!artist.value?.providerIds) return []

    return Object.entries(artist.value.providerIds)
      .map(([provider, providerId]) => {
        if (!providerId) return null

        const url = getProviderUrl(provider, providerId)
        const iconData = getIconForProvider(provider)

        if (url && iconData) {
          // Inject the brand color directly into the SVG for consistent display
          const coloredSvg = iconData.svg.replace('<svg', `<svg style="fill: #${iconData.hex};"`)
          return {
            provider,
            url,
            icon: { ...iconData, svg: coloredSvg },
          }
        }
        return null
      })
      .filter(Boolean) as { provider: string, url: string, icon: SimpleIcon }[]
  })

  const getSongImageUrl = (song: Song): string | undefined => {
    // First check if the song has its own album art
    if (song.albumArtUrl) {
      return song.albumArtUrl
    }

    // If not, check if the song belongs to an album and use the album's art
    if (song.albumId) {
      const album = artistAlbums.value.find(album => album.id === song.albumId)
      if (album?.albumArtUrl) {
        return album.albumArtUrl
      }
    }

    return undefined
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

