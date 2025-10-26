<script setup lang="ts">
  import { Play, Shuffle } from 'lucide-vue-next'
  import { computed } from 'vue'

  import { Album, Song } from '@/bindings'
  import AddToPlaylistMenu from '@/components/shared/AddToPlaylistMenu.vue'
  import AlbumStack from '@/components/shared/AlbumStack.vue'
  import ImageLoader from '@/components/shared/ImageLoader.vue'
  import ImagePlaceholder from '@/components/shared/ImagePlaceholder.vue'
  import Button from '@/components/ui/Button.vue'
  import {
    ContextMenu,
    ContextMenuContent,
    ContextMenuItem,
    ContextMenuTrigger,
  } from '@/components/ui/context-menu'
  import { Skeleton } from '@/components/ui/skeleton'
  import { useSongInteractions } from '@/composables/useSongInteractions'
  import { logger } from '@/lib/logger'
  import { sortSongsByTrackOrder } from '@/lib/transforms'
  import { useAuthStore } from '@/stores'

  const authStore = useAuthStore()

  const credentials = computed(() => ({
    serverUrl: authStore.serverUrl,
    token:     authStore.token,
    userId:    authStore.userId,
    username:  authStore.username,
  }))

  const serverUrl = computed(() => credentials.value.serverUrl)
  const token = computed(() => credentials.value.token)

  const { playInstantMix } = useSongInteractions(credentials)

  defineProps<{
    isLoading: boolean
    items:     Album[] | Song[]
    title:     string
    type:      'album' | 'song'
  }>()

  const emit = defineEmits<{
    'play-songs':   [songs: Song[]]
    'select-album': [album: Album]
  }>()

  const playSongs = (songs: Song[], startWith?: Song): void => {
    if (songs.length === 0) {
      logger.warn('No songs to play')
      return
    }

    const invalidSongs = songs.filter(song => !song || !song.id)

    if (invalidSongs.length > 0)
      logger.error('Found songs with invalid IDs:', invalidSongs)

    if (startWith) {
      const startIndex = songs.findIndex(song => song.id === startWith.id)
      if (startIndex === -1) {
        emit('play-songs', songs)
        return
      }
      const reorderedSongs = [...songs.slice(startIndex), ...songs.slice(0, startIndex)]
      emit('play-songs', reorderedSongs)
    } else {
      emit('play-songs', songs)
    }
  }

  const playAlbumSongs = (album: Album): void => {
    if (album.songs && album.songs.length > 0)
      emit('play-songs', sortSongsByTrackOrder(album.songs))
    else
      logger.warn('No songs found for album', album.name)
  }

  const getAlbumArtists = (album: Album): { id: string, name: string }[] => {
    const idToName = new Map<string, string>()
    const albumSongs = album.songs || []

    for (const song of albumSongs) {
      if (song.albumArtists) {
        for (const pair of song.albumArtists) {
          if (pair.id && pair.name) {
            idToName.set(pair.id, pair.name)
          }
        }
      }
    }

    if (idToName.size === 0) {
      const first = albumSongs[0]
      if (first?.artistIds && first.artists && first.artistIds.length === first.artists.length) {
        first.artistIds.forEach((id, idx) => {
          const name = first.artists![idx]
          if (id && name) {
            idToName.set(id, name)
          }
        })
      } else if (album.artist && album.artistId) {
        idToName.set(album.artistId, album.artist)
      }
    }

    if (idToName.size === 0 && album.artist) {
      // Final fallback for albums that might not have songs attached in this context
      idToName.set(album.artistId || album.artist, album.artist)
    }

    return Array.from(idToName, ([id, name]) => ({ id, name }))
  }
</script>

<template>
  <div class='mb-8'>
    <h2 class='text-2xl font-bold mb-4'>
      {{ title }}
    </h2>
    <div
      v-if='isLoading'
      class='grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4'
    >
      <div
        v-for='n in 6'
        :key='`skeleton-${n}`'
        class='cursor-pointer group'
      >
        <div class='relative mb-2 album-stack-container'>
          <Skeleton class='album-stack-layer album-stack-layer-3 album-art-image' />
          <Skeleton class='album-stack-layer album-stack-layer-2 album-art-image' />
          <Skeleton class='album-stack-layer album-stack-layer-1 album-art-image' />
        </div>
        <Skeleton class='h-6 w-3/4 mb-1' />
        <Skeleton class='h-4 w-1/2' />
      </div>
    </div>
    <div
      v-else
      class='grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4'
    >
      <template v-if="type === 'song'">
        <ContextMenu v-for='song in (items as Song[]).slice(0, 6)' :key='song.id'>
          <ContextMenuTrigger as-child>
            <div
              @click='playSongs(items as Song[], song)'
              class='cursor-pointer group hover:bg-muted/50 rounded-md transition-colors p-2'
            >
              <div class='relative mb-2'>
                <ImageLoader
                  :item-id='song.albumId || song.id'
                  :server-url='serverUrl'
                  :token='token'
                  alt='Album art'
                  class='album-art-image'
                >
                  <template #fallback>
                    <ImagePlaceholder class='album-art-image' size='large' type='album-art' />
                  </template>
                </ImageLoader>

                <!-- Play button overlay -->
                <div
                  class='absolute inset-0 bg-black/50 rounded-lg opacity-0 group-hover:opacity-100
                         transition-opacity flex items-center justify-center'
                >
                  <Button
                    @click.stop='playSongs(items as Song[], song)'
                    class='bg-white/20 hover:bg-white/30 backdrop-blur-sm text-white border border-white/20'
                    size='icon'
                  >
                    <Play class='h-4 w-4' />
                  </Button>
                </div>
              </div>
              <p class='font-semibold truncate'>
                {{ song.name }}
              </p>
              <p class='text-sm text-muted-foreground truncate'>
                <template v-if='song.artists && song.artistIds && song.artists.length === song.artistIds.length'>
                  <template v-for='(artist, index) in song.artists' :key='song.artistIds[index]'>
                    <RouterLink
                      @click.stop
                      :to='`/artists/${song.artistIds[index]}`'
                      class='hover:underline'
                    >
                      {{ artist }}
                    </RouterLink>
                    <span v-if='index < song.artists.length - 1'>, </span>
                  </template>
                </template>
                <template v-else>
                  {{ song.artists?.join(', ') }}
                </template>
              </p>
            </div>
          </ContextMenuTrigger>
          <ContextMenuContent>
            <ContextMenuItem @click='playSongs([song])'>
              <Play class='size-4 mr-2' />Play
            </ContextMenuItem>
            <ContextMenuItem @click='playInstantMix(song)'>
              <Shuffle class='size-4 mr-2' />Instant Mix
            </ContextMenuItem>
          </ContextMenuContent>
        </ContextMenu>
      </template>

      <template v-else-if="type === 'album'">
        <ContextMenu v-for='album in (items as Album[]).slice(0, 6)' :key='album.name'>
          <ContextMenuTrigger as-child>
            <div
              @click="$emit('select-album', album)"
              class='cursor-pointer group hover:bg-muted/50 rounded-md transition-colors p-2'
            >
              <div class='relative mb-2'>
                <AlbumStack
                  @play='playAlbumSongs'
                  :album='album'
                  :disabled='isLoading'
                  :server-url='serverUrl'
                  :size="'responsive'"
                  :token='token'
                />
              </div>
              <p class='font-semibold truncate'>
                {{ album.name }}
              </p>
              <p class='text-sm text-muted-foreground truncate'>
                <template v-if='getAlbumArtists(album).length > 0'>
                  <template v-for='(artist, index) in getAlbumArtists(album)' :key='artist.id'>
                    <RouterLink
                      @click.stop
                      :to='`/artists/${artist.id}`'
                      class='hover:underline'
                    >
                      {{ artist.name }}
                    </RouterLink>
                    <span v-if='index < getAlbumArtists(album).length - 1'>, </span>
                  </template>
                </template>
                <template v-else-if='album.artist'>
                  <RouterLink
                    @click.stop
                    v-if='album.artistId'
                    :to='`/artists/${album.artistId}`'
                    class='hover:underline'
                  >
                    {{ album.artist }}
                  </RouterLink>
                  <span v-else>{{ album.artist }}</span>
                </template>
              </p>
            </div>
          </ContextMenuTrigger>
          <ContextMenuContent>
            <ContextMenuItem @click='playAlbumSongs(album)'>
              <Play class='size-4 mr-2' />Play Album
            </ContextMenuItem>
            <AddToPlaylistMenu
              :songs='album.songs ? [...album.songs].sort((a, b) => (a.trackNumber ?? 0) - (b.trackNumber ?? 0)) : []'
              type='context'
            />
          </ContextMenuContent>
        </ContextMenu>
      </template>
    </div>
  </div>
</template>

<style scoped>
@reference "tailwindcss";

:deep(.album-art-image) {
  @apply w-full h-auto rounded-lg shadow-lg aspect-square object-cover transition-all;
}
</style>
