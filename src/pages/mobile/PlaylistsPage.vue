<script setup lang="ts">
  import Fuse from 'fuse.js'
  import { Plus } from 'lucide-vue-next'
  import { computed, ref, watch } from 'vue'
  import { useRouter } from 'vue-router'

  import { Playlist } from '@/bindings'
  import ImageLoader from '@/components/shared/ImageLoader.vue'
  import ImagePlaceholder from '@/components/shared/ImagePlaceholder.vue'
  import Button from '@/components/ui/Button.vue'
  import { Input } from '@/components/ui/input'
  import { Skeleton } from '@/components/ui/skeleton'
  import { useAuthStore } from '@/stores'
  import { usePlaylistStore } from '@/stores/playlists'

  const router = useRouter()
  const authStore = useAuthStore()
  const playlistStore = usePlaylistStore()

  const searchQuery = ref('')

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

  const selectPlaylist = (playlist: Playlist): void => {
    if (!playlist.id) return
    router.push(`/playlists/${playlist.id}`)
  }

  const createPlaylist = (): void => {
    router.push('/playlists/create')
  }
</script>

<template>
  <div class='p-4 space-y-6'>
    <div class='flex items-center justify-between'>
      <h1 class='text-3xl font-bold'>
        Playlists
      </h1>
      <Button @click='createPlaylist' class='gap-2' size='sm'>
        <Plus class='h-4 w-4' />
        New
      </Button>
    </div>

    <Input
      v-model='searchQuery'
      placeholder='Search playlists...'
      type='text'
    />

    <div v-if='playlistStore.isLoading' class='space-y-4'>
      <div v-for='n in 10' :key='`skeleton-${n}`' class='flex items-center gap-4'>
        <Skeleton class='size-16 rounded-lg' />
        <div class='flex-1 space-y-2'>
          <Skeleton class='h-5 w-3/4' />
          <Skeleton class='h-4 w-1/4' />
        </div>
      </div>
    </div>

    <div v-else-if='filteredPlaylists.length > 0' class='space-y-2'>
      <div
        v-for='playlist in filteredPlaylists'
        @click='selectPlaylist(playlist)'
        :key='playlist.id'
        class='flex items-center gap-4 p-2 rounded-lg hover:bg-muted/50 cursor-pointer transition-colors'
      >
        <ImageLoader
          :item-id='playlist.id'
          :server-url='authStore.serverUrl'
          :token='authStore.token'
          alt='Playlist art'
          class='size-16 rounded-lg object-cover shadow-lg'
        >
          <template #fallback>
            <ImagePlaceholder class='size-16 rounded-lg shadow-lg' type='playlist' />
          </template>
        </ImageLoader>
        <div class='flex-1 min-w-0'>
          <p class='font-semibold truncate'>
            {{ playlist.name || 'Untitled Playlist' }}
          </p>
          <p class='text-sm text-muted-foreground truncate'>
            {{ playlist.childCount || 0 }} songs
          </p>
        </div>
      </div>
    </div>

    <div v-else class='text-center py-12'>
      <p class='text-muted-foreground mb-4'>
        {{ playlistStore.playlists.length === 0 ? 'No playlists found' : 'No playlists match your search' }}
      </p>
      <Button @click='createPlaylist' v-if='playlistStore.playlists.length === 0' class='gap-2'>
        <Plus class='h-4 w-4' />
        Create Your First Playlist
      </Button>
    </div>
  </div>
</template>
