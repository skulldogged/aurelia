<script setup lang="ts">
  import { GripVertical, Plus, Save, X } from 'lucide-vue-next'
  import { Sortable } from 'sortablejs-vue3'
  import { computed, onMounted, ref, watch } from 'vue'
  import { useRoute, useRouter } from 'vue-router'

  import type { PlaylistWithMeta } from '@/stores'

  import { PlaylistCreateData, PlaylistUpdateData, Song } from '@/bindings'
  import ImageLoader from '@/components/shared/ImageLoader.vue'
  import SongPickerDialog from '@/components/shared/SongPickerDialog.vue'
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

  // Helper function to load songs from query params
  const loadSongsFromQuery = (): void => {
    if (!isCreate.value || !libraryStore.isLoaded) return

    const songIdsParam = route.query.songs as string | undefined
    if (!songIdsParam) return

    const songIds = songIdsParam.split(',').filter(id => id.length > 0)
    const newSongs = libraryStore.allSongs.filter(song => songIds.includes(song.id))

    const existingIds = new Set(selectedSongs.value.map(s => s.id))
    const songsToAdd = newSongs.filter(song => !existingIds.has(song.id))

    if (songsToAdd.length > 0) {
      selectedSongs.value = [...selectedSongs.value, ...songsToAdd]
    }
  }

  const loadPlaylist = async (): Promise<void> => {
    if (isCreate.value) {
      loadSongsFromQuery()
      return
    }

    if (!id.value) return

    try {
      const storedPlaylist = playlistStore.playlists.find(p => p.id === id.value)
      if (storedPlaylist) {
        playlist.value = storedPlaylist
        name.value = storedPlaylist.name
      } else {
        await playlistStore.loadPlaylists()
        const refreshedPlaylist = playlistStore.playlists.find(p => p.id === id.value)
        if (refreshedPlaylist) {
          playlist.value = refreshedPlaylist
          name.value = refreshedPlaylist.name
        } else {
          throw new Error('Playlist not found')
        }
      }
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
          await playlistStore.loadPlaylists()
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

  const removeSongFromPlaylist = (song: Song): void => {
    selectedSongs.value = selectedSongs.value.filter(s => s.id !== song.id)
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

  watch(() => libraryStore.isLoaded, loaded => {
    if (loaded && isCreate.value) {
      loadSongsFromQuery()
    }
  })

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
  <div class='p-4 space-y-6'>
    <!-- Header -->
    <div>
      <h1 class='text-3xl font-bold'>
        {{ isCreate ? 'Create Playlist' : 'Edit Playlist' }}
      </h1>
      <p class='text-muted-foreground mt-1'>
        {{ isCreate ? 'Build your perfect playlist.' : 'Update your playlist.' }}
      </p>
    </div>

    <!-- Form -->
    <div class='space-y-6'>
      <!-- Playlist Name -->
      <div class='space-y-2'>
        <Label for='name'>Playlist Name *</Label>
        <Input
          id='name'
          v-model='name'
          :disabled='isSaving'
          placeholder='My Awesome Playlist'
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
              <Button class='gap-2' size='sm'>
                <Plus class='h-4 w-4' />
                Add Songs
              </Button>
            </DialogTrigger>
            <SongPickerDialog v-model='selectedSongs' />
          </Dialog>
        </div>

        <div v-if='selectedSongs.length === 0' class='text-center py-12 border-2 border-dashed rounded-lg'>
          <p class='text-muted-foreground'>
            No songs added yet.
          </p>
        </div>

        <div v-else class='space-y-1'>
          <Sortable
            @end='handleDragEnd'
            @start='handleDragStart'
            :list='selectedSongs'
            handle='.handle'
            item-key='id'
          >
            <template #item='{ element: song }'>
              <div class='flex items-center gap-3 p-2 rounded-lg bg-card/30'>
                <Button class='handle cursor-grab p-1' size='icon' variant='ghost'>
                  <GripVertical class='size-4 text-muted-foreground' />
                </Button>
                <ImageLoader
                  :item-id='song.albumId || song.id'
                  :server-url='authStore.serverUrl'
                  :token='authStore.token'
                  class='size-12 rounded-md object-cover'
                />
                <div class='flex-1 min-w-0'>
                  <div class='font-semibold truncate'>
                    {{ song.name }}
                  </div>
                  <div class='text-sm text-muted-foreground truncate'>
                    {{ song.artists?.join(', ') }}
                  </div>
                </div>
                <Button @click='removeSongFromPlaylist(song)' size='icon' variant='ghost'>
                  <X class='h-4 w-4' />
                </Button>
              </div>
            </template>
          </Sortable>
        </div>
      </div>

      <!-- Actions -->
      <div class='flex items-center gap-4 pt-2'>
        <Button @click='savePlaylist' :disabled='isSaving || !name.trim()' size='lg'>
          <Save class='h-5 w-5 mr-2' />
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
      </div>
    </div>

    <!-- Error Dialogs -->
    <Dialog v-model:open='showNameErrorDialog'>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Playlist Name Required</DialogTitle>
          <DialogDescription>Please enter a name for your playlist.</DialogDescription>
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
          <DialogDescription>Failed to save playlist. Please try again.</DialogDescription>
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
