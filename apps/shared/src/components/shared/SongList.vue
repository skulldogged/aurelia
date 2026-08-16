<script setup lang="ts">
  import { Heart, Pause, Play, Share2, Shuffle } from 'lucide-vue-next'
  import { computed, inject, nextTick, ref, watch } from 'vue'

  import { scrollElementKey } from '../../composables/useMainLayout'
  import { Song } from '../../lib/api/types'
  import { formatDuration } from '../../lib/utils'
  import { usePlayerStore } from '../../stores'
  import Button from '../ui/Button.vue'
  import {
    ContextMenu,
    ContextMenuContent,
    ContextMenuItem,
    ContextMenuTrigger,
  } from '../ui/context-menu'
  import { Skeleton } from '../ui/skeleton'
  import AddToPlaylistMenu from './AddToPlaylistMenu.vue'
  import ImageLoader from './ImageLoader.vue'
  import ImagePlaceholder from './ImagePlaceholder.vue'
  import ShareDialog from './ShareDialog.vue'

  const playerStore = usePlayerStore()

  const props = withDefaults(defineProps<{
    hideHeader?:       boolean
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

  // Playlist functionality
  const showShareDialog = ref(false)
  const shareDialogItem = ref<null | { id: string; name: string; type: 'album' | 'artist' | 'song' }>(null)
  const contextMenuSong = ref<null | Song>(null)

  const removeSongFromPlaylist = (song: Song): void => {
    emit('remove-song', song)
  }

  const openShareDialog = (song: Song): void => {
    shareDialogItem.value = { id: song.id, name: song.name, type: 'song' }
    showShareDialog.value = true
  }

  // Virtual scrolling setup
  // Try to inject parent scroll element, fallback to internal container
  const injectedScrollElement = inject(scrollElementKey, ref<HTMLElement | null>(null))
  const internalScrollContainer = ref<HTMLElement | null>(null)

  // Use injected scroll element if available and valid, otherwise use internal container
  const scrollElement = computed(() => {
    if (injectedScrollElement.value) {
      return injectedScrollElement.value
    }
    return internalScrollContainer.value
  })

  const virtualList = ref<HTMLElement | null>(null)
  const rowSize = computed(() => layoutMode.value === 'comfy' ? 72 : 48)
  const overscan = computed(() => layoutMode.value === 'compact' ? 8 : 6)
  const visibleStart = ref(0)
  const visibleEnd = ref(-1)
  let listOffset = 0

  const updateVisibleRange = (): void => {
    const element = scrollElement.value
    if (!element || props.songs.length === 0) {
      visibleStart.value = 0
      visibleEnd.value = -1
      return
    }

    const relativeScrollTop = Math.max(0, element.scrollTop - listOffset)
    const nextStart = Math.max(0, Math.floor(relativeScrollTop / rowSize.value) - overscan.value)
    const nextEnd = Math.min(
      props.songs.length - 1,
      Math.ceil((relativeScrollTop + element.clientHeight) / rowSize.value) + overscan.value,
    )

    if (nextStart !== visibleStart.value)
      visibleStart.value = nextStart
    if (nextEnd !== visibleEnd.value)
      visibleEnd.value = nextEnd
  }

  const refreshVirtualWindow = async (): Promise<void> => {
    await nextTick()
    const element = scrollElement.value
    if (element && virtualList.value) {
      const scrollerRect = element.getBoundingClientRect()
      const listRect = virtualList.value.getBoundingClientRect()
      listOffset = listRect.top - scrollerRect.top + element.scrollTop
    }
    updateVisibleRange()
  }

  watch(scrollElement, (element, _, onCleanup) => {
    if (!element)
      return

    const resizeObserver = new ResizeObserver(updateVisibleRange)
    element.addEventListener('scroll', updateVisibleRange, { passive: true })
    resizeObserver.observe(element)
    void refreshVirtualWindow()

    onCleanup(() => {
      element.removeEventListener('scroll', updateVisibleRange)
      resizeObserver.disconnect()
    })
  }, { flush: 'post', immediate: true })

  watch([() => props.songs.length, layoutMode], () => {
    void refreshVirtualWindow()
  })

  const totalSize = computed(() => props.songs.length * rowSize.value)
  const virtualSongs = computed(() => {
    const songs: { song: Song; virtualRow: { index: number; start: number } }[] = []
    for (let index = visibleStart.value; index <= visibleEnd.value; index++) {
      const song = props.songs[index]
      if (song)
        songs.push({ song, virtualRow: { index, start: index * rowSize.value } })
    }
    return songs
  })

  // Check if we're using internal scroll container (no injected element)
  const useInternalScroll = computed(() => !injectedScrollElement.value)
</script>

<template>
  <div class='flex-1 flex flex-col h-full'>
    <!-- Use internal scroll container if no parent provides one -->
    <div
      ref='internalScrollContainer'
      :class="[
        'flex-1',
        useInternalScroll ? 'overflow-auto' : 'overflow-visible'
      ]"
    >
      <!-- Table Header -->
      <div v-if='!loading && !hideHeader' class='bg-sidebar px-2 py-1 sticky top-0 z-10'>
        <div
          :class="
            layoutMode === 'compact'
              ? 'flex items-center gap-2 text-xs text-muted-foreground'
              : 'flex items-center gap-2 text-xs text-muted-foreground'
          "
        >
          <div
            v-if='showTrackNumber'
            :class="layoutMode === 'compact' ? 'w-6 text-center font-medium' : 'w-8 text-center font-medium'"
          >
            #
          </div>

          <div v-if='shouldShowAlbumArt' :class="layoutMode === 'compact' ? 'size-8 shrink-0' : 'size-12 shrink-0'" />

          <div class='flex-1 min-w-0'>
            <div class='flex items-center justify-between'>
              <div class='flex-1 min-w-0 font-medium'>
                Song
                <div v-if='showArtist || showAlbum' class='text-xs font-normal opacity-70'>
                  Artist • Album
                </div>
              </div>

              <div
                :class="
                  layoutMode === 'compact'
                    ? 'flex items-center gap-3 ml-2 shrink-0'
                    : 'flex items-center gap-2 ml-4 shrink-0'
                "
              >
                <div v-if='showYear' class='w-10 text-right hidden sm:block'>
                  Year
                </div>
                <div class='w-12 text-right'>
                  Plays
                </div>
                <div v-if='showDuration' class='w-10 text-right font-mono'>
                  Time
                </div>
                <div class='w-8 text-center'>
                  <Heart class='size-3.5 opacity-0' />
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Loading skeletons -->
      <div
        v-if='loading'
        class='w-full'
      >
        <div
          v-for='n in 20'
          :key='`list-skeleton-${n}`'
          :class="{
            'border-b last:border-b-0': layoutMode === 'compact',
            'bg-card p-3': layoutMode === 'comfy'
          }"
        >
          <div :class="layoutMode === 'compact' ? 'flex items-center gap-2 px-2 py-1' : 'flex items-center gap-3'">
            <Skeleton
              v-if='showTrackNumber'
              :class="layoutMode === 'compact' ? 'w-6 h-3' : 'w-8 h-4'"
            />

            <Skeleton
              v-if='shouldShowAlbumArt'
              :class="{
                'size-8': layoutMode === 'compact',
                'size-12 rounded-lg': layoutMode === 'comfy'
              }"
            />

            <div class='flex-1 min-w-0'>
              <div class='flex items-center justify-between'>
                <div class='flex-1 min-w-0'>
                  <Skeleton :class="layoutMode === 'compact' ? 'h-4 w-3/4' : 'h-5 w-3/4'" />
                  <div :class="layoutMode === 'compact' ? 'mt-0.5' : 'mt-1'">
                    <Skeleton :class="layoutMode === 'compact' ? 'h-3 w-1/2' : 'h-4 w-1/2'" />
                  </div>
                </div>

                <div
                  :class="
                    layoutMode === 'compact'
                      ? 'flex items-center gap-3 ml-2 shrink-0'
                      : 'flex items-center gap-2 ml-4'
                  "
                >
                  <Skeleton
                    v-if='showYear'
                    :class="layoutMode === 'compact' ? 'h-3 w-10' : 'h-4 w-10'"
                  />
                  <Skeleton :class="layoutMode === 'compact' ? 'h-3 w-12' : 'h-4 w-12'" />
                  <Skeleton
                    v-if='showDuration'
                    :class="layoutMode === 'compact' ? 'h-3 w-10' : 'h-4 w-10'"
                  />
                  <Skeleton :class="layoutMode === 'compact' ? 'size-5' : 'size-8'" />
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Virtualized song list -->
      <ContextMenu>
        <ContextMenuTrigger as-child>
          <div
        v-if='!loading'
        ref='virtualList'
        :style="{
          height: `${totalSize}px`,
          width: '100%',
          position: 'relative'
        }"
        class='w-full'
      >
        <div
          v-for='{ song, virtualRow } in virtualSongs'
          @contextmenu='contextMenuSong = song'
          :key='song.id'
          :style='{
            position: "absolute",
            top: 0,
            left: 0,
            width: "100%",
            transform: `translateY(${virtualRow.start}px)`
          }'
        >
              <div
                @click="$emit('play-song', song)"
                :class="[
                  'group cursor-pointer transition-colors duration-150 w-full min-w-0 max-w-full',
                  layoutMode === 'comfy'
                    ? 'hover:bg-muted/50 p-3 bg-card'
                    : 'hover:bg-muted/50 border-b last:border-b-0'
                ]"
              >
                <div
                  :class="layoutMode === 'compact'
                    ? 'flex items-center gap-2 px-2 py-1'
                    : 'flex items-center gap-3'"
                >
                  <div
                    v-if='showTrackNumber'
                    :class="[
                      'text-center text-muted-foreground',
                      layoutMode === 'compact' ? 'w-6 text-xs' : 'w-8 text-sm font-medium'
                    ]"
                  >
                    {{ virtualRow.index + 1 }}
                  </div>

                  <div v-if='shouldShowAlbumArt' class='relative shrink-0'>
                    <ImageLoader
                      :class="[
                        'object-cover group-hover:opacity-75 transition-opacity',
                        layoutMode === 'compact' ? 'size-8' : 'size-12 rounded-lg'
                      ]"
                      :item-id='song.albumId || song.id'
                      :server-url='serverUrl'
                      :token='token'
                      alt='Album art'
                    >
                      <template #fallback>
                        <ImagePlaceholder
                          :class="[
                            'group-hover:opacity-75 transition-opacity',
                            layoutMode === 'compact' ? 'size-8' : 'size-12 rounded-lg'
                          ]"
                          size='small'
                          type='album-art'
                        />
                      </template>
                    </ImageLoader>

                    <div
                      :class="[
                        'absolute inset-0 bg-black/50 flex items-center justify-center transition-opacity',
                        layoutMode === 'comfy' ? 'rounded-lg' : '',
                        playerStore.currentSong?.id === song.id && playerStore.isPlaying
                          ? 'opacity-100'
                          : 'opacity-0 group-hover:opacity-100'
                      ]"
                    >
                      <Button
                        @click.stop="$emit('play-song', song)"
                        :class="[
                          'bg-white/40 hover:bg-white/50 text-white border border-white/30',
                          layoutMode === 'compact' ? 'size-6' : 'size-8'
                        ]"
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
                        <h3
                          :class="layoutMode === 'compact'
                            ? 'font-medium text-sm truncate'
                            : 'font-semibold truncate'"
                        >
                          {{ song.name }}
                        </h3>

                        <div
                          v-if='showArtist || showAlbum'
                          :class="[
                            'text-muted-foreground',
                            layoutMode === 'compact' ? 'text-xs mt-0.5' : 'text-sm mt-1'
                          ]"
                        >
                          <div class='flex items-center gap-1 truncate'>
                            <span v-if='showArtist'>
                              <template
                                v-if='song.artists && song.artistIds
                                  && song.artists.length === song.artistIds.length'
                              >
                                <template
                                  v-for='(artist, artistIndex) in song.artists'
                                  :key='song.artistIds![artistIndex]'
                                >
                                  <RouterLink
                                    @click.stop
                                    :to='`/artists/${song.artistIds[artistIndex]}`'
                                    class='hover:underline'
                                  >
                                    {{ artist }}
                                  </RouterLink>
                                  <span v-if='artistIndex < song.artists.length - 1'>, </span>
                                </template>
                              </template>
                              <template v-else>
                                {{ song.artists?.join(', ') || 'Unknown Artist' }}
                              </template>
                            </span>

                            <span v-if='showArtist && showAlbum' class='text-muted-foreground/60'>•</span>

                            <span v-if='showAlbum'>
                              <RouterLink
                                @click.stop
                                v-if='song.album && song.albumId'
                                :to='`/albums/${song.albumId}`'
                                class='hover:underline'
                              >
                                {{ song.album }}
                              </RouterLink>
                              <span v-else>Unknown Album</span>
                            </span>
                          </div>
                        </div>
                      </div>

                      <div
                        :class="
                          layoutMode === 'compact'
                            ? 'flex items-center gap-3 ml-2 shrink-0'
                            : 'flex items-center gap-2 ml-4'
                        "
                      >
                        <div
                          v-if='showYear && song.year'
                          :class="[
                            'text-right text-muted-foreground hidden sm:block whitespace-nowrap',
                            layoutMode === 'compact' ? 'w-10 text-xs' : 'w-10 text-right text-sm'
                          ]"
                        >
                          {{ song.year }}
                        </div>

                        <div
                          :class="[
                            'text-muted-foreground whitespace-nowrap',
                            layoutMode === 'compact' ? 'w-12 text-right text-xs' : 'w-12 text-right text-sm'
                          ]"
                        >
                          {{ song.playCount ?? 0 }}
                        </div>

                        <div
                          v-if='showDuration'
                          :class="[
                            'text-muted-foreground font-mono whitespace-nowrap',
                            layoutMode === 'compact' ? 'w-10 text-right text-xs' : 'w-10 text-right text-sm'
                          ]"
                        >
                          {{ formatDuration(song.duration) }}
                        </div>

                        <div :class="layoutMode === 'compact' ? 'w-8 text-center' : 'w-8 text-center'">
                          <Button
                            @click.stop="$emit('toggle-favorite', song)"
                            :size='layoutMode === "compact" ? "sm" : "icon"'
                            class='shrink-0 hover:text-accent-foreground'
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
              </div>
        </div>
      </div>
        </ContextMenuTrigger>
        <ContextMenuContent>
          <ContextMenuItem
            @click='contextMenuSong && $emit("play-song", contextMenuSong)'
            :disabled='!contextMenuSong'
          >
            <Play class='size-4 mr-2' />
            Play
          </ContextMenuItem>
          <ContextMenuItem
            @click='contextMenuSong && $emit("play-instant-mix", contextMenuSong)'
            :disabled='!contextMenuSong'
          >
            <Shuffle class='size-4 mr-2' />
            Instant Mix
          </ContextMenuItem>
          <AddToPlaylistMenu
            v-if='shouldShowAddButton'
            :songs='contextMenuSong ? [contextMenuSong] : []'
            type='context'
          />
          <ContextMenuItem
            @click='contextMenuSong && openShareDialog(contextMenuSong)'
            :disabled='!contextMenuSong'
          >
            <Share2 class='size-4 mr-2' />
            Share
          </ContextMenuItem>
          <ContextMenuItem
            @click='contextMenuSong && removeSongFromPlaylist(contextMenuSong)'
            v-if='showRemoveButton'
            :disabled='!contextMenuSong'
          >
            <Heart class='size-4 mr-2' />
            Remove
          </ContextMenuItem>
        </ContextMenuContent>
      </ContextMenu>
    </div>
  </div>

  <ShareDialog
    v-model:open='showShareDialog'
    :item-id='shareDialogItem?.id || ""'
    :item-name='shareDialogItem?.name || ""'
    :item-type='shareDialogItem?.type || "song"'
  />
</template>
