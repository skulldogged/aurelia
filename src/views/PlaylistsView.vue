<script setup lang="ts">
  import Fuse from 'fuse.js'
  import { Heart, HeartOff, MoreHorizontal, Play, Plus } from 'lucide-vue-next'
  import { ChevronLeft, ChevronRight, ChevronsLeft, ChevronsRight } from 'lucide-vue-next'
  import { computed, ref, watch } from 'vue'
  import { useRouter } from 'vue-router'

  import { Playlist } from '@/bindings'
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
  } from '@/components/ui/dialog'
  import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
  } from '@/components/ui/dropdown-menu'
  import { Input } from '@/components/ui/input'
  import {
    Select,
    SelectContent,
    SelectGroup,
    SelectItem,
    SelectTrigger,
    SelectValue,
  } from '@/components/ui/select'
  import { Skeleton } from '@/components/ui/skeleton'
  import { usePagination } from '@/composables/useLayoutPreference'
  import { useAuthStore, usePlaylistStore } from '@/stores'

  const router = useRouter()
  const authStore = useAuthStore()
  const playlistStore = usePlaylistStore()

  const searchQuery = ref('')
  const showDeleteDialog = ref(false)
  const playlistToDelete = ref<null | Playlist>(null)

  const playlistsFuse = ref(new Fuse(playlistStore.playlists, {
    includeScore: true,
    keys:         [
      { name: 'name', weight: 0.7 },
      { name: 'description', weight: 0.3 },
    ],
    minMatchCharLength: 2,
    threshold:          0.2,
  }))

  watch(() => playlistStore.playlists, newPlaylists => {
    playlistsFuse.value.setCollection(newPlaylists)
  })

  const filteredPlaylists = computed(() =>
    searchQuery.value && searchQuery.value.length >= 2
      ? playlistsFuse.value.search(searchQuery.value).map(result => result.item)
      : [...playlistStore.playlists].sort((a, b) => a.name.toLowerCase().localeCompare(b.name.toLowerCase())),
  )

  // Pagination
  const {
    canNextPage,
    canPreviousPage,
    goToFirstPage,
    goToLastPage,
    goToNextPage,
    goToPreviousPage,
    pageCount,
    pagedItems: pagedPlaylists,
    pageIndex,
    pageSize,
    pageSizeOptions,
    setPageSize,
    total,
  } = usePagination(filteredPlaylists, 'playlists-pagesize', 20)

  const playPlaylist = async (playlist: Playlist): Promise<void> => {
    await playlistStore.playPlaylist(playlist.id)
  }

  const selectPlaylist = (playlist: Playlist): void => {
    console.log('Selecting playlist:', playlist)
    if (!playlist.id) {
      console.error('Playlist has no ID:', playlist)
      return
    }
    router.push(`/playlists/${playlist.id}`)
  }

  const toggleFavorite = async (playlist: Playlist): Promise<void> => {
    await playlistStore.togglePlaylistFavorite(playlist.id)
  }

  const createPlaylist = (): void => {
    router.push('/playlists/create')
  }

  const deletePlaylist = (playlist: Playlist): void => {
    playlistToDelete.value = playlist
    showDeleteDialog.value = true
  }

  const confirmDelete = async (): Promise<void> => {
    if (!playlistToDelete.value) return
    await playlistStore.deletePlaylist(playlistToDelete.value.id)
    showDeleteDialog.value = false
    playlistToDelete.value = null
  }

  const cancelDelete = (): void => {
    showDeleteDialog.value = false
    playlistToDelete.value = null
  }
</script>

<template>
  <div class='p-4 max-w-7xl mx-auto'>
    <div class='mb-8'>
      <div class='flex items-center justify-between mb-4'>
        <h1 class='text-4xl font-bold'>
          Playlists
        </h1>
        <Button @click='createPlaylist' class='gap-2'>
          <Plus class='h-4 w-4' />
          Create Playlist
        </Button>
      </div>
      <div class='flex flex-col sm:flex-row gap-4 items-start sm:items-center'>
        <Input
          v-model='searchQuery'
          class='max-w-sm focus-visible:ring-1 focus-visible:ring-accent border-0 focus-visible:border-accent'
          placeholder='Search playlists...'
          type='text'
        />
      </div>
    </div>

    <div
      v-if='playlistStore.isLoading'
      class='grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-6'
    >
      <div
        v-for='n in 20'
        :key='`skeleton-${n}`'
        class='flex flex-col gap-4'
      >
        <Skeleton class='w-full aspect-square rounded-lg' name='playlist-art' />
        <div class='flex flex-col gap-1'>
          <Skeleton class='h-6 w-3/4' name='playlist-title' />
          <Skeleton class='h-4 w-20' name='description' />
          <Skeleton class='h-4 w-16' name='song-count' />
        </div>
      </div>
    </div>
    <div v-else class='grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-6'>
      <div
        v-for='playlist in pagedPlaylists'
        @click='selectPlaylist(playlist)'
        :key='playlist.id'
        class='cursor-pointer group'
      >
        <div class='relative mb-4'>
          <ImageLoader
            :alt='`${playlist.name} playlist art`'
            :item-id='playlist.id'
            :server-url='authStore.serverUrl'
            :token='authStore.token'
            class='
              w-full aspect-square rounded-lg object-cover shadow-lg
              group-hover:opacity-75 transition-opacity
            '
          >
            <template #fallback>
              <ImagePlaceholder
                class='w-full aspect-square shadow-lg group-hover:opacity-75 transition-opacity'
                size='large'
                type='playlist'
              />
            </template>
          </ImageLoader>

          <div
            class='
              absolute inset-0 bg-black/50 rounded-lg opacity-0
              group-hover:opacity-100 transition-opacity flex items-center
              justify-center gap-2
            '
          >
            <Button
              @click.stop='playPlaylist(playlist)'
              class='
                bg-white/20 hover:bg-white/30 backdrop-blur-sm text-white border
                border-white/20
              '
              size='icon'
            >
              <Play class='h-4 w-4' />
            </Button>
            <DropdownMenu>
              <DropdownMenuTrigger as-child>
                <Button
                  @click.stop
                  class='
                    bg-white/20 hover:bg-white/30 backdrop-blur-sm text-white border
                    border-white/20
                  '
                  size='icon'
                  variant='ghost'
                >
                  <MoreHorizontal class='h-4 w-4' />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent>
                <DropdownMenuItem @click.stop='toggleFavorite(playlist)'>
                  <Heart v-if='playlist.userData?.isFavorite' class='h-4 w-4 mr-2' />
                  <HeartOff v-else class='h-4 w-4 mr-2' />
                  {{ playlist.userData?.isFavorite ? 'Unfavorite' : 'Favorite' }}
                </DropdownMenuItem>
                <DropdownMenuItem @click.stop='deletePlaylist(playlist)' class='text-destructive'>
                  Delete Playlist
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
        </div>

        <div>
          <div class='flex items-center gap-2 mb-1'>
            <p class='font-semibold truncate flex-1'>
              {{ playlist.name || 'Untitled Playlist' }}
            </p>
            <Heart v-if='playlist.userData?.isFavorite' class='h-4 w-4 text-red-500 flex-shrink-0' />
          </div>
          <p v-if='playlist.description' class='text-sm text-muted-foreground truncate'>
            {{ playlist.description }}
          </p>
          <p class='text-sm text-muted-foreground truncate'>
            {{ playlist.childCount || 0 }} songs
          </p>
        </div>
      </div>
    </div>

    <div v-if='!playlistStore.isLoading && filteredPlaylists.length === 0' class='text-center py-12'>
      <p class='text-muted-foreground mb-4'>
        {{ playlistStore.playlists.length === 0 ? 'No playlists found' : 'No playlists match your search' }}
      </p>
      <Button @click='createPlaylist' v-if='playlistStore.playlists.length === 0' class='gap-2'>
        <Plus class='h-4 w-4' />
        Create Your First Playlist
      </Button>
    </div>

    <!-- Pagination Controls -->
    <div v-if='pageCount > 1' class='flex items-center justify-between border-t border-border pt-6 mt-8'>
      <div class='flex items-center gap-2'>
        <span class='text-sm text-muted-foreground'>Playlists per page:</span>
        <Select @update:model-value='(v) => setPageSize(Number(v))' :model-value='String(pageSize)'>
          <SelectTrigger class='w-20'>
            <SelectValue placeholder='Per page' />
          </SelectTrigger>
          <SelectContent>
            <SelectGroup>
              <SelectItem v-for='option in pageSizeOptions' :key='option' :value='String(option)'>
                {{ option }}
              </SelectItem>
            </SelectGroup>
          </SelectContent>
        </Select>
      </div>

      <div class='flex items-center gap-2'>
        <span class='text-sm text-muted-foreground'>
          Page {{ pageIndex + 1 }} of {{ pageCount }} ({{ total }} total)
        </span>
      </div>

      <div class='flex items-center gap-1'>
        <Button
          @click='goToFirstPage'
          :disabled='!canPreviousPage'
          size='sm'
          variant='outline'
        >
          <ChevronsLeft class='h-4 w-4' />
        </Button>
        <Button
          @click='goToPreviousPage'
          :disabled='!canPreviousPage'
          size='sm'
          variant='outline'
        >
          <ChevronLeft class='h-4 w-4' />
        </Button>
        <Button
          @click='goToNextPage'
          :disabled='!canNextPage'
          size='sm'
          variant='outline'
        >
          <ChevronRight class='h-4 w-4' />
        </Button>
        <Button
          @click='goToLastPage'
          :disabled='!canNextPage'
          size='sm'
          variant='outline'
        >
          <ChevronsRight class='h-4 w-4' />
        </Button>
      </div>
    </div>

    <!-- Delete Confirmation Dialog -->
    <Dialog v-model:open='showDeleteDialog'>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Delete Playlist</DialogTitle>
          <DialogDescription>
            Are you sure you want to delete "{{ playlistToDelete?.name }}"? This action cannot be undone.
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
