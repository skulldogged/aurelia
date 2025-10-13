<script setup lang="ts">
  import { computed, onMounted, onUnmounted, ref } from 'vue'
  import { useRouter } from 'vue-router'

  import { Album, Artist, Song } from '@/bindings'
  import { ScrollArea } from '@/components/ui/scroll-area'
  import { albumMatchesQuery, artistMatchesQuery, songMatchesQuery } from '@/lib/transforms'

  import ImageLoader from './ImageLoader.vue'

  const props =   defineProps<{
    albums:             Album[]
    artists:            Artist[]
    isSidebarCollapsed: boolean
    isVisible:          boolean
    query:              string
    serverUrl?:         string
    songs:              Song[]
    token?:             string
  }>()

  const emit = defineEmits<{
    'close':          []
    'play-song':      [song: Song]
    'result-clicked': []
    'select-album':   [album: Album]
    'select-artist':  [artist: Artist]
  }>()

  const router = useRouter()
  const searchResultsRef = ref<HTMLElement | null>(null)

  const handleClickOutside = (event: Event): void => {
    const target = event.target as HTMLElement
    if (searchResultsRef.value && !searchResultsRef.value.contains(target)) {
      // Check if we're not clicking on the search input
      const searchInput = document.querySelector('input[placeholder="Search music..."]')
      if (searchInput && !searchInput.contains(target))
        emit('close')
    }
  }

  onMounted(() => {
    document.addEventListener('click', handleClickOutside)
  })

  onUnmounted(() => {
    document.removeEventListener('click', handleClickOutside)
  })

  const filteredSongs = computed(() =>
    props.query.length >= 2
      ? props.songs.filter(song => songMatchesQuery(props.query, song)).slice(0, 5)
      : [],
  )

  const filteredAlbums = computed(() =>
    props.query.length >= 2
      ? props.albums.filter(album => albumMatchesQuery(props.query, album)).slice(0, 5)
      : [],
  )

  const filteredArtists = computed(() =>
    props.query.length >= 2
      ? props.artists.filter(artist => artistMatchesQuery(props.query, artist))
      : [],
  )

  const hasResults = computed(() =>
    filteredSongs.value.length > 0 ||
    filteredAlbums.value.length > 0 ||
    filteredArtists.value.length > 0,
  )

  const resultOrder = computed(() => ['songs', 'albums', 'artists'])

  const selectSong = (song: Song): void => {
    emit('play-song', song)
    emit('close')
    emit('result-clicked')
  }

  const selectAlbum = (album: Album): void => {
    router.push(`/songs/album/${encodeURIComponent(album.name)}`)
    emit('close')
    emit('result-clicked')
  }

  const selectArtist = (artist: Artist): void => {
    router.push(`/songs/artist/${artist.id}`)
    emit('close')
    emit('result-clicked')
  }
</script>

<template>
  <div
    v-if='isVisible && query'
    ref='searchResultsRef'
    :class="[
      'absolute top-12 bg-background border border-border rounded-md shadow-lg z-50 w-96',
      isSidebarCollapsed ? 'left-2' : 'left-2'
    ]"
  >
    <ScrollArea class='h-[400px]'>
      <div v-if='hasResults' class='p-2'>
        <template v-for='(section, index) in resultOrder' :key='section'>
          <div v-if="section === 'songs' && filteredSongs.length > 0" :class="{ 'mt-2': index > 0 }">
            <h3 class='text-sm font-semibold text-muted-foreground px-2 py-1.5'>
              Songs
            </h3>
            <ul>
              <li
                v-for='song in filteredSongs'
                @click='selectSong(song)'
                :key='song.id'
                class='flex items-center p-2 rounded-md hover:bg-accent/20 cursor-pointer'
              >
                <ImageLoader
                  v-if='serverUrl && token'
                  :item-id='song.id'
                  :server-url='serverUrl'
                  :token='token'
                  class='size-10 rounded-md mr-3'
                />
                <div>
                  <p class='font-semibold'>
                    {{ song.name }}
                  </p>
                  <p class='text-sm text-muted-foreground'>
                    {{ song.artists?.join(', ') }}
                  </p>
                </div>
              </li>
            </ul>
          </div>
          <div v-if="section === 'albums' && filteredAlbums.length > 0" :class="{ 'mt-2': index > 0 }">
            <h3 class='text-sm font-semibold text-muted-foreground px-2 py-1.5'>
              Albums
            </h3>
            <ul>
              <li
                v-for='album in filteredAlbums'
                @click='selectAlbum(album)'
                :key='album.name'
                class='flex items-center p-2 rounded-md hover:bg-accent/20 cursor-pointer'
              >
                <ImageLoader
                  v-if='serverUrl && token'
                  :item-id='album.id || album.name'
                  :server-url='serverUrl'
                  :token='token'
                  class='size-10 rounded-md mr-3'
                />
                <div>
                  <p class='font-semibold'>
                    {{ album.name }}
                  </p>
                  <p class='text-sm text-muted-foreground'>
                    {{ album.artist }}
                  </p>
                </div>
              </li>
            </ul>
          </div>
          <div v-if="section === 'artists' && filteredArtists.length > 0" :class="{ 'mt-2': index > 0 }">
            <h3 class='text-sm font-semibold text-muted-foreground px-2 py-1.5'>
              Artists
            </h3>
            <ul>
              <li
                v-for='artist in filteredArtists'
                @click='selectArtist(artist)'
                :key='artist.id'
                class='flex items-center p-2 rounded-md hover:bg-accent/20 cursor-pointer'
              >
                <div class='size-10 rounded-full bg-muted flex items-center justify-center mr-3'>
                  <span class='text-lg font-bold'>{{ artist.name.charAt(0) }}</span>
                </div>
                <p class='font-semibold'>
                  {{ artist.name }}
                </p>
              </li>
            </ul>
          </div>
        </template>
      </div>
      <div v-else class='p-4 text-center text-muted-foreground'>
        No results found for "{{ query }}"
      </div>
    </ScrollArea>
  </div>
</template>
