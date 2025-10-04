<script setup lang="ts">
  import Fuse from 'fuse.js'
  import { ArrowLeft, Edit, Heart, HeartOff, MoreHorizontal, Play, Shuffle, Trash2 } from 'lucide-vue-next'
  import { computed, onMounted, ref, watch } from 'vue'
  import { useRoute, useRouter } from 'vue-router'

  import { Playlist, Song, type UserData } from '@/bindings'
  import ImageLoader from '@/components/shared/ImageLoader.vue'
  import ImagePlaceholder from '@/components/shared/ImagePlaceholder.vue'
  import SongList from '@/components/shared/SongList.vue'
  import { Button } from '@/components/ui/button'
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
  import { useAuthStore, usePlayerStore, usePlaylistStore } from '@/stores'

  const route = useRoute()
  const router = useRouter()
  const playlistStore = usePlaylistStore()
  const authStore = useAuthStore()
  const playerStore = usePlayerStore()

  const playlistId = computed(() => route.params.playlistId as string)
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
    if (!playlistId.value) {
      console.error('No playlist ID provided')
      return
    }

    console.log('Loading playlist with ID:', playlistId.value)
    isLoading.value = true

    try {
      // Make sure playlists are loaded first
      if (playlistStore.playlists.length === 0) {
        console.log('No playlists in store, loading them first...')
        await playlistStore.loadPlaylists()
      }

      console.log('Available playlists:', playlistStore.playlists.map(p => ({ id: p.id, name: p.name })))

      // Find playlist in store
      const foundPlaylist = playlistStore.playlists.find(p => p.id === playlistId.value)
      if (!foundPlaylist) {
        console.error(`Playlist with ID ${playlistId.value} not found in store`)
        throw new Error('Playlist not found')
      }

      console.log('Found playlist:', foundPlaylist)
      playlist.value = foundPlaylist

      // Load playlist songs
      console.log('Loading playlist songs...')
      songs.value = await playlistStore.getPlaylistItems(playlistId.value)
      console.log('Loaded songs:', songs.value.length)
    } catch (error) {
      console.error('Failed to load playlist:', error)
      router.push('/playlists')
    } finally {
      isLoading.value = false
    }
  }

  const playPlaylist = async (shuffle = false): Promise<void> => {
    if (!playlistId.value) return
    await playlistStore.playPlaylist(playlistId.value, shuffle)
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
    router.push(`/playlists/${playlistId.value}/edit`)
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

  watch(() => playlistId.value, () => {
    loadPlaylist()
  })
</script>

<template>
  <div class='p-4 max-w-7xl mx-auto'>
    <!-- Header -->
    <div class='mb-8'>
      <Button @click='goBack' class='mb-4 gap-2' variant='ghost'>
        <ArrowLeft class='h-4 w-4' />
        Back to Playlists
      </Button>

      <div v-if='isLoading' class='flex items-center gap-6 mb-6'>
        <Skeleton class='w-48 h-48 rounded-lg' />
        <div class='flex-1 space-y-4'>
          <Skeleton class='h-8 w-64' />
          <Skeleton class='h-4 w-96' />
          <Skeleton class='h-6 w-24' />
        </div>
      </div>

      <div v-else-if='playlist' class='flex items-start gap-6 mb-6'>
        <ImageLoader
          :alt='`${playlist.name} playlist art`'
          :item-id='playlist.id'
          :server-url='authStore.serverUrl'
          :token='authStore.token'
          class='w-48 h-48 rounded-lg object-cover shadow-lg flex-shrink-0'
        >
          <template #fallback>
            <ImagePlaceholder
              class='w-48 h-48 rounded-lg shadow-lg'
              size='large'
              type='playlist'
            />
          </template>
        </ImageLoader>

        <div class='flex-1 min-w-0'>
          <div class='flex items-center gap-4 mb-2'>
            <h1 class='text-4xl font-bold truncate'>
              {{ playlist.name }}
            </h1>
            <Heart
              v-if='playlist.userData?.isFavorite'
              class='h-6 w-6 text-red-500 flex-shrink-0'
            />
          </div>

          <p v-if='playlist.description' class='text-muted-foreground mb-4'>
            {{ playlist.description }}
          </p>

          <p class='text-sm text-muted-foreground mb-6'>
            {{ songs.length }} songs
          </p>

          <div class='flex items-center gap-3'>
            <Button @click='playPlaylist()' class='gap-2'>
              <Play class='h-4 w-4' />
              Play
            </Button>
            <Button @click='playPlaylist(true)' class='gap-2' variant='outline'>
              <Shuffle class='h-4 w-4' />
              Shuffle
            </Button>

            <DropdownMenu>
              <DropdownMenuTrigger as-child>
                <Button size='icon' variant='outline'>
                  <MoreHorizontal class='h-4 w-4' />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent>
                <DropdownMenuItem @click='toggleFavorite'>
                  <Heart v-if='playlist.userData?.isFavorite' class='h-4 w-4 mr-2' />
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

      <Input
        v-model='searchQuery'
        class='max-w-sm focus-visible:ring-1 focus-visible:ring-accent border-0 focus-visible:border-accent'
        placeholder='Search songs in playlist...'
        type='text'
      />
    </div>

    <!-- Songs List -->
    <SongList
      v-if='!isLoading'
      :current-song='playerStore.currentSong'
      :is-playing='playerStore.isPlaying'
      :server-url='authStore.serverUrl'
      :show-album='true'
      :show-artist='true'
      :songs='filteredSongs'
      :token='""'
    />

    <div v-else class='space-y-4'>
      <Skeleton v-for='n in 10' :key='n' class='h-16 w-full' />
    </div>

    <div v-if='!isLoading && filteredSongs.length === 0' class='text-center py-12'>
      <p class='text-muted-foreground'>
        {{ songs.length === 0 ? 'This playlist is empty' : 'No songs match your search' }}
      </p>
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
