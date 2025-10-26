<script setup lang="ts">
  import { Edit, Heart, HeartOff, MoreHorizontal, Play, Shuffle, Trash2 } from 'lucide-vue-next'
  import { computed, onMounted, ref, watch } from 'vue'
  import { useRoute, useRouter } from 'vue-router'

  import { Playlist, Song, type UserData } from '@/bindings'
  import ImageLoader from '@/components/shared/ImageLoader.vue'
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
  import { Skeleton } from '@/components/ui/skeleton'
  import { useSongInteractions } from '@/composables/useSongInteractions'
  import { logger } from '@/lib/logger'
  import { useAuthStore } from '@/stores'
  import { usePlaylistStore } from '@/stores/playlists'

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
  const showDeleteDialog = ref(false)

  const createDefaultUserData = (): UserData => ({
    isFavorite:            false,
    lastPlayedDate:        null,
    playbackPositionTicks: BigInt(0),
    playCount:             0,
    played:                false,
  })

  const loadPlaylist = async (): Promise<void> => {
    if (!id.value) {
      logger.error('No playlist ID provided')
      return
    }

    isLoading.value = true

    try {
      if (playlistStore.playlists.length === 0) {
        await playlistStore.loadPlaylists()
      }

      const foundPlaylist = playlistStore.playlists.find(p => p.id === id.value)
      if (!foundPlaylist) {
        throw new Error('Playlist not found')
      }

      playlist.value = foundPlaylist
      songs.value = await playlistStore.getPlaylistItems(id.value)
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

  onMounted(() => {
    loadPlaylist()
  })

  watch(() => id.value, () => {
    loadPlaylist()
  })
</script>

<template>
  <div>
    <div v-if='isLoading || !playlist'>
      <!-- Immersive Header Skeleton -->
      <div
        :style="{
          minHeight: '400px',
          marginBottom: 'calc(-env(safe-area-inset-top) + 2rem)',
          position: 'relative',
          top: '-env(safe-area-inset-top)',
        }"
        class='relative isolate bg-sidebar -mx-4 -mt-4 overflow-hidden'
      >
        <div class='absolute inset-0 bg-secondary/50' />
        <div
          class='
            absolute bottom-0 left-0 right-0 h-32 bg-linear-to-t
            from-background via-background/80 to-transparent
          '
        />
        <!-- Skeleton Content -->
        <div class='z-10 absolute bottom-0 left-0 right-0 flex flex-col p-4'>
          <div class='flex-1 min-w-0 text-left'>
            <Skeleton class='h-10 w-3/4 mb-3' />
            <Skeleton class='h-5 w-2/4 mb-2' />
            <Skeleton class='h-5 w-1/4' />
          </div>
          <div class='flex items-center justify-between mt-4'>
            <div class='flex items-center gap-2'>
              <Skeleton class='size-12 rounded-full' />
              <Skeleton class='size-12 rounded-full' />
            </div>
            <Skeleton class='size-12 rounded-full' />
          </div>
        </div>
      </div>

      <!-- Content Skeleton -->
      <div class='p-4 space-y-4'>
        <div v-for='n in 10' :key='`song-skeleton-${n}`' class='flex items-center gap-4'>
          <Skeleton class='size-12 rounded-lg' />
          <div class='flex-1 space-y-2'>
            <Skeleton class='h-5 w-3/4' />
            <Skeleton class='h-4 w-1/2' />
          </div>
        </div>
      </div>
    </div>

    <div v-else class='p-4 space-y-6'>
      <!-- Featured Playlist Section -->
      <div
        :style="{
          minHeight: '400px',
          marginBottom: 'calc(-env(safe-area-inset-top) + 2rem)',
          position: 'relative',
          top: '-env(safe-area-inset-top)',
        }"
        class='relative isolate bg-sidebar -mx-4 -mt-4 overflow-hidden'
      >
        <!-- Background Image -->
        <div class='absolute bg-cover bg-center bg-no-repeat -top-4 inset-0'>
          <ImageLoader
            :item-id='playlist.id'
            :server-url='authStore.serverUrl'
            :token='authStore.token'
            alt='Playlist art'
            class='size-full object-cover'
          />
          <div class='absolute inset-0 bg-black/50' />
          <div
            class='
              absolute bottom-0 left-0 right-0 h-32 bg-linear-to-t
              from-background via-background/80 to-transparent
            '
          />
        </div>

        <!-- Content -->
        <div class='z-10 absolute bottom-0 left-0 right-0 flex flex-col p-4'>
          <div class='flex-1 min-w-0 text-left'>
            <h1 class='text-4xl font-bold mb-2 text-white drop-shadow-lg truncate select-text'>
              {{ playlist.name }}
            </h1>
            <p v-if='playlist.description' class='text-base text-white/80 mb-3 drop-shadow-md line-clamp-2'>
              {{ playlist.description }}
            </p>
            <p class='text-sm text-white/80 drop-shadow-md'>
              {{ songs.length }} song{{ songs.length !== 1 ? 's' : '' }}
            </p>
          </div>

          <!-- Actions -->
          <div class='flex items-center justify-between mt-4'>
            <div class='flex items-center gap-2'>
              <Button
                @click='playPlaylist()'
                class='bg-white/20 hover:bg-white/30 backdrop-blur-sm text-white rounded-full border border-white/20'
                size='icon-lg'
                variant='ghost'
              >
                <Play class='size-5' />
              </Button>
              <Button
                @click='playPlaylist(true)'
                class='bg-white/20 hover:bg-white/30 backdrop-blur-sm text-white rounded-full border border-white/20'
                size='icon-lg'
                variant='ghost'
              >
                <Shuffle class='size-5' />
              </Button>
            </div>
            <DropdownMenu>
              <DropdownMenuTrigger as-child>
                <Button
                  class='bg-white/20 hover:bg-white/30 backdrop-blur-sm text-white rounded-full border border-white/20'
                  size='icon-lg'
                  variant='ghost'
                >
                  <MoreHorizontal class='size-5' />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align='end'>
                <DropdownMenuItem @click='toggleFavorite'>
                  <Heart v-if='playlist.userData?.isFavorite' class='h-4 w-4 mr-2 fill-current' />
                  <HeartOff v-else class='h-4 w-4 mr-2' />
                  {{ playlist.userData?.isFavorite ? 'Unfavorite' : 'Favorite' }}
                </DropdownMenuItem>
                <DropdownMenuItem @click='() => editPlaylist()'>
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

      <!-- Songs List -->
      <SongList
        @play-instant-mix='playInstantMix'
        @play-song='playSongWithQueue'
        @toggle-favorite='toggleSongFavorite'
        :server-url='authStore.serverUrl'
        :show-album='true'
        :show-album-art='true'
        :show-artist='true'
        :show-duration='true'
        :songs='songs'
        :token='authStore.token'
      />

      <div v-if='songs.length === 0' class='text-center py-12'>
        <p class='text-muted-foreground text-lg'>
          This playlist is empty
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
