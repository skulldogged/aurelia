<script setup lang="ts">
  import Fuse from 'fuse.js'
  import { ArrowLeft, Edit, Heart, HeartOff, MoreHorizontal, Play, Shuffle, Trash2 } from 'lucide-vue-next'
  import { computed, onMounted, ref, watch } from 'vue'
  import { useRoute, useRouter } from 'vue-router'

  import { Playlist, Song, type UserData } from '@/bindings'
  import ImageLoader from '@/components/shared/ImageLoader.vue'
  import ImagePlaceholder from '@/components/shared/ImagePlaceholder.vue'
  import SongList from '@/components/shared/SongList.vue'
  import Button from '@/components/ui/Button.vue'
  import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
  } from '@/components/ui/dialog'
  import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
  } from '@/components/ui/dropdown-menu'
  import { Input } from '@/components/ui/input'
  import { Skeleton } from '@/components/ui/skeleton'
  import { useSongInteractions } from '@/composables/useSongInteractions'
  import { logger } from '@/lib/logger'
  import { useAuthStore, usePlaylistStore } from '@/stores'

  const route = useRoute()
  const router = useRouter()
  const playlistStore = usePlaylistStore()
  const authStore = useAuthStore()

  const credentials = computed(() => ({
    serverUrl: authStore.serverUrl,
    token:     authStore.token,
    userId:    authStore.userId,
    username:  authStore.username,
  }))

  const { playInstantMix, playSongs, toggleFavorite: toggleSongFavorite } = useSongInteractions(credentials)

  const id = computed(() => {
    const params = route.params
    if ('id' in params) {
      const param = params.id
      if (typeof param === 'string') return param
      if (Array.isArray(param)) return param[0] ?? ''
    }
    return ''
  })
  const playlist = ref<null | Playlist>(null)
  const songs = ref<Song[]>([])
  const isLoading = ref(false)
  const searchQuery = ref('')
  const showDeleteDialog = ref(false)

  const createDefaultUserData = (): UserData => ({
    isFavorite:            false,
    lastPlayedDate:        null,
    playbackPositionTicks: BigInt(0),
    playCount:             0,
    played:                false,
  })

  const songsFuse = ref(new Fuse(songs.value, {
    includeScore: true,
    keys:         [
      { name: 'name', weight: 0.6 },
      { name: 'artists', weight: 0.4 },
    ],
    minMatchCharLength: 2,
    threshold:          0.2,
  }))

  watch(() => songs.value, newSongs => {
    songsFuse.value.setCollection(newSongs)
  })

  const filteredSongs = computed(() =>
    searchQuery.value && searchQuery.value.length >= 2
      ? songsFuse.value.search(searchQuery.value).map(result => result.item)
      : songs.value,
  )

  const loadPlaylist = async (): Promise<void> => {
    if (!id.value) {
      logger.error('No playlist ID provided')
      return
    }

    logger.info('Loading playlist with ID:', id.value)
    isLoading.value = true

    try {
      // Make sure playlists are loaded first
      if (playlistStore.playlists.length === 0) {
        logger.info('No playlists in store, loading them first...')
        await playlistStore.loadPlaylists()
      }

      logger.info('Available playlists:', playlistStore.playlists.map(p => ({ id: p.id, name: p.name })))

      // Find playlist in store
      const foundPlaylist = playlistStore.playlists.find(p => p.id === id.value)
      if (!foundPlaylist) {
        logger.error(`Playlist with ID ${id.value} not found in store`)
        throw new Error('Playlist not found')
      }

      logger.info('Found playlist:', foundPlaylist)
      playlist.value = foundPlaylist

      // Load playlist songs
      logger.info('Loading playlist songs...')
      songs.value = await playlistStore.getPlaylistItems(id.value)
      logger.info('Loaded songs:', songs.value.length)
    } catch (error) {
      logger.error('Failed to load playlist:', error)
      router.push('/playlists')
    } finally {
      isLoading.value = false
    }
  }

  const playPlaylist = async (shuffle = false): Promise<void> => {
    if (!id.value) return
    await playlistStore.playPlaylist(id.value, shuffle)
  }

  const playSongWithQueue = (song: Song): void => {
    const songIndex = songs.value.findIndex(s => s.id === song.id)
    if (songIndex === -1) return

    // Queue current song and all songs after it
    const songsToQueue = songs.value.slice(songIndex)
    playSongs(songsToQueue)
  }

  const toggleFavorite = async (): Promise<void> => {
    if (!playlist.value) return
    const success = await playlistStore.togglePlaylistFavorite(playlist.value.id)
    if (!success) return

    const currentUserData = playlist.value.userData ?? createDefaultUserData()

    playlist.value = {
      ...playlist.value,
      userData: {
        ...currentUserData,
        isFavorite: !currentUserData.isFavorite,
      },
    }
  }

  const editPlaylist = (): void => {
    router.push(`/playlists/${id.value}/edit`)
  }

  const deletePlaylist = (): void => {
    showDeleteDialog.value = true
  }

  const confirmDelete = async (): Promise<void> => {
    if (!playlist.value) return
    await playlistStore.deletePlaylist(playlist.value.id)
    showDeleteDialog.value = false
    router.push('/playlists')
  }

  const cancelDelete = (): void => {
    showDeleteDialog.value = false
  }

  const goBack = (): void => {
    router.push('/playlists')
  }

  onMounted(() => {
    loadPlaylist()
  })

  watch(() => id.value, () => {
    loadPlaylist()
  })
</script>

<template>
  <div class='p-4 max-w-7xl mx-auto space-y-8'>
    <!-- Back Button -->
    <Button @click='goBack' class='gap-2' variant='ghost'>
      <ArrowLeft class='h-4 w-4' />
      Back to Playlists
    </Button>

    <!-- Loading Skeleton -->
    <div v-if='isLoading' class='space-y-8'>
      <div class='flex items-center space-x-6 p-8 bg-sidebar rounded-lg'>
        <Skeleton class='size-48 rounded-lg shrink-0' />
        <div class='flex-1 space-y-4'>
          <Skeleton class='h-12 w-3/4' />
          <Skeleton class='h-6 w-1/2' />
          <Skeleton class='h-6 w-32' />
          <div class='flex gap-2'>
            <Skeleton class='h-10 w-24' />
            <Skeleton class='h-10 w-28' />
            <Skeleton class='h-10 w-10' />
          </div>
        </div>
      </div>
    </div>

    <!-- Playlist Header -->
    <div v-else-if='playlist' class='space-y-8'>
      <div class='flex items-center space-x-6 p-8 bg-sidebar rounded-lg'>
        <div class='shrink-0'>
          <ImageLoader
            :alt='`${playlist.name} playlist art`'
            :item-id='playlist.id'
            :server-url='authStore.serverUrl'
            :token='authStore.token'
            class='size-48 rounded-lg object-cover'
          >
            <template #fallback>
              <ImagePlaceholder
                class='size-48 rounded-lg'
                size='large'
                type='playlist'
              />
            </template>
          </ImageLoader>
        </div>

        <div class='flex-1 min-w-0'>
          <div class='flex items-center gap-3 mb-2'>
            <h1 class='text-5xl font-bold text-foreground truncate'>
              {{ playlist.name }}
            </h1>
            <Heart
              v-if='playlist.userData?.isFavorite'
              class='h-7 w-7 text-red-500 shrink-0 fill-current'
            />
          </div>

          <p v-if='playlist.description' class='text-xl text-muted-foreground mt-2 mb-4'>
            {{ playlist.description }}
          </p>

          <div class='flex items-center gap-2 text-sm text-muted-foreground mt-3'>
            <span>{{ songs.length }} song{{ songs.length !== 1 ? 's' : '' }}</span>
          </div>

          <!-- Actions -->
          <div class='flex items-center gap-3 mt-6'>
            <Button @click='playPlaylist()' class='gap-2' size='lg'>
              <Play class='h-5 w-5' />
              Play
            </Button>

            <DropdownMenu>
              <DropdownMenuTrigger as-child>
                <Button size='lg' variant='outline'>
                  <MoreHorizontal class='h-5 w-5' />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align='start'>
                <DropdownMenuItem @click='playPlaylist(true)'>
                  <Shuffle class='h-4 w-4 mr-2' />
                  Shuffle
                </DropdownMenuItem>
                <DropdownMenuItem @click='toggleFavorite'>
                  <Heart v-if='playlist.userData?.isFavorite' class='h-4 w-4 mr-2 fill-current' />
                  <HeartOff v-else class='h-4 w-4 mr-2' />
                  {{ playlist.userData?.isFavorite ? 'Unfavorite' : 'Favorite' }}
                </DropdownMenuItem>
                <DropdownMenuItem @click='editPlaylist'>
                  <Edit class='h-4 w-4 mr-2' />
                  Edit Playlist
                </DropdownMenuItem>
                <DropdownMenuItem @click='deletePlaylist' class='text-destructive'>
                  <Trash2 class='h-4 w-4 mr-2' />
                  Delete Playlist
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
        </div>
      </div>

      <!-- Search -->
      <div class='flex items-center justify-between'>
        <h2 class='text-2xl font-semibold text-foreground'>
          Songs
        </h2>
        <Input
          v-model='searchQuery'
          class='max-w-sm h-11 focus-visible:ring-1 focus-visible:ring-accent border
                 focus-visible:border-accent'
          placeholder='Search songs...'
          type='text'
        />
      </div>

      <!-- Songs List -->
      <SongList
        @play-instant-mix='playInstantMix'
        @play-song='playSongWithQueue'
        @toggle-favorite='toggleSongFavorite'
        :server-url='authStore.serverUrl'
        :show-add-button='false'
        :show-album='true'
        :show-album-art='true'
        :show-artist='true'
        :show-duration='true'
        :songs='filteredSongs'
        :token='authStore.token'
        layout='comfy'
      />

      <div v-if='filteredSongs.length === 0' class='text-center py-12'>
        <p class='text-muted-foreground text-lg'>
          {{ songs.length === 0 ? 'This playlist is empty' : 'No songs match your search' }}
        </p>
      </div>
    </div>

    <!-- Delete Confirmation Dialog -->
    <Dialog v-model:open='showDeleteDialog'>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Delete Playlist</DialogTitle>
          <DialogDescription>
            Are you sure you want to delete "{{ playlist?.name }}"? This action cannot be undone.
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button @click='cancelDelete' variant='outline'>
            Cancel
          </Button>
          <Button @click='confirmDelete' variant='destructive'>
            Delete
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </div>
</template>