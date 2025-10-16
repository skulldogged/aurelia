import { computed, type ComputedRef, ref, type Ref, watch } from 'vue'

import { logger } from '@/lib/logger'

export type LayoutMode = 'comfy' | 'compact'

const safeGetLocalStorage = <T>(key: string, fallback: T, validator?: (value: unknown) => boolean): T => {
  try {
    const stored = localStorage.getItem(key)
    if (!stored) return fallback

    const parsed: unknown = JSON.parse(stored)
    if (validator && !validator(parsed)) return fallback

    return parsed as T
  } catch (error) {
    logger.warn(`Failed to read ${key} from localStorage:`, error)
    return fallback
  }
}

const safeSetLocalStorage = (key: string, value: unknown): void => {
  try {
    localStorage.setItem(key, JSON.stringify(value))
  } catch (error) {
    logger.warn(`Failed to save ${key} to localStorage:`, error)
  }
}

const getInitialLayout = (storageKey: string, defaultLayout: LayoutMode): LayoutMode =>
  safeGetLocalStorage(
    storageKey,
    defaultLayout,
    (value): value is LayoutMode => typeof value === 'string' && ['comfy', 'compact'].includes(value),
  )

export interface LayoutPreference {
  layout: Ref<LayoutMode>
}

export const useLayoutPreference = (
  storageKey = 'songlist-layout',
  defaultLayout: LayoutMode = 'comfy',
): LayoutPreference => {
  const layout = ref<LayoutMode>(getInitialLayout(storageKey, defaultLayout))

  watch(layout, newLayout => safeSetLocalStorage(storageKey, newLayout))

  return { layout }
}

const getInitialPageSize = (storageKey: string, defaultPageSize: number): number =>
  safeGetLocalStorage(
    storageKey,
    defaultPageSize,
    (value): value is number => {
      const parsed = typeof value === 'string' ? parseInt(value, 10) : value
      return typeof parsed === 'number' && !isNaN(parsed) && parsed > 0 && parsed <= 100
    },
  )

export interface PageSizePreference {
  pageSize: Ref<number>
}

export const usePageSizePreference = (
  storageKey = 'songlist-pagesize',
  defaultPageSize: number = 20,
): PageSizePreference => {
  const pageSize = ref<number>(getInitialPageSize(storageKey, defaultPageSize))

  watch(pageSize, newPageSize => safeSetLocalStorage(storageKey, newPageSize))

  return { pageSize }
}

const getInitialSort = (storageKey: string, defaultSort: string): string =>
  safeGetLocalStorage(storageKey, defaultSort)

export interface SortPreference {
  sort: Ref<string>
}

export const useSortPreference = (
  storageKey = 'songlist-sort',
  defaultSort: string = 'title',
): SortPreference => {
  const sort = ref<string>(getInitialSort(storageKey, defaultSort))

  watch(sort, newSort => safeSetLocalStorage(storageKey, newSort))

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
  pageSizeOptions:  ComputedRef<number[]>
  setPageSize:      (value: number) => void
  total:            ComputedRef<number>
}

export const usePagination = <T>(
  items: ComputedRef<T[]> | Ref<T[]>,
  pageSizeKey: Ref<string> | string = 'page-size',
  defaultPageSize: number | Ref<number> = 20,
  pageSizeOptions: number[] | Ref<number[]> = [10, 20, 30, 50],
): Pagination<T> => {
  const pageIndex = ref(0)

  // Convert to refs if they're not already
  const pageSizeKeyRef = typeof pageSizeKey === 'string' ? ref(pageSizeKey) : pageSizeKey
  const defaultPageSizeRef = typeof defaultPageSize === 'number' ? ref(defaultPageSize) : defaultPageSize
  const pageSizeOptionsRef = Array.isArray(pageSizeOptions) ? ref(pageSizeOptions) : pageSizeOptions

  // Watch for changes in pageSizeKey and reload pageSize from localStorage
  const pageSize = ref(getInitialPageSize(pageSizeKeyRef.value, defaultPageSizeRef.value))

  watch(pageSizeKeyRef, newKey => {
    pageSize.value = getInitialPageSize(newKey, defaultPageSizeRef.value)
  })

  // Watch for changes in defaultPageSize and update pageSize (but don't save to localStorage)
  watch(defaultPageSizeRef, newDefault => {
    pageSize.value = newDefault
  })

  // Save pageSize to localStorage when it changes
  watch(pageSize, newPageSize => {
    safeSetLocalStorage(pageSizeKeyRef.value, newPageSize)
  })

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
    pageSizeOptions: computed(() =>
      Array.isArray(pageSizeOptions) ? pageSizeOptions : pageSizeOptionsRef.value,
    ),
    setPageSize,
    total,
  }
}