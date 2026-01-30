<script setup lang="ts">
  import { MoreHorizontal, Music, Play, Share2, Shuffle } from 'lucide-vue-next'
  import { computed, inject, onMounted, onUnmounted, ref, watch } from 'vue'
  import { useRoute } from 'vue-router'

  import { Album, NameIdPair, Song } from '@/lib/api/bindings'
  import AddToPlaylistMenu from '@/components/shared/AddToPlaylistMenu.vue'
  import AlbumTrackList from '@/components/shared/AlbumTrackList.vue'
  import ImageLoader from '@/components/shared/ImageLoader.vue'
  import ShareDialog from '@/components/shared/ShareDialog.vue'
  import Button from '@/components/ui/Button.vue'
  import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
  } from '@/components/ui/dropdown-menu'
  import { Skeleton } from '@/components/ui/skeleton'
  import { scrollElementKey } from '@/composables/useMainLayout'
  import { useAuthStore } from '@/stores/auth'
  import { useLibraryStore } from '@/stores/library'

  const emit = defineEmits<{
    'play-instant-mix': [song: Song]
    'play-songs':       [songs: Song[]]
    'toggle-favorite':  [song: Song]
  }>()

  const authStore = useAuthStore()
  const libraryStore = useLibraryStore()

  // Create computed properties from stores
  const allAlbums = computed(() => libraryStore.allAlbums as Album[])
  const libraryLoaded = computed(() => libraryStore.isLoaded)
  const libraryLoading = computed(() => libraryStore.isLoading)
  const serverUrl = computed(() => authStore.serverUrl)
  const token = computed(() => authStore.token)

  const route = useRoute()
  const showShareDialog = ref(false)

  const id = computed(() => {
    const params = route.params
    if ('id' in params) {
      const param = params.id
      if (typeof param === 'string') return param
      if (Array.isArray(param)) return param[0] ?? ''
    }
    return ''
  })

  const album = computed(() =>
    id.value
    && libraryLoaded.value
    && allAlbums.value.length > 0
      ? allAlbums.value.find(a => a.id === id.value) || null
      : null,
  )

  const albumSongs = computed(() =>
    album.value
      ? [...album.value.songs || []].sort((a, b) => {
          // Sort by disc number first, then by track number
          const discA = a.discNumber ?? 1
          const discB = b.discNumber ?? 1
          if (discA !== discB) {
            return discA - discB
          }
          return (a.trackNumber ?? 0) - (b.trackNumber ?? 0)
        })
      : [],
  )

  // Unique album artists aggregated from songs
  const albumArtistPairs = computed<NameIdPair[]>(() => {
    const idToName = new Map<string, string>()

    for (const song of albumSongs.value)
      if (song.albumArtists)
        for (const pair of song.albumArtists)
          if (pair.id && pair.name) idToName.set(pair.id, pair.name)

    // Fallbacks if albumArtists are not provided by backend
    if (idToName.size === 0) {
      const first = albumSongs.value[0]
      if (first?.artistIds && first.artists && first.artistIds.length === first.artists.length) {
        first.artistIds.forEach((id, idx) => {
          const name = first.artists![idx]
          if (id && name) idToName.set(id, name)
        })
      } else if (album.value?.artist && album.value.artistId) {
        idToName.set(album.value.artistId, album.value.artist)
      }
    }

    return Array.from(idToName, ([id, name]) => ({ id, name }))
  })

  // Determine if any song in the album has multiple artists
  const hasMultipleArtists = computed(() =>
    albumSongs.value.length > 1 &&
    albumSongs.value.some(song =>
      song.artists?.length &&
      song.artists.length > 1,
    ),
  )

  // Aggregated album metadata
  const albumYear = computed(() => albumSongs.value.find(s => s.year)?.year || null)
  const totalDurationSec = computed(() => albumSongs.value.reduce((acc, s) => acc + (s.duration || 0), 0))
  const formattedTotalDuration = computed(() => {
    const total = totalDurationSec.value
    const hours = Math.floor(total / 3600)
    const minutes = Math.floor((total % 3600) / 60)
    if (hours > 0) return `${hours} hr ${minutes} min`
    return `${minutes} min`
  })

  const albumGenres = computed(() => {
    const set = new Set<string>()

    for (const song of albumSongs.value)
      if (song.genres)
        for (const g of song.genres)
          set.add(g)

    return Array.from(set)
  })

  const playAll = (): void => {
    if (albumSongs.value.length > 0)
      emit('play-songs', albumSongs.value)
  }

  const playSongWithQueue = (song: Song): void => {
    const songIndex = albumSongs.value.findIndex(s => s.id === song.id)
    if (songIndex === -1) return

    // Queue current song and all songs after it
    const songsToQueue = albumSongs.value.slice(songIndex)
    emit('play-songs', songsToQueue)
  }

  const shuffleAll = (): void => {
    const songs = [...albumSongs.value]
    for (let i = songs.length - 1; i > 0; i -= 1) {
      const j = Math.floor(Math.random() * (i + 1))
      const temp = songs[i]
      songs[i] = songs[j]
      songs[j] = temp
    }
    if (songs.length > 0) emit('play-songs', songs)
  }

  // Scroll tracking for sticky header
  const scrollElement = inject(scrollElementKey, ref<HTMLElement | null>(null))
  const heroRef = ref<HTMLElement | null>(null)
  const scrollY = ref(0)
  const heroHeight = ref(400)

  // Show sticky header when scrolled past 80% of hero
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
      v-if='album'
      :class="[
        'sticky top-0 z-40 overflow-hidden',
        'bg-sidebar/95 backdrop-blur-md border-b border-border/20 shadow-md',
        'transition-all duration-200 ease-out',
      ]"
      :style="{ height: showStickyHeader ? '64px' : '0px' }"
    >
      <div class='h-16 flex items-center gap-4 px-6 md:px-10 lg:px-16 max-w-7xl mx-auto'>
        <!-- Album Art Thumbnail -->
        <div class='shrink-0'>
          <ImageLoader
            :item-id='album.id || album.name'
            :server-url='serverUrl'
            :token='token'
            :width='100'
            class='size-10 rounded-lg shadow-md object-cover'
          >
            <template #fallback>
              <div class='size-10 rounded-lg bg-muted flex items-center justify-center'>
                <Music class='size-5 text-muted-foreground' />
              </div>
            </template>
          </ImageLoader>
        </div>

        <!-- Album Info -->
        <div class='flex-1 min-w-0'>
          <h2 class='font-bold text-foreground truncate'>
            {{ album.name }}
          </h2>
          <p class='text-sm text-muted-foreground truncate'>
            {{ albumArtistPairs.map(p => p.name).join(', ') || album.artist || 'Unknown Artist' }}
          </p>
        </div>

        <!-- Play Button -->
        <Button
          @click='playAll'
          class='shrink-0'
          size='sm'
        >
          <Play class='size-4 fill-current' />
          <span class='ml-1.5'>Play</span>
        </Button>
      </div>
    </div>

    <!-- Hero Section -->
    <section
      ref='heroRef'
      v-if='album || libraryLoading'
      class='
        relative isolate overflow-hidden min-h-[400px]
        bg-linear-to-b from-sidebar via-sidebar to-background
      '
    >
      <div class='absolute inset-0 overflow-hidden'>
        <div class='absolute inset-0 opacity-20'>
          <ImageLoader
            v-if='album && !libraryLoading'
            :item-id='album.id || album.name'
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

      <div class='relative z-10 flex flex-col items-center py-12'>
        <div class='w-full max-w-7xl space-y-8 px-6 md:px-10 lg:px-16'>
          <div class='flex items-start justify-between gap-8 lg:gap-12'>
            <div class='flex-1 min-w-0 space-y-6'>
              <template v-if='libraryLoading'>
                <Skeleton class='h-12 w-3/4 rounded-lg' />
                <Skeleton class='h-8 w-1/2 rounded-lg' />
                <Skeleton class='h-5 w-2/3 rounded-lg' />
                <div class='flex gap-3 pt-2'>
                  <Skeleton class='h-10 w-32 rounded-lg' />
                  <Skeleton class='h-10 w-32 rounded-lg' />
                </div>
              </template>
              <template v-else-if='album'>
                <h1 class='text-5xl lg:text-6xl font-black text-white'>
                  {{ album.name }}
                </h1>

                <p class='text-lg text-white/90 font-semibold'>
                  <template v-if='albumArtistPairs.length'>
                    <template v-for='(pair, index) in albumArtistPairs' :key='pair.id'>
                      <RouterLink
                        :to='`/artists/${pair.id}`'
                        class='hover:text-accent transition-colors'
                      >
                        {{ pair.name }}
                      </RouterLink>
                      <span v-if='index < albumArtistPairs.length - 1'>, </span>
                    </template>
                  </template>
                  <template v-else>
                    {{ album?.artist || 'Unknown Artist' }}
                  </template>
                </p>

                <div class='flex flex-wrap gap-4 items-center text-sm text-white/70'>
                  <span v-if='albumYear'>{{ albumYear }}</span>
                  <div v-if='albumYear && albumSongs.length' class='w-px bg-white/10' />
                  <span v-if='albumSongs.length'>
                    {{ albumSongs.length }} song{{ albumSongs.length > 1 ? 's' : '' }}
                  </span>
                  <div v-if='albumSongs.length' class='w-px bg-white/10' />
                  <span v-if='albumSongs.length'>{{ formattedTotalDuration }}</span>
                </div>

                <div v-if='albumGenres.length > 0' class='flex flex-wrap gap-2'>
                  <span
                    v-for='genre in albumGenres.slice(0, 5)'
                    :key='genre'
                    class='px-3 py-1 bg-white/10 text-white text-xs font-semibold rounded-full border border-white/20'
                  >
                    {{ genre }}
                  </span>
                </div>

                <div class='flex items-center gap-3 pt-2'>
                  <button
                    @click='playAll'
                    class='
                      px-6 py-3 bg-accent hover:bg-accent/90 text-sidebar font-bold rounded-lg
                      transition-all duration-200 flex items-center gap-2 shadow-lg
                      hover:shadow-xl
                    '
                  >
                    <Play class='h-5 w-5 fill-current' />
                    <span>Play</span>
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
                      <DropdownMenuItem @click='shuffleAll'>
                        <Shuffle class='size-4 mr-2' />
                        Shuffle
                      </DropdownMenuItem>
                      <AddToPlaylistMenu :songs='albumSongs' type='dropdown' />
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
              <template v-if='libraryLoading'>
                <Skeleton class='w-64 h-64 rounded-2xl' />
              </template>
              <template v-else-if='album'>
                <div class='relative group'>
                  <div
                    class='
                      absolute -inset-4 rounded-3xl blur-xl opacity-0
                      group-hover:opacity-100 transition-opacity duration-300
                      bg-linear-to-br from-accent/30 to-accent/10
                    '
                  />

                  <ImageLoader
                    :item-id='album.id || album.name'
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
        </div>
      </div>
    </section>

    <section class='flex justify-center'>
      <div class='w-full max-w-7xl py-6 px-6 md:px-10 lg:px-16'>
        <AlbumTrackList
          @play-instant-mix='$emit("play-instant-mix", $event)'
          @play-song='playSongWithQueue'
          @toggle-favorite='(song) => $emit("toggle-favorite", song)'
          :loading='libraryLoading || !libraryLoaded || !album'
          :server-url='serverUrl'
          :show-artist='hasMultipleArtists'
          :songs='albumSongs'
          :token='token'
        />
      </div>
    </section>

    <p v-if='!libraryLoading && !album' class='text-center py-12 text-muted-foreground px-6'>
      Album not found.
    </p>

    <ShareDialog
      v-if='album'
      v-model:open='showShareDialog'
      :item-id='album.id || ""'
      :item-name='album.name'
      :item-type="'album'"
    />
  </div>
</template>