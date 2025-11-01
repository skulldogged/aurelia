<script setup lang="ts">
  import { refDebounced } from '@vueuse/core'
  import { computed, inject, onMounted, onUnmounted, ref, Ref, watch } from 'vue'

  import type { Credentials, Song } from '@/bindings'

  import SongListItem from '@/components/desktop/SongListItem.vue'
  import ShareDialog from '@/components/shared/ShareDialog.vue'
  import { useDebouncedTopBar } from '@/composables/useDebouncedTopBar'
  import { useLayoutPreference, useSortPreference } from '@/composables/useLayoutPreference'
  import { scrollElementKey } from '@/composables/useMainLayout'
  import { useMemoizedSearch } from '@/composables/useMemoizedSearch'
  import { useMemoizedSort } from '@/composables/useMemoizedSort'
  import { useOptimizedVirtualScroller } from '@/composables/useOptimizedVirtualScroller'
  import { useAuthStore } from '@/stores/auth'
  import { useLibraryStore } from '@/stores/library'

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

  // Use debounced top bar for better performance
  const { clearTopBarContent } = useDebouncedTopBar(searchQuery, sortOption, viewLayout, sortingOptions)

  // Use memoized search for better performance
  const { searchResults: filteredSongs } = useMemoizedSearch({
    searchQuery: debouncedSearchQuery,
    songs:       allSongs,
  })

  const scrollElement = inject(scrollElementKey) as Ref<HTMLElement | null>

  // Use memoized sorting for better performance
  const { sortedSongs } = useMemoizedSort({
    songs:      filteredSongs,
    sortOption: sortOption,
  })

  // Use optimized virtual scroller with dynamic overscan
  const estimateSize = computed(() => viewLayout.value === 'comfy' ? 72 : 48)
  const songCount = computed(() => sortedSongs.value.length)

  const {
    remeasure,
    rowVirtualizer,
    virtualItems,
  } = useOptimizedVirtualScroller({
    count: songCount,
    estimateSize,
    scrollElement,
    viewLayout,
  })

  // Watch for changes and remeasure
  watch([sortedSongs, viewLayout], () => {
    remeasure()
  })

  const virtualSongs = computed(() =>
    virtualItems.value.map(item => ({
      ...sortedSongs.value[item.index],
      virtualRow: item.virtualRow,
    })),
  )

  // Set up component lifecycle
  onMounted(() => {
    document.body.classList.add('songs-page-active')
  })

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
            <!-- Match SongListItem structure exactly -->
            <div
              :class="
                viewLayout === 'compact'
                  ? 'flex items-center gap-2 text-xs text-muted-foreground'
                  : 'flex items-center gap-3 text-xs text-muted-foreground px-1'
              "
            >
              <!-- Number column - exact match -->
              <div
                :class="[
                  'text-center text-muted-foreground font-medium',
                  viewLayout === 'compact' ? 'w-6 text-xs' : 'w-8 text-sm'
                ]"
              >
                #
              </div>

              <!-- Image placeholder - width only for alignment -->
              <div
                :class="[
                  'shrink-0',
                  viewLayout === 'compact' ? 'w-8' : 'w-12'
                ]"
              />

              <!-- Content area - exact match -->
              <div class='flex-1 min-w-0'>
                <div class='flex items-center justify-between'>
                  <div class='flex-1 min-w-0 font-medium'>
                    Song
                    <div class='text-xs opacity-70'>
                      Artist • Album
                    </div>
                  </div>

                  <!-- Metadata columns - exact match -->
                  <div
                    :class="[
                      'flex items-center',
                      viewLayout === 'compact' ? 'gap-3 ml-2' : 'gap-2 ml-4'
                    ]"
                  >
                    <div class='w-8 text-right hidden sm:block'>
                      Year
                    </div>
                    <div class='w-8 text-right'>
                      Plays
                    </div>
                    <div class='w-8 text-right'>
                      Time
                    </div>
                    <div class='w-8 text-center'>
                      <div class='size-3.5 opacity-0' />
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
              <SongListItem
                @play-instant-mix="$emit('play-instant-mix', $event)"
                @play-song="$emit('play-song', $event)"
                @share-song='openShareDialog($event)'
                @toggle-favorite="$emit('toggle-favorite', $event)"
                v-if='song'
                :index='song.virtualRow.index'
                :server-url='serverUrl'
                :song='song'
                :token='token'
                :view-layout='viewLayout'
              />
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