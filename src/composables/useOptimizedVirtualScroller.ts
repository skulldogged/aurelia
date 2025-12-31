import type { ComputedRef, Ref } from 'vue'

import { useVirtualizer } from '@tanstack/vue-virtual'
import { computed } from 'vue'

interface VirtualItem {
  index: number
  key:   bigint | number | string
  start: number
}

interface VirtualScrollerOptions {
  count:         Ref<number>
  estimateSize:  (() => number) | Ref<number>
  scrollElement: Ref<HTMLElement | null>
  viewLayout:    Ref<'comfy' | 'compact'>
}

export const useOptimizedVirtualScroller = ({
  count,
  estimateSize,
  scrollElement,
  viewLayout,
}: VirtualScrollerOptions): {
  getOptimalOverscan: () => number
  isScrolling:        ComputedRef<boolean>
  remeasure:          () => void
  rowVirtualizer: Ref<{
    getTotalSize:    () => number
    getVirtualItems: () => VirtualItem[]
    isScrolling:     boolean
    measure:         () => void
    measureElement:  (node: Element | null) => void
    range:           null | { endIndex: number; startIndex: number }
    scrollToIndex:   (index: number) => void
  }>
  scrollToIndex: (index: number) => void
  virtualItems: ComputedRef<{
    index:      number
    isVisible:  boolean
    virtualRow: VirtualItem
  }[]>
  virtualRows: ComputedRef<VirtualItem[]>
} => {
  // Dynamic overscan based on device performance and layout
  const getOptimalOverscan = (): number => {
    const isCompact = viewLayout.value === 'compact'
    const isLowEndDevice = navigator.hardwareConcurrency <= 4

    if (isLowEndDevice) {
      return isCompact ? 3 : 2
    }

    // For high-end devices, we can afford more overscan for smoother scrolling
    return isCompact ? 5 : 4
  }

  // Create the virtualizer with dynamic parameters
  const rowVirtualizer = useVirtualizer({
    count:            count.value,
    enabled:          true,
    estimateSize:     typeof estimateSize === 'function' ? estimateSize : () => estimateSize.value,
    getScrollElement: () => scrollElement.value,
    overscan:         getOptimalOverscan(),
  })

  // Computed property for virtual items
  const virtualRows = computed(() => rowVirtualizer.value.getVirtualItems())

  // Computed property for virtual songs with metadata
  const virtualItems = computed(() => {
    const range = rowVirtualizer.value.range
    const items = virtualRows.value.map(row => ({
      index:      row.index,
      // Add position metadata for optimization
      isVisible:  range ? row.index >= range.startIndex && row.index <= range.endIndex : true,
      virtualRow: row,
    }))
    return items
  })

  // Function to remeasure when needed
  const remeasure = (): void => {
    rowVirtualizer.value.measure()
  }

  // Function to scroll to specific item
  const scrollToIndex = (index: number): void => {
    rowVirtualizer.value.scrollToIndex(index)
  }

  const isScrolling = computed(() => rowVirtualizer.value.isScrolling)

  return {
    getOptimalOverscan,
    isScrolling,
    remeasure,
    rowVirtualizer,
    scrollToIndex,
    virtualItems,
    virtualRows,
  }
}