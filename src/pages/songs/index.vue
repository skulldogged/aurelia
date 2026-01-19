<script setup lang="ts">
  import { refDebounced } from '@vueuse/core'
  import { computed, onMounted, onUnmounted, ref } from 'vue'

  import type { Credentials, Song } from '@/lib/api/bindings'

  import SongList from '@/components/shared/SongList.vue'
  import { useDebouncedTopBar } from '@/composables/useDebouncedTopBar'
  import { useLayoutPreference, useSortPreference } from '@/composables/useLayoutPreference'
  import { useMemoizedSearch } from '@/composables/useMemoizedSearch'
  import { useMemoizedSort } from '@/composables/useMemoizedSort'
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

  // Use memoized sorting for better performance
  const { sortedSongs } = useMemoizedSort({
    songs:      filteredSongs,
    sortOption: sortOption,
  })

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
  <div class='h-full flex flex-col'>
    <SongList
      @play-instant-mix="$emit('play-instant-mix', $event)"
      @play-song="$emit('play-song', $event)"
      @toggle-favorite="$emit('toggle-favorite', $event)"
      :layout='viewLayout'
      :loading='libraryLoading'
      :server-url='serverUrl'
      :show-album='true'
      :show-album-art='true'
      :show-artist='true'
      :show-duration='true'
      :show-track-number='true'
      :show-year='true'
      :songs='sortedSongs'
      :token='token'
    />
  </div>
</template>
