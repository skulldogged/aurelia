  <script setup lang="ts">
  import { ArrowLeft, Plus, Save } from 'lucide-vue-next'
  import { OverlayScrollbarsComponent } from 'overlayscrollbars-vue'
  import { computed, onMounted, ref, watch } from 'vue'
  import { useRoute, useRouter } from 'vue-router'

  import type { PlaylistWithMeta } from '@/stores'

  import { PlaylistCreateData, PlaylistUpdateData, Song } from '@/bindings'
  import SongList from '@/components/shared/SongList.vue'
  import { Button } from '@/components/ui/button'
  import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
    DialogTrigger,
  } from '@/components/ui/dialog'
  import { Input } from '@/components/ui/input'
  import { Label } from '@/components/ui/label'
  import { useAuthStore, useLibraryStore, usePlaylistStore } from '@/stores'

  const route = useRoute()
  const router = useRouter()
  const playlistStore = usePlaylistStore()
  const libraryStore = useLibraryStore()
  const authStore = useAuthStore()

  const playlistId = computed(() => route.params.playlistId as string)
  const isCreate = computed(() => route.path.includes('/create'))

  const playlist = ref<null | PlaylistWithMeta>(null)
  const name = ref('')
  const selectedSongs = ref<Song[]>([])
  const isSaving = ref(false)

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

  const loadPlaylist = async (): Promise<void> => {
    if (isCreate.value) {
      // Initialize empty playlist for creation
      name.value = ''
      selectedSongs.value = []
      return
    }

    if (!playlistId.value) return

    try {
      // Find playlist in store
      const storedPlaylist = playlistStore.playlists.find(p => p.id === playlistId.value)
      if (storedPlaylist) {
        playlist.value = storedPlaylist
        name.value = storedPlaylist.name
      } else {
        // If not in store, refresh playlists
        await playlistStore.loadPlaylists()
        const refreshedPlaylist = playlistStore.playlists.find(p => p.id === playlistId.value)
        if (refreshedPlaylist) {
          playlist.value = refreshedPlaylist
          name.value = refreshedPlaylist.name
        } else {
          throw new Error('Playlist not found')
        }
      }

      // Load playlist songs
      selectedSongs.value = await playlistStore.getPlaylistItems(playlistId.value)
    } catch (error) {
      console.error('Failed to load playlist:', error)
      router.push('/playlists')
    }
  }

  const savePlaylist = async (): Promise<void> => {
    if (!name.value.trim()) {
      alert('Please enter a playlist name')
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
          router.push(`/playlists/${newPlaylist.id}`)
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
          router.push(`/playlists/${playlist.value.id}`)
        }
      }
    } catch (error) {
      console.error('Failed to save playlist:', error)
      alert('Failed to save playlist. Please try again.')
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

  const goBack = (): void => {
    if (isCreate.value) {
      router.push('/playlists')
    } else {
      router.push(`/playlists/${playlistId.value}`)
    }
  }

  onMounted(() => {
    loadPlaylist()
  })

  watch(() => playlistId.value, () => {
    loadPlaylist()
  })
</script>

<template>
  <div class='p-4 max-w-4xl mx-auto'>
    <!-- Header -->
    <div class='mb-8'>
      <Button @click='goBack' class='mb-4 gap-2' variant='ghost'>
        <ArrowLeft class='h-4 w-4' />
        {{ isCreate ? 'Back to Playlists' : 'Back to Playlist' }}
      </Button>

      <h1 class='text-4xl font-bold mb-6'>
        {{ isCreate ? 'Create Playlist' : 'Edit Playlist' }}
      </h1>
    </div>

    <!-- Form -->
    <div class='space-y-6'>
      <div class='space-y-2'>
        <Label for='name'>Name *</Label>
        <Input
          id='name'
          v-model='name'
          :disabled='isSaving'
          placeholder='Enter playlist name'
        />
      </div>

      <!-- Selected Songs -->
      <div class='space-y-4'>
        <div class='flex items-center justify-between'>
          <h2 class='text-xl font-semibold'>
            Songs ({{ selectedSongs.length }})
          </h2>

          <Dialog v-model:open='showAddSongsDialog'>
            <DialogTrigger as-child>
              <Button class='gap-2' variant='outline'>
                <Plus class='h-4 w-4' />
                Add Songs
              </Button>
            </DialogTrigger>
            <DialogContent class='max-w-2xl max-h-[80vh] flex flex-col'>
              <DialogHeader class='flex-shrink-0'>
                <DialogTitle>Add Songs to Playlist</DialogTitle>
              </DialogHeader>

              <div class='flex flex-col space-y-4 flex-1 min-h-0'>
                <Input
                  v-model='songSearchQuery'
                  class='focus-visible:ring-1 focus-visible:ring-accent border-0
                         focus-visible:border-accent flex-shrink-0'
                  placeholder='Search songs...'
                />

                <div class='max-h-96'>
                  <OverlayScrollbarsComponent :options='{ scrollbars: { autoHide: "scroll" } }' class='h-96' defer>
                    <div class='px-2 py-1 space-y-2'>
                      <div
                        v-for='song in filteredLibrarySongs'
                        @click='toggleSongSelection(song)'
                        :key='song.id'
                        class='flex items-center gap-3 p-3 rounded-lg hover:bg-accent/50
                               cursor-pointer transition-colors'
                      >
                        <div class='relative flex items-center justify-center'>
                          <input
                            @change.stop='toggleSongSelection(song)'
                            @click.stop
                            :checked='selectedSongs.some(s => s.id === song.id)'
                            class='peer h-5 w-5 shrink-0 appearance-none rounded-sm border border-input
                                   ring-offset-background focus-visible:outline-none focus-visible:ring-2
                                   focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed
                                   disabled:opacity-50 checked:bg-accent checked:text-accent-foreground
                                   checked:border-accent'
                            type='checkbox'
                          >
                          <div
                            class='absolute inset-0 flex items-center justify-center text-accent-foreground
                                     opacity-0 peer-checked:opacity-100 pointer-events-none'
                          >
                            <svg
                              class='h-3.5 w-3.5'
                              fill='none'
                              viewBox='0 0 12 12'
                              xmlns='http://www.w3.org/2000/svg'
                            >
                              <path
                                d='M10.5 3L4.5 9L2 6.5'
                                stroke='currentColor'
                                stroke-linecap='round'
                                stroke-linejoin='round'
                                stroke-width='1.5'
                              />
                            </svg>
                          </div>
                        </div>

                        <div class='flex-1 min-w-0'>
                          <div class='font-medium truncate'>
                            {{ song.name }}
                          </div>
                          <div class='text-sm text-muted-foreground truncate'>
                            {{ song.artists?.join(', ') }} • {{ song.album }}
                          </div>
                        </div>
                      </div>

                      <div v-if='filteredLibrarySongs.length === 0' class='text-center py-8 text-muted-foreground'>
                        No songs found matching "{{ songSearchQuery }}"
                      </div>
                    </div>
                  </OverlayScrollbarsComponent>
                </div>
              </div>
            </DialogContent>
          </Dialog>
        </div>

        <div v-if='selectedSongs.length === 0' class='text-center py-8 text-muted-foreground'>
          No songs added yet. Click "Add Songs" to get started.
        </div>

        <div v-else class='space-y-2'>
          <SongList
            @remove-song='removeSongFromPlaylist'
            :current-song='null'
            :is-playing='false'
            :server-url='""'
            :show-album='true'
            :show-artist='true'
            :show-remove-button='true'
            :songs='selectedSongs'
            :token='""'
          />
        </div>
      </div>

      <!-- Actions -->
      <div class='flex items-center gap-4 pt-6 border-t' />
      <Button @click='savePlaylist' :disabled='isSaving || !name.trim()' class='gap-2'>
        <Save class='h-4 w-4' />
        {{ isSaving ? 'Saving...' : 'Save Playlist' }}
      </Button>
      <Button @click='goBack' :disabled='isSaving' variant='outline'>
        Cancel
      </Button>
    </div>
  </div>
</template>
