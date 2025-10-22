<script setup lang="ts">
  import { Heart, Pause, Play, Share2, Shuffle } from 'lucide-vue-next'
  import { computed, ref, watch } from 'vue'

  import { Song } from '@/bindings'
  import Button from '@/components/ui/Button.vue'
  import {
    ContextMenu,
    ContextMenuContent,
    ContextMenuItem,
    ContextMenuTrigger,
  } from '@/components/ui/context-menu'
  import {
    Select,
    SelectContent,
    SelectGroup,
    SelectItem,
    SelectTrigger,
    SelectValue,
  } from '@/components/ui/select'
  import { Skeleton } from '@/components/ui/skeleton'
  import { usePageSizePreference } from '@/composables/useLayoutPreference'
  import { formatDuration } from '@/lib/utils'
  import { usePlayerStore } from '@/stores'

  import AddToPlaylistMenu from './AddToPlaylistMenu.vue'
  import ImageLoader from './ImageLoader.vue'
  import ImagePlaceholder from './ImagePlaceholder.vue'
  import ShareDialog from './ShareDialog.vue'

  const playerStore = usePlayerStore()

  const props = withDefaults(defineProps<{
    layout?:           'comfy' | 'compact'
    loading?:          boolean
    serverUrl:         string
    showAddButton?:    boolean
    showAlbum?:        boolean
    showAlbumArt?:     boolean
    showArtist?:       boolean
    showDuration?:     boolean
    showRemoveButton?: boolean
    showTrackNumber?:  boolean
    showYear?:         boolean
    songs:             Song[]
    token:             string
  }>(), {
    layout:        'comfy',
    showAddButton: true,
  })

  const shouldShowAlbumArt = computed(() => props.showAlbumArt !== false)
  const shouldShowAddButton = computed(() => props.showAddButton === undefined ? true : props.showAddButton)
  const layoutMode = computed(() => props.layout || 'comfy')

  const emit = defineEmits<{
    'add-song':         [song: Song]
    'play-instant-mix': [song: Song]
    'play-song':        [song: Song]
    'remove-song':      [song: Song]
    'toggle-favorite':  [song: Song]
  }>()

  const pageIndex = ref(0)
  const { pageSize } = usePageSizePreference('songlist-pagesize', 20)

  watch(
    () => props.songs,
    () => {
      pageIndex.value = 0
    },
    { deep: false },
  )

  const pageCount = computed(() => Math.max(1, Math.ceil((props.songs?.length ?? 0) / pageSize.value)))

  const canPreviousPage = computed(() => pageIndex.value > 0)
  const canNextPage = computed(() => pageIndex.value < pageCount.value - 1)

  const pagedSongs = computed(() =>
    props.songs.slice((pageIndex.value * pageSize.value), (pageIndex.value * pageSize.value + pageSize.value)),
  )

  const previousPage = (): void => {
    if (canPreviousPage.value) pageIndex.value -= 1
  }

  const nextPage = (): void => {
    if (canNextPage.value) pageIndex.value += 1
  }

  const setPageSize = (value: number): void => {
    const oldStart = pageIndex.value * pageSize.value
    pageSize.value = value
    pageIndex.value = Math.floor(oldStart / pageSize.value)
  }

  const pageSizeOptions = [10, 20, 30, 50]

  // Playlist functionality
  const showShareDialog = ref(false)
  const shareDialogItem = ref<null | { id: string; name: string; type: 'album' | 'artist' | 'song' }>(null)

  const removeSongFromPlaylist = (song: Song): void => {
    emit('remove-song', song)
  }

  const openShareDialog = (song: Song): void => {
    shareDialogItem.value = { id: song.id, name: song.name, type: 'song' }
    showShareDialog.value = true
  }
</script>

<template>
  <div class='bg-sidebar rounded-lg p-6'>
    <div class='space-y-4 w-full'>
      <div
        v-if='loading'
        class='space-y-2 w-full max-w-full'
      >
        <div
          v-for='n in pageSize'
          :key='`list-skeleton-${n}`'
          :class="{
            'rounded-lg p-3': layoutMode === 'comfy',
            'rounded-md px-2 py-1.5': layoutMode === 'compact'
          }"
        >
          <div class='flex items-center gap-3'>
            <Skeleton v-if='showTrackNumber' class='w-8 h-4' />

            <Skeleton
              v-if='shouldShowAlbumArt'
              :class="{
                'size-12 rounded-lg': layoutMode === 'comfy',
                'size-8 rounded-md': layoutMode === 'compact'
              }"
            />

            <div class='flex-1 min-w-0'>
              <div class='flex items-center justify-between'>
                <div class='flex-1 min-w-0'>
                  <Skeleton :class="layoutMode === 'compact' ? 'h-4 w-3/4' : 'h-5 w-3/4'" />
                  <div
                    v-if='showArtist || showAlbum'
                    :class="layoutMode === 'compact' ? 'space-y-1' : 'mt-1 space-y-1'"
                  >
                    <Skeleton :class="layoutMode === 'compact' ? 'h-3 w-1/2' : 'h-4 w-1/2'" />
                  </div>
                </div>

                <div class='flex items-center gap-2 ml-4'>
                  <Skeleton v-if='showYear' class='h-4 w-12' />
                  <Skeleton class='h-4 w-16' />
                  <Skeleton v-if='showDuration' class='h-4 w-12' />
                  <Skeleton class='size-8 rounded' />
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div
        v-else
        class='space-y-2 w-full max-w-full'
      >
        <ContextMenu v-for='(song, index) in pagedSongs' :key='song.id'>
          <ContextMenuTrigger>
            <div
              @click="$emit('play-song', song)"
              :class="layoutMode === 'comfy'
                ? 'group cursor-pointer hover:bg-muted/50 rounded-lg p-3 ' +
                  'transition-all duration-200 w-full min-w-0 max-w-full'
                : 'group cursor-pointer hover:bg-muted/30 rounded-md px-2 py-1.5 ' +
                  'transition-all duration-200 w-full min-w-0 max-w-full'"
            >
              <div class='flex items-center gap-3 min-w-0'>
                <div
                  v-if='showTrackNumber'
                  class='w-8 text-center text-sm text-muted-foreground font-medium'
                >
                  {{ pageIndex * pageSize + index + 1 }}
                </div>

                <div v-if='shouldShowAlbumArt' class='relative flex-shrink-0'>
                  <ImageLoader
                    :class="layoutMode === 'comfy'
                      ? 'size-12 rounded-lg object-cover ' +
                        'group-hover:opacity-75 transition-opacity'
                      : 'size-8 rounded-md object-cover group-hover:opacity-75 transition-opacity'"
                    :item-id='song.albumId || song.id'
                    :server-url='serverUrl'
                    :token='token'
                    alt='Album art'
                  >
                    <template #fallback>
                      <ImagePlaceholder
                        :class="{
                          'size-12 rounded-lg group-hover:opacity-75 transition-opacity': layoutMode === 'comfy',
                          'size-8 rounded-md group-hover:opacity-75 transition-opacity': layoutMode === 'compact'
                        }"
                        size='small'
                        type='album-art'
                      />
                    </template>
                  </ImageLoader>

                  <div
                    :class="[
                      'absolute inset-0 bg-black/50 rounded-lg flex items-center justify-center transition-opacity',
                      playerStore.currentSong?.id === song.id && playerStore.isPlaying
                        ? 'opacity-100'
                        : 'opacity-0 group-hover:opacity-100'
                    ]"
                  >
                    <Button
                      @click.stop="$emit('play-song', song)"
                      :class="layoutMode === 'comfy'
                        ? 'bg-white/20 hover:bg-white/30 backdrop-blur-sm text-white ' +
                          'border border-white/20 size-8'
                        : 'bg-white/20 hover:bg-white/30 backdrop-blur-sm text-white ' +
                          'border border-white/20 size-6'"
                      size='icon'
                    >
                      <Pause
                        v-if='playerStore.currentSong?.id === song.id && playerStore.isPlaying'
                        :class="layoutMode === 'compact' ? 'h-2.5 w-2.5' : 'h-3 w-3'"
                      />
                      <Play
                        v-else
                        :class="layoutMode === 'compact' ? 'h-2.5 w-2.5' : 'h-3 w-3'"
                      />
                    </Button>
                  </div>
                </div>

                <div class='flex-1 min-w-0'>
                  <div class='flex items-center justify-between'>
                    <div class='flex-1 min-w-0 overflow-hidden'>
                      <h3 :class="layoutMode === 'compact' ? 'font-medium text-sm' : 'font-semibold'" class='truncate'>
                        {{ song.name }}
                      </h3>

                      <div
                        v-if='showArtist || showAlbum'
                        :class="layoutMode === 'compact' ? 'text-xs' : 'text-sm mt-1'"
                        class='text-muted-foreground'
                      >
                        <div class='flex items-center gap-1 truncate'>
                          <span v-if='showArtist'>
                            <template
                              v-if='song.artists && song.artistIds &&
                                song.artists.length === song.artistIds.length'
                            >
                              <template
                                v-for='(artist, artistIndex) in song.artists'
                                :key='song.artistIds[artistIndex]'
                              >
                                <router-link
                                  @click.stop
                                  :to='`/songs/artist/${song.artistIds[artistIndex]}`'
                                  class='hover:underline'
                                >
                                  {{ artist }}
                                </router-link>
                                <span v-if='artistIndex < song.artists.length - 1'>, </span>
                              </template>
                            </template>
                            <template v-else>
                              {{ song.artists?.join(', ') || 'Unknown Artist' }}
                            </template>
                          </span>

                          <span v-if='showArtist && showAlbum' class='text-muted-foreground/60'>•</span>

                          <span v-if='showAlbum'>
                            <router-link
                              @click.stop
                              v-if='song.album && song.albumId'
                              :to='`/songs/album/${song.albumId}`'
                              class='hover:underline'
                            >
                              {{ song.album }}
                            </router-link>
                            <span v-else>Unknown Album</span>
                          </span>
                        </div>
                      </div>
                    </div>

                    <div class='flex items-center gap-2 ml-4 flex-shrink-0'>
                      <span
                        v-if='showYear && song.year'
                        :class="layoutMode === 'compact' ? 'text-xs' : 'text-sm'"
                        class='text-muted-foreground hidden sm:block whitespace-nowrap'
                      >
                        {{ song.year }}
                      </span>

                      <span
                        :class="layoutMode === 'compact' ? 'text-xs' : 'text-sm'"
                        class='text-muted-foreground whitespace-nowrap'
                      >
                        {{ song.playCount ?? 0 }} plays
                      </span>

                      <span
                        v-if='showDuration'
                        :class="layoutMode === 'compact' ? 'text-xs' : 'text-sm'"
                        class='text-muted-foreground font-mono whitespace-nowrap'
                      >
                        {{ formatDuration(song.duration) }}
                      </span>

                      <Button
                        @click.stop="$emit('toggle-favorite', song)"
                        :size='layoutMode === "compact" ? "sm" : "icon"'
                        class='flex-shrink-0 hover:text-accent-foreground'
                        variant='ghost'
                      >
                        <Heart
                          :class="[
                            layoutMode === 'compact' ? 'size-3.5' : 'size-5',
                            song.isFavorite ? 'fill-current' : ''
                          ]"
                        />
                      </Button>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </ContextMenuTrigger>
          <ContextMenuContent>
            <ContextMenuItem @click="$emit('play-song', song)">
              <Play class='size-4 mr-2' />
              Play
            </ContextMenuItem>
            <ContextMenuItem @click="$emit('play-instant-mix', song)">
              <Shuffle class='size-4 mr-2' />
              Instant Mix
            </ContextMenuItem>
            <AddToPlaylistMenu v-if='shouldShowAddButton' :songs='[song]' type='context' />
            <ContextMenuItem @click='openShareDialog(song)'>
              <Share2 class='size-4 mr-2' />
              Share
            </ContextMenuItem>
            <ContextMenuItem @click='removeSongFromPlaylist(song)' v-if='showRemoveButton'>
              <Heart class='size-4 mr-2' />
              Remove
            </ContextMenuItem>
          </ContextMenuContent>
        </ContextMenu>
      </div>

      <div
        v-if='loading || pageCount > 1'
        class='flex flex-col sm:flex-row items-center justify-between gap-4'
      >
        <template v-if='loading'>
          <div class='flex items-center gap-2'>
            <Skeleton class='h-4 w-32' />
            <Skeleton class='h-9 w-20 rounded-md' />
          </div>

          <div class='flex items-center gap-2'>
            <Skeleton class='h-4 w-24' />
            <div class='flex items-center gap-1'>
              <Skeleton class='h-9 w-16 rounded-md' />
              <Skeleton class='h-9 w-20 rounded-md' />
              <Skeleton class='h-9 w-16 rounded-md' />
              <Skeleton class='h-9 w-16 rounded-md' />
            </div>
          </div>
        </template>

        <template v-else>
          <div class='flex items-center gap-2'>
            <span class='text-sm text-muted-foreground'>Songs per page:</span>
            <Select @update:model-value='(v) => setPageSize(Number(v))' :model-value='String(pageSize)'>
              <SelectTrigger class='w-[80px]'>
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
              Page {{ pageIndex + 1 }} of {{ pageCount }}
            </span>

            <div class='flex items-center gap-1'>
              <Button
                @click='pageIndex = 0'
                :disabled='!canPreviousPage'
                class='h-9 px-3'
                size='sm'
                variant='outline'
              >
                First
              </Button>
              <Button
                @click='previousPage'
                :disabled='!canPreviousPage'
                class='h-9 px-3'
                size='sm'
                variant='outline'
              >
                Previous
              </Button>
              <Button
                @click='nextPage'
                :disabled='!canNextPage'
                class='h-9 px-3'
                size='sm'
                variant='outline'
              >
                Next
              </Button>
              <Button
                @click='pageIndex = pageCount - 1'
                :disabled='!canNextPage'
                class='h-9 px-3'
                size='sm'
                variant='outline'
              >
                Last
              </Button>
            </div>
          </div>
        </template>
      </div>
    </div>
  </div>

  <ShareDialog
    v-model:open='showShareDialog'
    :item-id='shareDialogItem?.id || ""'
    :item-name='shareDialogItem?.name || ""'
    :item-type='shareDialogItem?.type || "song"'
  />
</template>
