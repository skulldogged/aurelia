import { computed, type ComputedRef, ref, type Ref, watch } from 'vue'

export type LayoutMode = 'comfy' | 'compact'

const getInitialLayout = (storageKey: string, defaultLayout: LayoutMode): LayoutMode => {
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

export interface LayoutPreference {
  layout: Ref<LayoutMode>
}

export const useLayoutPreference = (
  storageKey = 'songlist-layout',
  defaultLayout: LayoutMode = 'comfy',
): LayoutPreference => {
  const layout = ref<LayoutMode>(getInitialLayout(storageKey, defaultLayout))

  watch(layout, newLayout => {
    try {
      localStorage.setItem(storageKey, newLayout)
    } catch (error) {
      console.warn('Failed to save layout preference to localStorage:', error)
    }
  })

  return { layout }
}

const getInitialPageSize = (storageKey: string, defaultPageSize: number): number => {
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

export interface PageSizePreference {
  pageSize: Ref<number>
}

export const usePageSizePreference = (
  storageKey = 'songlist-pagesize',
  defaultPageSize: number = 20,
): PageSizePreference => {
  const pageSize = ref<number>(getInitialPageSize(storageKey, defaultPageSize))

  watch(pageSize, newPageSize => {
    try {
      localStorage.setItem(storageKey, newPageSize.toString())
    } catch (error) {
      console.warn('Failed to save page size preference to localStorage:', error)
    }
  })

  return { pageSize }
}

const getInitialSort = (storageKey: string, defaultSort: string): string => {
  try {
    const stored = localStorage.getItem(storageKey)
    if (stored)
      return stored
  } catch (error) {
    console.warn('Failed to read sort preference from localStorage:', error)
  }
  return defaultSort
}

export interface SortPreference {
  sort: Ref<string>
}

export const useSortPreference = (
  storageKey = 'songlist-sort',
  defaultSort: string = 'title',
): SortPreference => {
  const sort = ref<string>(getInitialSort(storageKey, defaultSort))

  watch(sort, newSort => {
    try {
      localStorage.setItem(storageKey, newSort)
    } catch (error) {
      console.warn('Failed to save sort preference to localStorage:', error)
    }
  })

  return { sort }
}

export interface Pagination<T> {
  canNextPage:      ComputedRef<boolean>
  canPreviousPage:  ComputedRef<boolean>
  goToFirstPage:    () => void
  goToLastPage:     () => void
  goToNextPage:     () => void
  goToPreviousPage: () => void
  pageCount:        ComputedRef<number>
  pagedItems:       ComputedRef<T[]>
  pageIndex:        Ref<number>
  pageSize:         Ref<number>
  pageSizeOptions:  number[]
  setPageSize:      (value: number) => void
  total:            ComputedRef<number>
}

export const usePagination = <T>(
  items: ComputedRef<T[]> | Ref<T[]>,
  pageSizeKey = 'page-size',
  defaultPageSize = 20,
): Pagination<T> => {
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

  const goToPreviousPage = (): void => {
    if (canPreviousPage.value) pageIndex.value -= 1
  }

  const goToNextPage = (): void => {
    if (canNextPage.value) pageIndex.value += 1
  }

  const goToFirstPage = (): void => {
    pageIndex.value = 0
  }

  const goToLastPage = (): void => {
    pageIndex.value = pageCount.value - 1
  }

  const setPageSize = (value: number): void => {
    const oldStart = pageIndex.value * pageSize.value
    pageSize.value = value
    pageIndex.value = Math.floor(oldStart / pageSize.value)
  }

  const pageSizeOptions = [10, 20, 30, 50]

  return {
    canNextPage,
    canPreviousPage,
    goToFirstPage,
    goToLastPage,
    goToNextPage,
    goToPreviousPage,
    pageCount,
    pagedItems,
    pageIndex,
    pageSize,
    pageSizeOptions,
    setPageSize,
    total,
  }
}