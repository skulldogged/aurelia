import type { Ref } from 'vue'

import { refDebounced } from '@vueuse/core'
import { onMounted, watch } from 'vue'

import SongsPageTopBar from '../components/desktop/SongsPageTopBar.vue'
import { useTopBar } from './useTopBar'

interface TopBarProps {
  searchQuery:    string
  sortingOptions: string[]
  sortOption:     string
  viewLayout:     string
}

export const useDebouncedTopBar = (
  searchQuery: Ref<string>,
  sortOption: Ref<string>,
  viewLayout: Ref<string>,
  sortingOptions: string[],
  debounceMs = 200,
): { clearTopBarContent: () => void } => {
  const { clearTopBarContent, setTopBarContent } = useTopBar()

  // Create debounced versions of the reactive values
  const debouncedSearchQuery = refDebounced(searchQuery, debounceMs)
  const debouncedSortOption = refDebounced(sortOption, debounceMs)
  const debouncedViewLayout = refDebounced(viewLayout, debounceMs)

  const updateTopBar = (): void => {
    const topBarProps: TopBarProps = {
      searchQuery: debouncedSearchQuery.value,
      sortingOptions,
      sortOption:  debouncedSortOption.value,
      viewLayout:  debouncedViewLayout.value,
    }

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
        'onUpdate:viewLayout': (value: string) => {
          viewLayout.value = value
        },
        ...topBarProps,
      },
    })
  }

  // Set up top bar content immediately on mount
  onMounted(() => {
    updateTopBar()
  })

  // Update top bar content when debounced values change
  watch(
    [debouncedSearchQuery, debouncedSortOption, debouncedViewLayout],
    () => {
      updateTopBar()
    },
  )

  return {
    clearTopBarContent,
  }
}