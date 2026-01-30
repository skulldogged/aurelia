<script setup lang="ts">
  import { Edit, Heart, HeartOff, MoreHorizontal, Play, Shuffle, Trash2 } from 'lucide-vue-next'
  import { computed, onMounted, ref, watch } from 'vue'
  import { useRoute, useRouter } from 'vue-router'

  import { getApiClient, Playlist, Song, type UserData } from '../../index'
  import ImageLoader from '../../components/shared/ImageLoader.vue'
  import ImagePlaceholder from '../../components/shared/ImagePlaceholder.vue'
  import SongList from '../../components/shared/SongList.vue'
  import Button from '../../components/ui/Button.vue'
  import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
  } from '../../components/ui/dialog'
  import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
  } from '../../components/ui/dropdown-menu'
  import { Skeleton } from '../../components/ui/skeleton'
  import { useSongInteractions } from '../../composables/useSongInteractions'
  import { logger } from '../../lib/logger'
  import { useAuthStore } from '../../stores'
  import { usePlaylistStore } from '../../stores/playlists'

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

  onMounted(() => {
    loadPlaylist()
  })

  watch(() => id.value, () => {
    loadPlaylist()
  })
</script>

<template>
  <div class='flex flex-col'>
    <section
      v-if='playlist || isLoading'
      class='
        relative isolate overflow-hidden min-h-[400px]
        bg-linear-to-b from-sidebar via-sidebar to-background
      '
    >
      <div class='absolute inset-0 overflow-hidden'>
        <div class='absolute inset-0 opacity-20'>
          <ImageLoader
            v-if='playlist && !isLoading'
            :item-id='playlist.id'
            :server-url='authStore.serverUrl'
            :token='authStore.token'
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
              <template v-if='isLoading'>
                <Skeleton class='h-12 w-3/4 rounded-lg' />
                <Skeleton class='h-8 w-1/2 rounded-lg' />
                <Skeleton class='h-5 w-2/3 rounded-lg' />
                <div class='flex gap-3 pt-2'>
                  <Skeleton class='h-10 w-32 rounded-lg' />
                  <Skeleton class='h-10 w-32 rounded-lg' />
                </div>
              </template>
              <template v-else-if='playlist'>
                <div class='flex items-center gap-3'>
                  <h1 class='text-5xl lg:text-6xl font-black text-white truncate'>
                    {{ playlist.name }}
                  </h1>
                  <Heart
                    v-if='playlist.userData?.isFavorite'
                    class='h-8 w-8 text-red-500 shrink-0 fill-current'
                  />
                </div>

                <p v-if='playlist.description' class='text-lg text-white/90 font-semibold'>
                  {{ playlist.description }}
                </p>

                <div class='flex items-center gap-2 text-sm text-white/70'>
                  <span>{{ songs.length }} song{{ songs.length !== 1 ? 's' : '' }}</span>
                </div>

                <div class='flex items-center gap-3 pt-2'>
                  <button
                    @click='playPlaylist()'
                    class='
                      px-6 py-3 bg-accent hover:bg-accent/90 text-sidebar font-bold rounded-lg
                      transition-all duration-200 flex items-center gap-2 shadow-lg
                      hover:shadow-xl
                    '
                  >
                    <Play class='h-5 w-5 fill-current' />
                    <span>Play</span>
                  </button>
                  <button
                    @click='playPlaylist(true)'
                    class='
                      px-6 py-3 bg-white/10 hover:bg-white/20 text-white font-semibold rounded-lg
                      border border-white/20 transition-all duration-200 flex items-center gap-2
                      backdrop-blur-sm hover:backdrop-blur-md
                    '
                  >
                    <Shuffle class='h-5 w-5' />
                    <span>Shuffle</span>
                  </button>

                  <DropdownMenu>
                    <DropdownMenuTrigger as-child>
                      <button
                        class='
                          px-4 py-3 bg-white/10 hover:bg-white/20 text-white rounded-lg
                          border border-white/20 transition-all duration-200 flex items-center gap-2
                          backdrop-blur-sm hover:backdrop-blur-md
                        '
                      >
                        <MoreHorizontal class='h-5 w-5' />
                      </button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align='start'>
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
              </template>
            </div>

            <div class='hidden lg:flex shrink-0 items-start justify-end'>
              <template v-if='isLoading'>
                <Skeleton class='w-64 h-64 rounded-2xl' />
              </template>
              <template v-else-if='playlist'>
                <div class='relative group'>
                  <div
                    class='
                      absolute -inset-4 rounded-3xl blur-xl opacity-0
                      group-hover:opacity-100 transition-opacity duration-300
                      bg-linear-to-br from-accent/30 to-accent/10
                    '
                  />

                  <ImageLoader
                    :alt='`${playlist.name} playlist art`'
                    :item-id='playlist.id'
                    :server-url='authStore.serverUrl'
                    :token='authStore.token'
                    class='
                      relative w-64 h-64 rounded-2xl shadow-2xl object-cover
                      transition-shadow duration-300 group-hover:shadow-2xl
                    '
                  >
                    <template #fallback>
                      <ImagePlaceholder
                        class='w-64 h-64 rounded-2xl shadow-2xl'
                        size='large'
                        type='playlist'
                      />
                    </template>
                  </ImageLoader>
                </div>
              </template>
            </div>
          </div>
        </div>
      </div>
    </section>

    <section v-if='playlist && !isLoading' class='flex justify-center'>
      <div class='w-full max-w-7xl py-6 px-6 md:px-10 lg:px-16'>
        <SongList
          @play-instant-mix='playInstantMix'
          @play-song='playSongWithQueue'
          @toggle-favorite='toggleSongFavorite'
          :hide-header='true'
          :loading='isLoading'
          :server-url='authStore.serverUrl'
          :show-album='true'
          :show-album-art='true'
          :show-artist='true'
          :show-duration='true'
          :songs='songs'
          :token='authStore.token'
        />

        <section v-if='songs.length === 0' class='text-center py-12'>
          <p class='text-muted-foreground text-lg'>
            This playlist is empty
          </p>
        </section>
      </div>
    </section>

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