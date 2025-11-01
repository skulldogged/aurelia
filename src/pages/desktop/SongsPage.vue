<script setup lang="ts">
  import { useVirtualizer } from '@tanstack/vue-virtual'
  import { refDebounced } from '@vueuse/core'
  import Fuse from 'fuse.js'
  import { Heart, Pause, Play, Share2, Shuffle } from 'lucide-vue-next'
  import { computed, inject, onMounted, onUnmounted, ref, Ref, watch } from 'vue'

  import type { Credentials, Song } from '@/bindings'
  import type { LayoutMode } from '@/composables/useLayoutPreference'

  import SongsPageTopBar from '@/components/desktop/SongsPageTopBar.vue'
  import AddToPlaylistMenu from '@/components/shared/AddToPlaylistMenu.vue'
  import ImageLoader from '@/components/shared/ImageLoader.vue'
  import ImagePlaceholder from '@/components/shared/ImagePlaceholder.vue'
  import ShareDialog from '@/components/shared/ShareDialog.vue'
  import Button from '@/components/ui/Button.vue'
  import { ContextMenu, ContextMenuContent, ContextMenuItem, ContextMenuTrigger } from '@/components/ui/context-menu'
  import { useLayoutPreference, useSortPreference } from '@/composables/useLayoutPreference'
  import { scrollElementKey } from '@/composables/useMainLayout'
  import { useTopBar } from '@/composables/useTopBar'
  import { formatDuration } from '@/lib/utils'
  import { useAuthStore } from '@/stores/auth'
  import { useLibraryStore } from '@/stores/library'
  import { usePlayerStore } from '@/stores/player'

  defineProps<{
    credentials: Credentials
  }>()

  defineEmits<{
    'play-instant-mix': [song: Song]
    'play-song':        [song: Song]
    'toggle-favorite':  [song: Song]
  }>()

  const authStore = useAuthStore()
  const libraryStore = useLibraryStore()
  const playerStore = usePlayerStore()
  const { clearTopBarContent, setTopBarContent } = useTopBar()

  // Create computed properties from stores
  const allSongs = computed(() => libraryStore.allSongs as Song[])
  const libraryLoading = computed(() => libraryStore.isLoading)
  const serverUrl = computed(() => authStore.serverUrl)
  const token = computed(() => authStore.token)

  const showShareDialog = ref(false)
  const shareDialogItem = ref<null | { id: string; name: string; type: 'album' | 'artist' | 'song' }>(null)
  const openShareDialog = (song: Song): void => {
    shareDialogItem.value = { id: song.id, name: song.name, type: 'song' }
    showShareDialog.value = true
  }

  const searchQuery = ref('')
  const debouncedSearchQuery = refDebounced(searchQuery, 300)

  const { layout: viewLayout } = useLayoutPreference('songlist-layout', 'comfy')
  const { sort: sortOption } = useSortPreference('songlist-sort', 'Title')

  const sortingOptions = ['Title', 'Artist', 'Album', 'Date Added', 'Play Count']

  const songFuse = ref<Fuse<Song>>()

  watch(() => allSongs.value, newSongs => {
    if (newSongs && newSongs.length > 0) {
      songFuse.value = new Fuse(newSongs as Song[], {
        includeScore: true,
        keys:         [
          { name: 'name', weight: 0.5 },
          { name: 'artists', weight: 0.3 },
          { name: 'album', weight: 0.2 },
        ],
        minMatchCharLength: 2,
        threshold:          0.2,
      })
    }
  })

  const filteredSongs = computed(() =>
    debouncedSearchQuery.value && debouncedSearchQuery.value.length >= 2 && songFuse.value
      ? songFuse.value.search(debouncedSearchQuery.value).map(result => result.item)
      : allSongs.value as Song[],
  )

  const sortedSongs = ref<Song[]>([])

  const scrollElement = inject(scrollElementKey) as Ref<HTMLElement | null>

  watch([filteredSongs, sortOption], ([newFilteredSongs, newSortOption]) => {
    const songsToSort = [...newFilteredSongs]
    switch (newSortOption) {
      case 'Album':
        songsToSort.sort((a, b) => (a.album || '').localeCompare(b.album || ''))
        break
      case 'Artist':
        songsToSort.sort((a, b) => (a.artists?.[0] || '').localeCompare(b.artists?.[0] || ''))
        break
      case 'Date Added':
        songsToSort.sort((a, b) => (b.dateCreated || '').localeCompare(a.dateCreated || ''))
        break
      case 'Play Count':
        songsToSort.sort((a, b) => (b.playCount || 0) - (a.playCount || 0))
        break
      case 'Title':
        songsToSort.sort((a, b) => a.name.localeCompare(b.name))
        break
    }
    sortedSongs.value = songsToSort
  }, { immediate: true })

  const estimateSize = computed(() => viewLayout.value === 'comfy' ? 72 : 48)

  const rowVirtualizer = useVirtualizer({
    count:            sortedSongs.value.length,
    estimateSize:     () => estimateSize.value,
    getScrollElement: () => scrollElement.value,
    overscan:         2,
  })

  watch(sortedSongs, () => {
    rowVirtualizer.value.measure()
  })

  watch(viewLayout, () => {
    rowVirtualizer.value.measure()
  })

  const virtualRows = computed(() => rowVirtualizer.value.getVirtualItems())

  const virtualSongs = computed(() =>
    virtualRows.value.map(row => ({
      ...sortedSongs.value[row.index],
      virtualRow: row,
    })),
  )

  // Set the top bar content when the component mounts
  onMounted(() => {
    document.body.classList.add('songs-page-active')
    setTopBarContent({
      component: SongsPageTopBar,
      id:        'songs-page',
      props:     {
        'onUpdate:searchQuery': (value: string) => {
          searchQuery.value = value
        },
        'onUpdate:sortOption': (value: string) => {
          sortOption.value = value
        },
        'onUpdate:viewLayout': (value: LayoutMode) => {
          viewLayout.value = value
        },
        searchQuery: searchQuery.value,
        sortingOptions,
        sortOption:  sortOption.value,
        viewLayout:  viewLayout.value,
      },
    })
  })

  // Update top bar props when reactive values change
  watch([searchQuery, sortOption, viewLayout], () => {
    setTopBarContent({
      component: SongsPageTopBar,
      id:        'songs-page',
      props:     {
        'onUpdate:searchQuery': (value: string) => {
          searchQuery.value = value
        },
        'onUpdate:sortOption': (value: string) => {
          sortOption.value = value
        },
        'onUpdate:viewLayout': (value: LayoutMode) => {
          viewLayout.value = value
        },
        searchQuery: searchQuery.value,
        sortingOptions,
        sortOption:  sortOption.value,
        viewLayout:  viewLayout.value,
      },
    })
  })

  // Clean up top bar content when unmounting
  onUnmounted(() => {
    document.body.classList.remove('songs-page-active')
    clearTopBarContent()
  })
</script>

<template>
  <div>
    <div class='h-screen flex flex-col'>
      <div class='w-full flex-1 flex flex-col'>
        <div class='flex-1'>
          <div v-if='!libraryLoading' class='bg-sidebar/80 backdrop-blur-sm px-2 py-1 sticky top-0 z-10'>
            <div
              :class="
                viewLayout === 'compact'
                  ? 'flex items-center gap-2 text-xs text-muted-foreground'
                  : 'flex items-center gap-2 text-xs text-muted-foreground'
              "
            >
              <div
                :class="viewLayout === 'compact' ? 'w-6 text-center font-medium' : 'w-8 text-center font-medium'"
              >
                #
              </div>

              <div :class="viewLayout === 'compact' ? 'size-8 shrink-0' : 'size-12 shrink-0'" />

              <div class='flex-1 min-w-0'>
                <div class='flex items-center justify-between'>
                  <div class='flex-1 min-w-0 font-medium'>
                    Song
                    <div class='text-xs font-normal opacity-70'>
                      Artist • Album
                    </div>
                  </div>

                  <div
                    :class="
                      viewLayout === 'compact'
                        ? 'flex items-center gap-3 ml-2 shrink-0'
                        : 'flex items-center gap-2 ml-4 shrink-0'
                    "
                  >
                    <div class='w-10 text-right hidden sm:block'>
                      Year
                    </div>
                    <div class='w-12 text-right'>
                      Plays
                    </div>
                    <div class='w-10 text-right font-mono'>
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
          <div :style="{ height: `${rowVirtualizer.getTotalSize()}px`, width: '100%', position: 'relative' }">
            <div
              v-for='song in virtualSongs'
              :key='String(song.virtualRow.key)'
              :style='{ transform: `translateY(${song.virtualRow.start}px)` }'
              class='absolute top-0 left-0 w-full'
            >
              <ContextMenu v-if='song'>
                <ContextMenuTrigger>
                  <div
                    @click="$emit('play-song', song)"
                    :class="[
                      'group cursor-pointer transition-all duration-200 w-full min-w-0 max-w-full',
                      viewLayout === 'comfy'
                        ? 'hover:bg-muted/50 p-3 bg-card'
                        : 'hover:bg-muted/50 border-b last:border-b-0'
                    ]"
                  >
                    <div
                      :class="
                        viewLayout === 'compact'
                          ? 'flex items-center gap-2 px-2 py-1'
                          : 'flex items-center gap-3'
                      "
                    >
                      <div
                        :class="[
                          'text-center text-muted-foreground',
                          viewLayout === 'compact' ? 'w-6 text-xs' : 'w-8 text-sm font-medium'
                        ]"
                      >
                        {{ song.virtualRow.index + 1 }}
                      </div>
                      <div class='relative shrink-0'>
                        <ImageLoader
                          :class="[
                            'object-cover group-hover:opacity-75 transition-opacity',
                            viewLayout === 'compact' ? 'size-8' : 'size-12 rounded-lg'
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
                                viewLayout === 'compact' ? 'size-8' : 'size-12 rounded-lg'
                              ]"
                              size='small'
                              type='album-art'
                            />
                          </template>
                        </ImageLoader>
                        <div
                          :class="[
                            'absolute inset-0 bg-black/50 flex items-center justify-center transition-opacity',
                            viewLayout === 'comfy' ? 'rounded-lg' : '',
                            playerStore.currentSong?.id === song.id && playerStore.isPlaying
                              ? 'opacity-100'
                              : 'opacity-0 group-hover:opacity-100'
                          ]"
                        >
                          <Button
                            @click.stop="$emit('play-song', song)"
                            :class="[
                              'bg-white/20 hover:bg-white/30 backdrop-blur-sm text-white border border-white/20',
                              viewLayout === 'compact' ? 'size-6' : 'size-8'
                            ]"
                            size='icon'
                          >
                            <Pause
                              v-if='playerStore.currentSong?.id === song.id && playerStore.isPlaying'
                              :class="viewLayout === 'compact' ? 'h-2.5 w-2.5' : 'h-3 w-3'"
                            />
                            <Play
                              v-else
                              :class="viewLayout === 'compact' ? 'h-2.5 w-2.5' : 'h-3 w-3'"
                            />
                          </Button>
                        </div>
                      </div>
                      <div class='flex-1 min-w-0'>
                        <div class='flex items-center justify-between'>
                          <div class='flex-1 min-w-0 overflow-hidden'>
                            <h3 :class="viewLayout === 'compact' ? 'font-medium text-sm' : 'font-semibold'">
                              {{ song.name }}
                            </h3>
                            <div
                              :class="[
                                'text-muted-foreground',
                                viewLayout === 'compact' ? 'text-xs mt-0.5' : 'text-sm mt-1'
                              ]"
                            >
                              <div class='flex items-center gap-1 truncate'>
                                <span>
                                  <template
                                    v-if='song.artists && song.artistIds &&
                                      song.artists.length === song.artistIds.length'
                                  >
                                    <template
                                      v-for='(artist, artistIndex) in song.artists'
                                      :key='song.artistIds[artistIndex]'
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
                                <span class='text-muted-foreground/60'>•</span>
                                <span>
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
                              viewLayout === 'compact'
                                ? 'flex items-center gap-3 ml-2 shrink-0'
                                : 'flex items-center gap-2 ml-4'
                            "
                          >
                            <div
                              v-if='song.year'
                              :class="[
                                'text-right text-muted-foreground hidden sm:block whitespace-nowrap',
                                viewLayout === 'compact' ? 'w-10 text-xs' : 'w-10 text-right text-sm'
                              ]"
                            >
                              {{ song.year }}
                            </div>
                            <div
                              :class="[
                                'text-muted-foreground whitespace-nowrap',
                                viewLayout === 'compact' ? 'w-12 text-right text-xs' : 'w-12 text-right text-sm'
                              ]"
                            >
                              {{ song.playCount ?? 0 }}
                            </div>
                            <div
                              :class="[
                                'text-muted-foreground font-mono whitespace-nowrap',
                                viewLayout === 'compact' ? 'w-10 text-right text-xs' : 'w-10 text-right text-sm'
                              ]"
                            >
                              {{ formatDuration(song.duration) }}
                            </div>
                            <div :class="viewLayout === 'compact' ? 'w-8 text-center' : 'w-8 text-center'">
                              <Button
                                @click.stop="$emit('toggle-favorite', song)"
                                :size='viewLayout === "compact" ? "sm" : "icon"'
                                class='shrink-0 hover:text-accent-foreground'
                                variant='ghost'
                              >
                                <Heart
                                  :class="[
                                    viewLayout === 'compact' ? 'size-3.5' : 'size-5',
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
                  <AddToPlaylistMenu :songs='[song]' type='context' />
                  <ContextMenuItem @click='openShareDialog(song)'>
                    <Share2 class='size-4 mr-2' />
                    Share
                  </ContextMenuItem>
                </ContextMenuContent>
              </ContextMenu>
            </div>
          </div>
        </div>
      </div>
    </div>
    <ShareDialog
      v-model:open='showShareDialog'
      :item-id='shareDialogItem?.id || ""'
      :item-name='shareDialogItem?.name || ""'
      :item-type='shareDialogItem?.type || "song"'
    />
  </div>
</template>