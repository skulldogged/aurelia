<script setup lang="ts">
  import Fuse from 'fuse.js'
  import { Play } from 'lucide-vue-next'
  import { computed, ref, watch } from 'vue'
  import { useRouter } from 'vue-router'

  import { Album, Song } from '@/bindings'
  import AddToPlaylistMenu from '@/components/shared/AddToPlaylistMenu.vue'
  import AlbumStack from '@/components/shared/AlbumStack.vue'
  import Button from '@/components/ui/Button.vue'
  import {
    ContextMenu,
    ContextMenuContent,
    ContextMenuItem,
    ContextMenuTrigger,
  } from '@/components/ui/context-menu'
  import { Input } from '@/components/ui/input'
  import { Skeleton } from '@/components/ui/skeleton'
  import { useAuthStore } from '@/stores'
  import { useLibraryStore } from '@/stores/library'

  const router = useRouter()
  const authStore = useAuthStore()
  const libraryStore = useLibraryStore()

  defineProps<{
    currentSong?: null | Song
  }>()

  const emit = defineEmits<{
    'play-songs':   [songs: Song[]]
    'select-album': [album: Album]
  }>()

  const searchQuery = ref('')
  const allAlbums = computed(() => libraryStore.allAlbums as Album[])
  const libraryLoading = computed(() => libraryStore.isLoading)
  const serverUrl = computed(() => authStore.serverUrl)
  const token = computed(() => authStore.token)

  const albumsFuse = ref(new Fuse(allAlbums.value, {
    includeScore: true,
    keys:         [
      { name: 'name', weight: 0.6 },
      { name: 'artist', weight: 0.4 },
    ],
    minMatchCharLength: 2,
    threshold:          0.2,
  }))

  watch(() => allAlbums.value, newAlbums => {
    albumsFuse.value.setCollection(newAlbums as Album[])
  })

  const filteredAlbums = computed(() =>
    searchQuery.value && searchQuery.value.length >= 2
      ? albumsFuse.value.search(searchQuery.value).map(result => result.item)
      : [...allAlbums.value].sort((a, b) => a.name.toLowerCase().localeCompare(b.name.toLowerCase())),
  )

  const playAlbum = (album: Album): void => {
    if (album.songs && album.songs.length > 0) {
      const sortedSongs = [...album.songs].sort((a, b) => (a.trackNumber ?? 0) - (b.trackNumber ?? 0))
      emit('play-songs', sortedSongs)
    }
  }

  const selectAlbum = (album: Album): void => {
    if (album.id)
      router.push(`/albums/${album.id}`)
  }
</script>

<template>
  <div class='px-4 pb-4' style='padding-top: env(safe-area-inset-top)'>
    <div class='mb-6'>
      <h1 class='text-3xl font-bold mb-4'>
        Albums
      </h1>
      <Input
        v-model='searchQuery'
        class='w-full focus-visible:ring-1 focus-visible:ring-accent border-0 focus-visible:border-accent'
        placeholder='Search albums...'
        type='text'
      />
    </div>

    <div v-if='libraryLoading' class='grid grid-cols-2 gap-4'>
      <div
        v-for='n in 10'
        :key='`skeleton-${n}`'
        class='flex flex-col gap-3'
      >
        <Skeleton class='w-full aspect-square rounded-lg' name='album-art' />
        <div class='flex flex-col gap-1'>
          <Skeleton class='h-5 w-3/4' name='album-title' />
          <Skeleton class='h-4 w-20' name='artist' />
          <Skeleton class='h-4 w-16' name='song-count' />
        </div>
      </div>
    </div>

    <div v-else class='grid grid-cols-2 gap-4'>
      <ContextMenu v-for='album in filteredAlbums' :key='album.name'>
        <ContextMenuTrigger as-child>
          <div
            @click='selectAlbum(album)'
            class='cursor-pointer group'
          >
            <div class='relative mb-3'>
              <AlbumStack
                @play='playAlbum'
                :album='album'
                :server-url='serverUrl'
                :show-play-button='false'
                :size='"responsive"'
                :token='token'
              />

              <div
                class='
                  absolute inset-0 bg-black/25 rounded-lg opacity-0
                  group-hover:opacity-100 transition-opacity flex items-center
                  justify-center z-10
                '
              >
                <Button
                  @click.stop='playAlbum(album)'
                  class='
                    bg-white/20 hover:bg-white/30 backdrop-blur-sm text-white border
                    border-white/20
                  '
                  size='icon'
                >
                  <Play class='h-4 w-4' />
                </Button>
              </div>
            </div>

            <div>
              <p class='font-medium truncate text-sm'>
                {{ album.name }}
              </p>
              <p class='text-xs text-muted-foreground truncate'>
                {{ album.artist }}
              </p>
              <p
                v-if='album.songs'
                class='text-xs text-muted-foreground truncate'
              >
                {{ album.songs.length }} songs
              </p>
            </div>
          </div>
        </ContextMenuTrigger>
        <ContextMenuContent>
          <ContextMenuItem @click='playAlbum(album)'>
            <Play class='size-4 mr-2' />
            Play Album
          </ContextMenuItem>
          <AddToPlaylistMenu
            :songs='album.songs ? [...album.songs].sort((a, b) => (a.trackNumber ?? 0) - (b.trackNumber ?? 0)) : []'
            type='context'
          />
        </ContextMenuContent>
      </ContextMenu>
    </div>

    <div v-if='!libraryLoading && filteredAlbums.length === 0' class='text-center py-12'>
      <p class='text-muted-foreground'>
        No albums found
      </p>
    </div>
  </div>
</template>

<style scoped>
@reference "tailwindcss";
</style>