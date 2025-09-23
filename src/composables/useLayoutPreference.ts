import { ref, watch, computed, type Ref, type ComputedRef } from 'vue'

export type LayoutMode = 'comfy' | 'compact'

export const useLayoutPreference = (storageKey = 'songlist-layout', defaultLayout: LayoutMode = 'comfy') => {
  const getInitialLayout = (): LayoutMode => {
    try {
      const stored = localStorage.getItem(storageKey)

      if (stored && ['comfy', 'compact'].includes(stored))
        return stored as LayoutMode
    } catch (error) {
      // localStorage might not be available (SSR, etc.)
      console.warn('Failed to read layout preference from localStorage:', error)
    }
    return defaultLayout
  }

  const layout = ref<LayoutMode>(getInitialLayout())

  watch(layout, newLayout => {
    try {
      localStorage.setItem(storageKey, newLayout)
    } catch (error) {
      console.warn('Failed to save layout preference to localStorage:', error)
    }
  })

  return {
    layout,
  }
}

export const usePageSizePreference = (storageKey = 'songlist-pagesize', defaultPageSize: number = 20) => {
  const getInitialPageSize = (): number => {
    try {
      const stored = localStorage.getItem(storageKey)
      if (stored) {
        const parsed = parseInt(stored, 10)
        if (!isNaN(parsed) && parsed > 0 && parsed <= 100)
          return parsed
      }
    } catch (error) {
      console.warn('Failed to read page size preference from localStorage:', error)
    }
    return defaultPageSize
  }

  const pageSize = ref<number>(getInitialPageSize())

  watch(pageSize, newPageSize => {
    try {
      localStorage.setItem(storageKey, newPageSize.toString())
    } catch (error) {
      console.warn('Failed to save page size preference to localStorage:', error)
    }
  })

  return { pageSize }
}

export const useSortPreference = (storageKey = 'songlist-sort', defaultSort: string = 'title') => {
  const getInitialSort = (): string => {
    try {
      const stored = localStorage.getItem(storageKey)
      if (stored)
        return stored
    } catch (error) {
      console.warn('Failed to read sort preference from localStorage:', error)
    }
    return defaultSort
  }

  const sort = ref<string>(getInitialSort())

  watch(sort, newSort => {
    try {
      localStorage.setItem(storageKey, newSort)
    } catch (error) {
      console.warn('Failed to save sort preference to localStorage:', error)
    }
  })

  return { sort }
}

export const usePagination = <T>(
  items: Ref<T[]> | ComputedRef<T[]>,
  pageSizeKey = 'page-size',
  defaultPageSize = 20,
) => {
  const pageIndex = ref(0)
  const { pageSize } = usePageSizePreference(pageSizeKey, defaultPageSize)

  // Reset to first page when page size changes
  watch(pageSize, () => {
    pageIndex.value = 0
  })

  // Reset to first page when items change
  watch(() => items.value.length, () => {
    pageIndex.value = 0
  })

  const total = computed(() => items.value.length)
  const pageCount = computed(() => Math.max(1, Math.ceil(total.value / pageSize.value)))
  const canPreviousPage = computed(() => pageIndex.value > 0)
  const canNextPage = computed(() => pageIndex.value < pageCount.value - 1)

  const pagedItems = computed(() => {
    const start = pageIndex.value * pageSize.value
    const end = start + pageSize.value
    return items.value.slice(start, end)
  })

  const goToPreviousPage = () => {
    if (canPreviousPage.value) pageIndex.value -= 1
  }

  const goToNextPage = () => {
    if (canNextPage.value) pageIndex.value += 1
  }

  const goToFirstPage = () => {
    pageIndex.value = 0
  }

  const goToLastPage = () => {
    pageIndex.value = pageCount.value - 1
  }

  const setPageSize = (value: number) => {
    const oldStart = pageIndex.value * pageSize.value
    pageSize.value = value
    pageIndex.value = Math.floor(oldStart / pageSize.value)
  }

  const pageSizeOptions = [10, 20, 30, 50]

  return {
    pageIndex,
    pageSize,
    total,
    pageCount,
    canPreviousPage,
    canNextPage,
    pagedItems,
    goToPreviousPage,
    goToNextPage,
    goToFirstPage,
    goToLastPage,
    setPageSize,
    pageSizeOptions,
  }
}