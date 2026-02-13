<script setup lang="ts">
  import { MoreHorizontal, Music, Pause, Play, Share2, Shuffle, Star } from 'lucide-vue-next'
  import { computed, inject, onMounted, onUnmounted, ref, watch } from 'vue'
  import { useRoute } from 'vue-router'

  import AddToPlaylistMenu from '../../components/shared/AddToPlaylistMenu.vue'
  import AlbumCard from '../../components/shared/AlbumCard.vue'
  import Carousel from '../../components/shared/Carousel.vue'
  import ImageLoader from '../../components/shared/ImageLoader.vue'
  import ImagePlaceholder from '../../components/shared/ImagePlaceholder.vue'
  import ShareDialog from '../../components/shared/ShareDialog.vue'
  import Button from '../../components/ui/Button.vue'
  import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
  } from '../../components/ui/dropdown-menu'
  import { Skeleton } from '../../components/ui/skeleton'
  import { scrollElementKey } from '../../composables/useMainLayout'
  import { ApiError, runAureliaEffect } from '../../effect'
  import { getArtistEffect, getRelatedArtistsEffect } from '../../effect/services/api'
  import { Album, Artist, Song } from '../../index'
  import { logger } from '../../lib/logger'
  import { formatDuration } from '../../lib/utils'
  import { usePlayerStore } from '../../stores'
  import { useAuthStore } from '../../stores/auth'
  import { useLibraryStore } from '../../stores/library'

  const topSongsCount = 10

  const emit = defineEmits<{
    'play-song':     [song: Song],
    'play-songs':    [songs: Song[]],
    'select-album':  [album: Album],
    'select-artist': [artist: Artist],
  }>()

  const authStore = useAuthStore()
  const libraryStore = useLibraryStore()
  const playerStore = usePlayerStore()

  // Create computed properties from stores
  const allSongs = computed(() => libraryStore.allSongs as Song[])
  const libraryLoaded = computed(() => libraryStore.isLoaded)
  const libraryLoading = computed(() => libraryStore.isLoading)
  const isLoading = computed(() => libraryLoading.value || loadingFallbackSongs.value)
  const serverUrl = computed(() => authStore.serverUrl)
  const token = computed(() => authStore.token)

  const route = useRoute()
  const id = computed(() => {
    const params = route.params
    if ('id' in params) {
      const param = params.id
      if (typeof param === 'string') return param
      if (Array.isArray(param)) return param[0] ?? ''
    }
    return ''
  })
  const showFullOverview = ref(false)
  const showShareDialog = ref(false)

  // Get artist from store like albums do
  const artist = computed(() =>
    id.value
    && libraryLoaded.value
    && libraryStore.allArtistsWithSongs.length > 0
      ? libraryStore.allArtistsWithSongs.find(a => a.id === id.value) || null
      : null,
  )

  // Fallback: fetch songs for artists with 0 songs (Jellyfin ID mismatch issue)
  const fallbackSongsCache = new Map<string, Song[]>()
  const fallbackSongs = ref<Song[]>([])
  const loadingFallbackSongs = ref(false)
  const artistDataError = ref(false)

  const fetchFallbackSongs = async (artistId: string): Promise<void> => {
    if (fallbackSongsCache.has(artistId)) {
      fallbackSongs.value = fallbackSongsCache.get(artistId)!
      artistDataError.value = fallbackSongs.value.length === 0
      return
    }

    loadingFallbackSongs.value = true
    artistDataError.value = false
    try {
      const artistData = await runAureliaEffect(getArtistEffect(artistId))
      if (artistData.songs) {
        const songs = artistData.songs as Song[]
        fallbackSongs.value = songs
        fallbackSongsCache.set(artistId, songs)

        if (artistData.songs.length === 0)
          artistDataError.value = true
      } else {
        artistDataError.value = true
      }
    } catch (cause) {
      const errorMessage = cause instanceof ApiError
        ? cause.message
        : String(cause)
      logger.error('Error fetching fallback artist songs:', errorMessage)
      artistDataError.value = true
    } finally {
      loadingFallbackSongs.value = false
    }
  }

  // Watch artist and fetch fallback songs if needed
  watch(artist, async newArtist => {
    if (newArtist && (!newArtist.songs || newArtist.songs.length === 0))
      await fetchFallbackSongs(newArtist.id)
    else
      fallbackSongs.value = []
  }, { immediate: true })

  const artistSongs = computed<Song[]>(() => {
    // Use fallback songs if artist has none in store
    if (artist.value && (!artist.value.songs || artist.value.songs.length === 0) && fallbackSongs.value.length > 0)
      return [...fallbackSongs.value].sort((a, b) => (b.playCount ?? 0) - (a.playCount ?? 0)) as Song[]

    return artist.value && artist.value.songs
      ? [...artist.value.songs].sort((a, b) => (b.playCount ?? 0) - (a.playCount ?? 0)) as Song[]
      : []
  })

  const artistAlbums = computed(() => {
    if (!artistSongs.value || artistSongs.value.length === 0) return []
    const albumIds = new Set(artistSongs.value.map(s => s.albumId).filter(Boolean))
    return libraryStore.allAlbums.filter(album => album.id && albumIds.has(album.id)) as Album[]
  })

  const relatedArtistsCache = new Map<string, Artist[]>()
  const relatedArtists = ref<Artist[]>([])

  watch(artist, async newArtist => {
    if (newArtist) {
      // Check cache first
      if (relatedArtistsCache.has(newArtist.id)) {
        relatedArtists.value = relatedArtistsCache.get(newArtist.id)!
        return
      }

      try {
        const artists = await runAureliaEffect(getRelatedArtistsEffect(newArtist.id))
        relatedArtists.value = artists as Artist[]
        relatedArtistsCache.set(newArtist.id, artists as Artist[])
      } catch (cause) {
        logger.warn('Failed to load related artists', cause)
      }
    }
  })

  const primarySongs = computed(() => artistSongs.value.filter(song => song.artists?.[0] === artist.value?.name))
  const featuredSongs = computed(() => artistSongs.value.filter(song => song.artists?.[0] !== artist.value?.name))
  const isFeaturedOnlyArtist = computed(() => primarySongs.value.length === 0 && featuredSongs.value.length > 0)

  const playArtistShuffle = (): void => {
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

  const isFeaturedOnSong = (song: Song): boolean =>
    song.artists?.[0] !== artist.value?.name && !!song.artists?.includes(artist.value?.name || '')

  type SimpleArtist = { id: null | string, name: string }

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

    for (const s of allSongs.value)
      if (s.albumId)
        counts.set(s.albumId, (counts.get(s.albumId) || 0) + 1)

    return counts
  })

  const isSingle = (song: Song): boolean => {
    const sameName = (song.album || '').trim().toLowerCase() === (song.name || '').trim().toLowerCase()
    const trackCount = song.albumId ? (albumTrackCountsById.value.get(song.albumId) || 0) : 0
    return !!song.album && sameName && trackCount <= 1
  }

  type NameId = { id: null | string, name: string }

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

  const playSong = (song: Song): void => {
    emit('play-song', song)
  }

  const playAlbum = (album: Album): void => {
    if (album.songs && album.songs.length > 0)
      emit('play-songs', [...album.songs].sort((a, b) => (a.trackNumber ?? 0) - (b.trackNumber ?? 0)))
  }

  // Scroll tracking for sticky header
  const scrollElement = inject(scrollElementKey, ref<HTMLElement | null>(null))
  const heroRef = ref<HTMLElement | null>(null)
  const scrollY = ref(0)
  const heroHeight = ref(400)

  // Show sticky header when scrolled past 60% of hero
  const showStickyHeader = computed(() => scrollY.value > heroHeight.value * 0.6)

  const handleScroll = (): void => {
    if (scrollElement.value) {
      scrollY.value = scrollElement.value.scrollTop
    }
  }

  const updateHeroHeight = (): void => {
    if (heroRef.value) {
      heroHeight.value = heroRef.value.offsetHeight
    }
  }

  watch(scrollElement, (el, oldEl) => {
    if (oldEl) {
      oldEl.removeEventListener('scroll', handleScroll)
    }
    if (el) {
      el.addEventListener('scroll', handleScroll, { passive: true })
      handleScroll()
    }
  }, { immediate: true })

  onMounted(() => {
    updateHeroHeight()
    window.addEventListener('resize', updateHeroHeight)
  })

  onUnmounted(() => {
    if (scrollElement.value) {
      scrollElement.value.removeEventListener('scroll', handleScroll)
    }
    window.removeEventListener('resize', updateHeroHeight)
  })
</script>

<template>
  <div class='relative'>
    <!-- Sticky Header - uses sticky positioning within scroll container -->
    <div
      v-if='artist'
      :class="[
        'sticky top-0 z-40 overflow-hidden',
        'bg-sidebar/95 backdrop-blur-md border-b border-border/20 shadow-md',
        'transition-all duration-200 ease-out',
      ]"
      :style="{ height: showStickyHeader ? '64px' : '0px' }"
    >
      <div class='h-16 flex items-center gap-4 px-6 md:px-10 lg:px-16 max-w-7xl mx-auto'>
        <!-- Artist Thumbnail -->
        <div class='shrink-0'>
          <ImageLoader
            :item-id='artist.id'
            :server-url='serverUrl'
            :token='token'
            :width='100'
            class='size-10 rounded-full shadow-md object-cover'
          >
            <template #fallback>
              <div class='size-10 rounded-full bg-muted flex items-center justify-center'>
                <Music class='size-5 text-muted-foreground' />
              </div>
            </template>
          </ImageLoader>
        </div>

        <!-- Artist Info -->
        <div class='flex-1 min-w-0'>
          <h2 class='font-bold text-foreground truncate'>
            {{ artist.name }}
          </h2>
          <p class='text-sm text-muted-foreground truncate'>
            {{ artistSongs.length }} songs
          </p>
        </div>

        <!-- Shuffle Button -->
        <Button
          @click='playArtistShuffle'
          class='shrink-0'
          size='sm'
        >
          <Shuffle class='size-4' />
          <span class='ml-1.5'>Shuffle</span>
        </Button>
      </div>
    </div>

    <!-- Hero Section -->
    <section
      v-if='artist || isLoading'
      ref='heroRef'
      class='
        relative isolate overflow-hidden min-h-[400px]
        bg-linear-to-b from-sidebar via-sidebar to-background
      '
    >
      <div class='absolute inset-0 overflow-hidden'>
        <div class='absolute inset-0 opacity-20'>
          <ImageLoader
            v-if='artist && !isLoading'
            :item-id='artist.id'
            :server-url='serverUrl'
            :token='token'
            :width='600'
            class='size-full object-cover blur-2xl scale-110'
          />
        </div>

        <div
          class='
            absolute bottom-0 left-0 right-0 h-40 pointer-events-none
            bg-linear-to-t from-background to-transparent
          '
        />
      </div>

      <div class='relative z-10 flex flex-col items-center py-12 px-6 md:px-10 lg:px-16'>
        <div class='w-full max-w-7xl space-y-8'>
          <div class='flex items-start justify-between gap-8 lg:gap-12'>
            <div class='flex-1 min-w-0 space-y-6'>
              <template v-if='isLoading'>
                <Skeleton class='h-12 w-3/4 rounded-lg' />
                <Skeleton class='h-8 w-1/2 rounded-lg' />
                <Skeleton class='h-5 w-2/3 rounded-lg' />
                <div class='flex gap-3 pt-2'>
                  <Skeleton class='h-10 w-32 rounded-lg' />
                  <Skeleton class='h-10 w-32 rounded-lg' />
                </div>
              </template>
              <template v-else-if='artist'>
                <h1 class='text-5xl lg:text-6xl font-black text-white'>
                  {{ artist.name }}
                </h1>

                <div class='flex flex-wrap gap-4 items-center'>
                  <div v-if='artist.communityRating' class='flex items-center gap-1'>
                    <Star class='size-4 text-yellow-500' />
                    <span class='text-sm text-white/90 font-semibold'>
                      {{ artist.communityRating.toFixed(1) }} / 10
                    </span>
                  </div>
                  <p class='text-sm text-white/70'>
                    <template v-if='isFeaturedOnlyArtist'>
                      Featured on {{ featuredSongs.length }} {{ featuredSongs.length === 1 ? 'song' : 'songs' }}
                    </template>
                    <template v-else>
                      {{ artistSongs.length }} songs across {{ artistAlbums.length }} albums
                    </template>
                  </p>
                </div>

                <div v-if='artistGenres.length > 0' class='flex flex-wrap gap-2'>
                  <span
                    v-for='genre in artistGenres.slice(0, 5)'
                    :key='genre'
                    class='px-3 py-1 bg-white/10 text-white text-xs font-semibold rounded-full border border-white/20'
                  >
                    {{ genre }}
                  </span>
                </div>

                <div class='flex items-center gap-3 pt-2'>
                  <button
                    @click='playArtistShuffle'
                    class='
                      px-6 py-3 bg-accent hover:bg-accent/90 text-sidebar font-bold rounded-lg
                      transition-all duration-200 flex items-center gap-2 shadow-lg
                      hover:shadow-xl
                    '
                  >
                    <Shuffle class='h-5 w-5' />
                    <span>Shuffle All</span>
                  </button>
                  <DropdownMenu>
                    <DropdownMenuTrigger as-child>
                      <button
                        class='
                          px-6 py-3 bg-white/10 hover:bg-white/20 text-white font-semibold rounded-lg
                          border border-white/20 transition-all duration-200
                          backdrop-blur-sm hover:backdrop-blur-md
                        '
                      >
                        <MoreHorizontal class='h-5 w-5' />
                      </button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align='start'>
                      <AddToPlaylistMenu :songs='artistSongs' type='dropdown' />
                      <DropdownMenuItem @click='showShareDialog = true'>
                        <Share2 class='size-4 mr-2' />
                        Share
                      </DropdownMenuItem>
                    </DropdownMenuContent>
                  </DropdownMenu>
                </div>
              </template>
            </div>

            <div class='hidden lg:flex shrink-0 items-start justify-end'>
              <template v-if='isLoading'>
                <Skeleton class='w-64 h-64 rounded-2xl' />
              </template>
              <template v-else-if='artist'>
                <div class='relative group'>
                  <div
                    class='
                      absolute -inset-4 rounded-3xl blur-xl opacity-0
                      group-hover:opacity-100 transition-opacity duration-300
                      bg-linear-to-br from-accent/30 to-accent/10
                    '
                  />

                  <ImageLoader
                    :item-id='artist.id'
                    :server-url='serverUrl'
                    :token='token'
                    :width='400'
                    class='
                      relative w-64 h-64 rounded-2xl shadow-2xl object-cover
                      transition-shadow duration-300 group-hover:shadow-2xl
                    '
                  >
                    <template #fallback>
                      <div class='relative w-64 h-64 rounded-2xl shadow-2xl bg-muted flex items-center justify-center'>
                        <Music class='size-32 text-muted-foreground' />
                      </div>
                    </template>
                  </ImageLoader>
                </div>
              </template>
            </div>
          </div>

          <div v-if='artist && artist.overview && !isLoading' class='prose dark:prose-invert max-w-none'>
            <p
              :class="[
                'text-white/90 text-sm leading-relaxed',
                { 'line-clamp-3': !showFullOverview }
              ]"
              v-html='artist.overview'
            />
            <Button
              @click='showFullOverview = !showFullOverview'
              v-if='artist.overview.length > 200'
              class='mt-3 text-white hover:text-accent'
              variant='link'
            >
              {{ showFullOverview ? 'Show Less' : 'Read More' }}
            </Button>
          </div>
        </div>
      </div>
    </section>

    <section class='flex flex-col px-6 md:px-10 lg:px-16'>
      <div
        v-if='artistDataError && artist'
        class='py-4 bg-destructive/10 border-b border-destructive/20 flex items-start gap-3'
      >
        <div class='text-destructive'>
          ⚠️
        </div>
        <div class='flex-1 text-sm'>
          <p class='font-semibold text-destructive'>
            Artist Data Issue
          </p>
          <p class='text-muted-foreground mt-1'>
            This artist has a metadata mismatch in your Jellyfin library.
            Try re-identifying this artist in Jellyfin to fix the issue.
          </p>
        </div>
      </div>

      <!-- Top Songs List -->
      <section class='flex justify-center'>
        <div class='w-full max-w-7xl py-5'>
          <h2 class='text-2xl md:text-3xl font-bold text-foreground mb-4'>
            {{ isFeaturedOnlyArtist ? 'Features' : 'Top Songs' }}
          </h2>

          <!-- Loading skeletons -->
          <div v-if='isLoading || artistSongs.length === 0' class='space-y-1'>
            <div
              v-for='n in 5'
              :key='`skeleton-${n}`'
              class='flex items-center gap-4 px-3 py-3 rounded-lg'
            >
              <Skeleton class='w-8 h-6 rounded' />
              <Skeleton class='size-12 rounded-lg shrink-0' />
              <div class='flex-1 min-w-0'>
                <Skeleton class='h-5 w-2/3 mb-1.5' />
                <Skeleton class='h-3.5 w-1/3' />
              </div>
              <Skeleton class='w-12 h-4' />
            </div>
          </div>

          <!-- Top Songs List -->
          <div v-else class='space-y-1'>
            <div
              v-for='(song, index) in artistSongs.slice(0, topSongsCount)'
              @click='playSong(song)'
              :key='song.id'
              :class="[
                'group flex items-center gap-4 px-3 py-2.5 rounded-lg cursor-pointer',
                'transition-all duration-150',
                playerStore.currentSong?.id === song.id
                  ? 'bg-accent/10 ring-1 ring-accent/20'
                  : 'hover:bg-white/5'
              ]"
            >
              <!-- Rank Number / Play Button -->
              <div class='w-8 flex items-center justify-center shrink-0'>
                <span
                  :class="[
                    'tabular-nums font-bold text-lg transition-all duration-150',
                    playerStore.currentSong?.id === song.id
                      ? 'text-accent'
                      : 'text-muted-foreground group-hover:opacity-0'
                  ]"
                >
                  {{ index + 1 }}
                </span>
                <Button
                  @click.stop='playSong(song)'
                  :class="[
                    'absolute size-8 transition-all duration-150',
                    playerStore.currentSong?.id === song.id && playerStore.isPlaying
                      ? 'opacity-100'
                      : 'opacity-0 group-hover:opacity-100'
                  ]"
                  size='icon'
                  variant='ghost'
                >
                  <Pause
                    v-if='playerStore.currentSong?.id === song.id && playerStore.isPlaying'
                    class='size-4 text-accent'
                  />
                  <Play
                    v-else
                    class='size-4 text-accent fill-accent'
                  />
                </Button>
              </div>

              <!-- Album Art Thumbnail -->
              <div class='shrink-0'>
                <ImageLoader
                  :item-id='song.albumId || song.id'
                  :server-url='serverUrl'
                  :token='token'
                  :width='100'
                  class='size-12 rounded-lg shadow-sm object-cover'
                >
                  <template #fallback>
                    <div class='size-12 rounded-lg bg-muted flex items-center justify-center'>
                      <Music class='size-5 text-muted-foreground' />
                    </div>
                  </template>
                </ImageLoader>
              </div>

              <!-- Song Info -->
              <div class='flex-1 min-w-0 py-1'>
                <h3
                  :class="[
                    'font-medium truncate transition-colors duration-150',
                    playerStore.currentSong?.id === song.id
                      ? 'text-accent'
                      : 'text-foreground group-hover:text-accent'
                  ]"
                >
                  {{ song.name }}
                </h3>
                <p class='text-sm text-muted-foreground truncate mt-0.5'>
                  <template v-if='(song.album && !isSingle(song)) || isFeaturedOnSong(song)'>
                    <template v-if='song.album && song.albumId && !isSingle(song)'>
                      <RouterLink
                        @click.stop
                        :to='`/albums/${song.albumId}`'
                        class='hover:underline hover:text-accent'
                      >
                        {{ song.album }}
                      </RouterLink>
                    </template>
                    <span v-if='(song.album && !isSingle(song)) && isFeaturedOnSong(song)' class='mx-1'>•</span>
                    <span v-if='isFeaturedOnSong(song)'>
                      with
                      <template v-for='(collab, idx) in collaboratorsFor(song)' :key='collab.id || collab.name'>
                        <RouterLink
                          @click.stop
                          v-if='collab.id'
                          :to='`/artists/${collab.id}`'
                          class='hover:underline hover:text-accent'
                        >
                          {{ collab.name }}
                        </RouterLink>
                        <span v-else>{{ collab.name }}</span>
                        <span v-if='idx < collaboratorsFor(song).length - 1'>, </span>
                      </template>
                    </span>
                  </template>
                </p>
              </div>

              <!-- Duration -->
              <div class='w-14 text-right text-sm text-muted-foreground tabular-nums shrink-0'>
                {{ formatDuration(song.duration) }}
              </div>
            </div>
          </div>
        </div>
      </section>

      <section class='flex justify-center'>
        <div class='w-full max-w-7xl'>
          <Carousel
            v-if='artistAlbums.length > 0'
            :disabled='libraryLoading || !libraryLoaded || !artist'
            :title="isFeaturedOnlyArtist ? 'Appears On' : 'Albums'"
          >
            <AlbumCard
              v-for='album in artistAlbums'
              @click="$emit('select-album', { ...album })"
              @play='playAlbum'
              :key='album.name'
              :album='album'
              :collaborators='albumCollaboratorsFor(album)'
              :server-url='serverUrl'
              :token='token'
            />
          </Carousel>
        </div>
      </section>

      <section class='flex justify-center'>
        <div class='w-full max-w-7xl'>
          <Carousel
            v-if='relatedArtists.length > 0'
            :disabled='libraryLoading || !libraryLoaded || !artist'
            title='Related Artists'
          >
            <div
              v-for='relatedArtist in relatedArtists'
              @click="$emit('select-artist', relatedArtist)"
              :key='relatedArtist.name'
              class='cursor-pointer group'
            >
              <div class='relative mb-3 overflow-hidden rounded-lg'>
                <ImageLoader
                  :item-id='relatedArtist.id'
                  :server-url='serverUrl'
                  :token='token'
                  :width='400'
                  alt='Artist art'
                  class='
                    w-full aspect-square rounded-lg object-cover
                    shadow-lg group-hover:opacity-75 transition-opacity
                  '
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
                  {{ relatedArtist.songCount }} {{ relatedArtist.songCount === 1 ? 'song' : 'songs' }}
                </p>
              </div>
            </div>
          </Carousel>
        </div>
      </section>
    </section>

    <p v-if='!libraryLoading && !artist' class='text-center py-12 text-muted-foreground px-6'>
      Artist not found.
    </p>

    <ShareDialog
      v-if='artist'
      v-model:open='showShareDialog'
      :item-id='artist.id'
      :item-name='artist.name'
      :item-type="'artist'"
    />
  </div>
</template>

<style scoped>
@reference "tailwindcss";

.scrollbar-hide::-webkit-scrollbar {
  display: none;
}

.scrollbar-hide {
  -ms-overflow-style: none;
  scrollbar-width: none;
}

:deep(.album-art-image) {
  @apply w-full h-auto rounded-lg shadow-lg aspect-square object-cover transition-all;
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
