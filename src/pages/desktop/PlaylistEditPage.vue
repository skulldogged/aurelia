<script setup lang="ts">
  import { Check, GripVertical, Plus, Save, X } from 'lucide-vue-next'
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-vue'
  import { Sortable } from 'sortablejs-vue3'
  import { computed, onMounted, ref, watch } from 'vue'
  import { useRoute, useRouter } from 'vue-router'

  import type { PlaylistWithMeta } from '@/stores'

  import { PlaylistCreateData, PlaylistUpdateData, Song } from '@/bindings'
  import ImageLoader from '@/components/shared/ImageLoader.vue'
  import ImagePlaceholder from '@/components/shared/ImagePlaceholder.vue'
  import Button from '@/components/ui/Button.vue'
  import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
    DialogTrigger,
  } from '@/components/ui/dialog'
  import { Input } from '@/components/ui/input'
  import Label from '@/components/ui/Label.vue'
  import { logger } from '@/lib/logger'
  import { useAuthStore } from '@/stores'
  import { useLibraryStore } from '@/stores/library'
  import { usePlaylistStore } from '@/stores/playlists'

  const route = useRoute()
  const router = useRouter()
  const playlistStore = usePlaylistStore()
  const libraryStore = useLibraryStore()
  const authStore = useAuthStore()

  const id = computed(() => {
    const params = route.params
    if ('id' in params) {
      const param = params.id
      if (typeof param === 'string') return param
      if (Array.isArray(param)) return param[0] ?? ''
    }
    return ''
  })
  const isCreate = computed(() => route.path.includes('/create'))

  const playlist = ref<null | PlaylistWithMeta>(null)
  const name = ref('')
  const selectedSongs = ref<Song[]>([])
  const isSaving = ref(false)
  const isDragging = ref(false)

  // Error dialogs
  const showNameErrorDialog = ref(false)
  const showSaveErrorDialog = ref(false)

  // Add songs dialog
  const showAddSongsDialog = ref(false)
  const songSearchQuery = ref('')
  const filteredLibrarySongs = computed(() => {
    if (!songSearchQuery.value) return libraryStore.allSongs.slice(0, 50) // Limit for performance

    return libraryStore.allSongs.filter(song =>
      song.name.toLowerCase().includes(songSearchQuery.value.toLowerCase()) ||
      song.artists?.some(artist => artist?.toLowerCase().includes(songSearchQuery.value.toLowerCase())),
    ).slice(0, 50)
  })

  // Helper function to load songs from query params
  const loadSongsFromQuery = (): void => {
    if (!isCreate.value || !libraryStore.isLoaded) return

    const songIdsParam = route.query.songs as string | undefined
    if (!songIdsParam) return

    const songIds = songIdsParam.split(',').filter(id => id.length > 0)
    const newSongs = libraryStore.allSongs.filter(song => songIds.includes(song.id))

    // Avoid duplicates when accumulating songs
    const existingIds = new Set(selectedSongs.value.map(s => s.id))
    const songsToAdd = newSongs.filter(song => !existingIds.has(song.id))

    if (songsToAdd.length > 0) {
      selectedSongs.value = [...selectedSongs.value, ...songsToAdd]
      logger.info('Added', songsToAdd.length, 'songs. Total:', selectedSongs.value.length)
    }
  }

  const loadPlaylist = async (): Promise<void> => {
    if (isCreate.value) {
      // Initialize empty playlist for creation
      name.value = ''
      selectedSongs.value = []

      // Pre-select songs if provided via query params
      loadSongsFromQuery()

      return
    }

    if (!id.value) return

    try {
      // Find playlist in store
      const storedPlaylist = playlistStore.playlists.find(p => p.id === id.value)
      if (storedPlaylist) {
        playlist.value = storedPlaylist
        name.value = storedPlaylist.name
      } else {
        // If not in store, refresh playlists
        await playlistStore.loadPlaylists()
        const refreshedPlaylist = playlistStore.playlists.find(p => p.id === id.value)
        if (refreshedPlaylist) {
          playlist.value = refreshedPlaylist
          name.value = refreshedPlaylist.name
        } else {
          throw new Error('Playlist not found')
        }
      }

      // Load playlist songs
      selectedSongs.value = await playlistStore.getPlaylistItems(id.value)
    } catch (error) {
      logger.error('Failed to load playlist:', error)
      router.push('/playlists')
    }
  }

  const savePlaylist = async (): Promise<void> => {
    if (!name.value.trim()) {
      showNameErrorDialog.value = true
      return
    }

    isSaving.value = true

    try {
      if (isCreate.value) {
        const createData: PlaylistCreateData = {
          ids:      selectedSongs.value.map(song => song.id),
          isPublic: false,
          name:     name.value.trim(),
          userId:   authStore.userId,
        }

        const newPlaylist = await playlistStore.createPlaylist(createData)
        if (newPlaylist) {
          // Reload playlists to ensure the new one is fully available
          await playlistStore.loadPlaylists()
          // Navigate to the new playlist detail page
          router.push({ path: `/playlists/${newPlaylist.id}` })
        }
      } else if (playlist.value) {
        const updateData: PlaylistUpdateData = {
          ids:        selectedSongs.value.map(song => song.id),
          isFavorite: null,
          isPublic:   null,
          name:       name.value.trim(),
          songs:      null,
          userId:     null,
        }

        const success = await playlistStore.updatePlaylist(playlist.value.id, updateData)
        if (success) {
          // Reload playlists to get updated data
          await playlistStore.loadPlaylists()
          router.push({ path: `/playlists/${playlist.value.id}` })
        }
      }
    } catch (error) {
      logger.error('Failed to save playlist:', error)
      showSaveErrorDialog.value = true
    } finally {
      isSaving.value = false
    }
  }

  const toggleSongSelection = (song: Song): void => {
    const isSelected = selectedSongs.value.some(s => s.id === song.id)
    if (isSelected) {
      selectedSongs.value = selectedSongs.value.filter(s => s.id !== song.id)
    } else {
      selectedSongs.value.push(song)
    }
  }

  const removeSongFromPlaylist = (song: Song): void => {
    selectedSongs.value = selectedSongs.value.filter(s => s.id !== song.id)
  }

  const formatDuration = (seconds?: null | number): string => {
    if (seconds === undefined || seconds === null) return '?:??'
    const mins = Math.floor(seconds / 60)
    const secs = Math.floor(seconds % 60)
    return `${mins}:${secs.toString().padStart(2, '0')}`
  }

  const handleDragStart = (): void => {
    isDragging.value = true
  }

  const handleDragEnd = (event: { newIndex: number | undefined; oldIndex: number | undefined }): void => {
    isDragging.value = false
    const { newIndex, oldIndex } = event
    if (oldIndex === undefined || newIndex === undefined || oldIndex === newIndex)
      return

    const newList = [...selectedSongs.value]
    const [item] = newList.splice(oldIndex, 1)
    newList.splice(newIndex, 0, item)

    selectedSongs.value = newList
  }

  const goBack = (): void => {
    if (isCreate.value) {
      router.push('/playlists')
    } else {
      router.push(`/playlists/${id.value}`)
    }
  }

  // Watch for library to load and then apply pre-selected songs
  watch(() => libraryStore.isLoaded, loaded => {
    if (loaded && isCreate.value) {
      loadSongsFromQuery()
    }
  })

  // Watch for query param changes to add more songs
  watch(() => route.query.songs, () => {
    if (isCreate.value && libraryStore.isLoaded) {
      loadSongsFromQuery()
    }
  })

  onMounted(() => {
    loadPlaylist()
  })

  watch(() => id.value, () => {
    loadPlaylist()
  })
</script>

<template>
  <div class='px-4 md:px-6 lg:px-8 py-4 md:py-6 lg:py-8'>
    <div class='max-w-7xl mx-auto space-y-8'>
      <section class='space-y-2 mb-8'>
        <h1 class='text-4xl font-bold text-foreground'>
          {{ isCreate ? 'Create Playlist' : 'Edit Playlist' }}
        </h1>
        <p class='text-muted-foreground text-lg'>
          {{
            isCreate
              ? 'Build your perfect playlist from your music library'
              : 'Update your playlist details and song selection'
          }}
        </p>
      </section>

      <section class='bg-sidebar rounded-lg p-6 space-y-4'>
        <header class='mb-4'>
          <h2 class='text-2xl font-semibold'>
            Playlist Details
          </h2>
        </header>
        <div class='space-y-2'>
          <Label class='text-base' for='name'>
            Playlist Name *
          </Label>
          <Input
            id='name'
            v-model='name'
            :disabled='isSaving'
            class='text-base h-11'
            placeholder='My Awesome Playlist'
          />
          <p class='text-sm text-muted-foreground'>
            Give your playlist a memorable name
          </p>
        </div>
      </section>

      <section class='bg-sidebar rounded-lg p-6 space-y-6'>
        <header class='flex items-center justify-between'>
          <div>
            <h2 class='text-2xl font-semibold'>
              Songs
            </h2>
            <p class='text-sm text-muted-foreground mt-1'>
              {{ selectedSongs.length }} {{ selectedSongs.length === 1 ? 'song' : 'songs' }} added
            </p>
          </div>

          <Dialog v-model:open='showAddSongsDialog'>
            <DialogTrigger as-child>
              <Button class='gap-2' size='lg'>
                <Plus class='h-5 w-5' />
                Add Songs
              </Button>
            </DialogTrigger>
            <DialogContent class='max-w-3xl h-[85vh] flex flex-col p-0'>
              <DialogHeader class='shrink-0 px-6 pt-6 pb-4'>
                <DialogTitle class='text-2xl'>
                  Add Songs to Playlist
                </DialogTitle>
                <DialogDescription>
                  Search and select songs from your library
                </DialogDescription>
              </DialogHeader>

              <div class='flex flex-col flex-1 min-h-0 px-6 pb-6 gap-4'>
                <Input
                  v-model='songSearchQuery'
                  class='focus-visible:ring-1 focus-visible:ring-accent border
                         focus-visible:border-accent shrink-0 h-11'
                  placeholder='Search by song name, artist, or album...'
                />

                <div class='flex-1 min-h-0 -mx-2'>
                  <OverlayScrollbarsComponent
                    :options='{ scrollbars: { autoHide: "scroll" } }'
                    class='h-full'
                    defer
                  >
                    <div class='px-1 py-1 space-y-1.5'>
                      <div
                        v-for='song in filteredLibrarySongs'
                        @click='toggleSongSelection(song)'
                        :key='song.id'
                        :class="[
                          'flex items-center gap-4 p-3 rounded-lg cursor-pointer transition-all',
                          selectedSongs.some(s => s.id === song.id)
                            ? 'bg-accent/40 border-2 border-accent shadow-sm'
                            : 'hover:bg-accent/10 border-2 border-transparent hover:border-accent/30'
                        ]"
                      >
                        <div class='size-14 shrink-0 rounded-md overflow-hidden shadow-md'>
                          <ImageLoader
                            :alt='`${song.album || song.name} album art`'
                            :item-id='song.albumId || song.id'
                            :server-url='authStore.serverUrl'
                            :token='authStore.token'
                            class='size-full object-cover'
                          >
                            <template #fallback>
                              <ImagePlaceholder
                                class='size-full'
                                size='small'
                                type='album'
                              />
                            </template>
                          </ImageLoader>
                        </div>

                        <div class='flex-1 min-w-0'>
                          <div class='font-semibold truncate text-base'>
                            {{ song.name }}
                          </div>
                          <div class='text-sm text-muted-foreground truncate mt-1'>
                            <span v-if='song.artists'>
                              {{ song.artists.join(', ') }}
                            </span>
                            <span v-if='song.artists && song.album' class='text-muted-foreground/60'>
                              •
                            </span>
                            <span v-if='song.album'>
                              {{ song.album }}
                            </span>
                          </div>
                        </div>

                        <div
                          :class="[
                            'shrink-0 size-7 rounded-full flex items-center justify-center',
                            'transition-all',
                            selectedSongs.some(s => s.id === song.id)
                              ? 'bg-accent text-accent-foreground'
                              : 'border-2 border-muted-foreground/40'
                          ]"
                        >
                          <Check
                            v-if='selectedSongs.some(s => s.id === song.id)'
                            class='size-4 font-bold'
                          />
                        </div>
                      </div>

                      <div
                        v-if='filteredLibrarySongs.length === 0'
                        class='text-center py-12 text-muted-foreground'
                      >
                        <div class='text-lg font-medium mb-1'>
                          No songs found
                        </div>
                        <div class='text-sm'>
                          Try a different search term
                        </div>
                      </div>
                    </div>
                  </OverlayScrollbarsComponent>
                </div>
              </div>
            </DialogContent>
          </Dialog>
        </header>

        <section
          v-if='selectedSongs.length === 0'
          class='text-center py-16 text-muted-foreground border-2 border-dashed border-border/50 rounded-lg'
        >
          <div class='flex flex-col items-center gap-3'>
            <div class='size-16 rounded-full bg-accent/10 flex items-center justify-center'>
              <Plus class='size-8 text-muted-foreground' />
            </div>
            <div>
              <div class='text-lg font-medium mb-1'>
                No songs added yet
              </div>
              <div class='text-sm'>
                Click "Add Songs" above to start building your playlist
              </div>
            </div>
          </div>
        </section>

        <section v-else class='space-y-1.5'>
          <Sortable
            @end='handleDragEnd'
            @start='handleDragStart'
            :list='selectedSongs'
            :options="{ animation: 150, ghostClass: 'ghost', dragClass: 'drag' }"
            handle='.handle'
            item-key='id'
          >
            <template #item='{ element: song }'>
              <div
                class='flex items-center gap-3 p-3 rounded-lg bg-card/30 border border-border/30
                       hover:bg-card/50 hover:border-border/50 transition-all group mb-1.5'
              >
                <Button
                  class='handle cursor-grab shrink-0 p-1 opacity-0 group-hover:opacity-100
                         transition-opacity'
                  size='icon'
                  variant='ghost'
                >
                  <GripVertical class='size-4 text-muted-foreground' />
                </Button>

                <div class='size-14 shrink-0 rounded-md overflow-hidden shadow-md'>
                  <ImageLoader
                    :alt='`${song.album || song.name} album art`'
                    :item-id='song.albumId || song.id'
                    :server-url='authStore.serverUrl'
                    :token='authStore.token'
                    class='size-full object-cover'
                  >
                    <template #fallback>
                      <ImagePlaceholder
                        class='size-full'
                        size='small'
                        type='album'
                      />
                    </template>
                  </ImageLoader>
                </div>

                <div class='flex-1 min-w-0'>
                  <div class='font-semibold truncate text-base'>
                    {{ song.name }}
                  </div>
                  <div class='text-sm text-muted-foreground truncate mt-1'>
                    <span v-if='song.artists'>
                      {{ song.artists.join(', ') }}
                    </span>
                    <span v-if='song.artists && song.album' class='text-muted-foreground/60'>
                      •
                    </span>
                    <span v-if='song.album'>
                      {{ song.album }}
                    </span>
                  </div>
                </div>

                <div
                  v-if='song.duration'
                  class='text-sm text-muted-foreground font-medium shrink-0 w-16 text-right'
                >
                  {{ formatDuration(song.duration) }}
                </div>

                <Button
                  @click='removeSongFromPlaylist(song)'
                  class='shrink-0 opacity-0 group-hover:opacity-100 transition-opacity'
                  size='icon'
                  variant='ghost'
                >
                  <X class='h-4 w-4' />
                </Button>
              </div>
            </template>
          </Sortable>
        </section>
      </section>

      <footer class='flex items-center gap-4 pt-2'>
        <Button
          @click='savePlaylist'
          :disabled='isSaving || !name.trim()'
          class='gap-2'
          size='lg'
        >
          <Save class='h-5 w-5' />
          {{ isSaving ? 'Saving...' : 'Save Playlist' }}
        </Button>
        <Button
          @click='goBack'
          :disabled='isSaving'
          size='lg'
          variant='outline'
        >
          Cancel
        </Button>
      </footer>
    </div>

    <Dialog v-model:open='showNameErrorDialog'>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Playlist Name Required</DialogTitle>
          <DialogDescription>
            Please enter a name for your playlist before saving.
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button @click='showNameErrorDialog = false'>
            OK
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

    <Dialog v-model:open='showSaveErrorDialog'>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Save Failed</DialogTitle>
          <DialogDescription>
            Failed to save playlist. Please try again.
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button @click='showSaveErrorDialog = false'>
            OK
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </div>
</template>

<style scoped>
.ghost {
  opacity: 0.5;
  background: var(--color-accent);
}

.drag {
  opacity: 0;
}
</style>