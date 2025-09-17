import { ref, watch } from 'vue'

export type LayoutMode = 'comfy' | 'compact'

export const useLayoutPreference = (storageKey = 'songlist-layout', defaultLayout: LayoutMode = 'comfy') => {
  // Get initial value from localStorage or use default
  const getInitialLayout = (): LayoutMode => {
    try {
      const stored = localStorage.getItem(storageKey)
      if (stored === 'list') {
        localStorage.setItem(storageKey, 'comfy') // Migrate old value
        return 'comfy'
      }
      if (stored && ['comfy', 'compact'].includes(stored)) {
        return stored as LayoutMode
      }
    } catch (error) {
      // localStorage might not be available (SSR, etc.)
      console.warn('Failed to read layout preference from localStorage:', error)
    }
    return defaultLayout
  }

  const layout = ref<LayoutMode>(getInitialLayout())

  // Watch for changes and persist to localStorage
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
  // Get initial value from localStorage or use default
  const getInitialPageSize = (): number => {
    try {
      const stored = localStorage.getItem(storageKey)
      if (stored) {
        const parsed = parseInt(stored, 10)
        if (!isNaN(parsed) && parsed > 0 && parsed <= 100) {
          return parsed
        }
      }
    } catch (error) {
      console.warn('Failed to read page size preference from localStorage:', error)
    }
    return defaultPageSize
  }

  const pageSize = ref<number>(getInitialPageSize())

  // Watch for changes and persist to localStorage
  watch(pageSize, newPageSize => {
    try {
      localStorage.setItem(storageKey, newPageSize.toString())
    } catch (error) {
      console.warn('Failed to save page size preference to localStorage:', error)
    }
  })

  return {
    pageSize,
  }
}

export const useSortPreference = (storageKey = 'songlist-sort', defaultSort: string = 'title') => {
  const getInitialSort = (): string => {
    try {
      const stored = localStorage.getItem(storageKey)
      if (stored) {
        return stored
      }
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

  return {
    sort,
  }
}
