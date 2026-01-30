import { onUnmounted, ref, type Ref } from 'vue'

interface PrefetchConfig {
  enabled?:        boolean
  observerMargin?: string
  preloadCount?:   number
}

export const useViewportPrefetching = (
  items: Ref<Array<{ [key: string]: unknown; id: string }>>,
  config: PrefetchConfig = {},
): {
  cleanup:         () => void
  observeItem:     (element: HTMLElement, itemId: string) => void
  prefetchedItems: Ref<Set<string>>
  setupObserver:   (container: Element | null) => void
  unobserveItem:   (element: HTMLElement) => void
  visibleItems:    Ref<Set<string>>
} => {
  const {
    enabled = true,
    observerMargin = '200px',
    preloadCount = 2,
  } = config

  const prefetchedItems = ref<Set<string>>(new Set())
  const visibleItems = ref<Set<string>>(new Set())
  const observerRef = ref<IntersectionObserver | null>(null)

  const setupObserver = (container: Element | null): void => {
    if (!enabled || !container) return

    observerRef.value = new IntersectionObserver(
      entries => {
        entries.forEach(entry => {
          const element = entry.target as HTMLElement
          const itemId = element.dataset.itemId

          if (!itemId) return

          if (entry.isIntersecting) {
            visibleItems.value.add(itemId)
            // Trigger prefetch for nearby items
            prefetchNearbyItems(itemId)
          } else {
            visibleItems.value.delete(itemId)
          }
        })
      },
      {
        root:       container,
        rootMargin: observerMargin,
        threshold:  0.1,
      },
    )
  }

  const prefetchNearbyItems = (currentItemId: string): void => {
    const itemIndex = items.value.findIndex(item => item.id === currentItemId)
    if (itemIndex === -1) return

    // Preload items before and after the current item
    const start = Math.max(0, itemIndex - preloadCount)
    const end = Math.min(items.value.length, itemIndex + preloadCount + 1)

    for (let i = start; i < end; i++) {
      const item = items.value[i]
      if (item && !prefetchedItems.value.has(item.id)) {
        prefetchedItems.value.add(item.id)
        // Emit custom event for prefetching
        window.dispatchEvent(new CustomEvent('prefetch-item', {
          detail: { item, itemId: item.id },
        }))
      }
    }
  }

  const observeItem = (element: HTMLElement, itemId: string): void => {
    if (!observerRef.value) return

    element.dataset.itemId = itemId
    observerRef.value.observe(element)
  }

  const unobserveItem = (element: HTMLElement): void => {
    if (!observerRef.value) return
    observerRef.value.unobserve(element)
  }

  const cleanup = (): void => {
    if (observerRef.value) {
      observerRef.value.disconnect()
      observerRef.value = null
    }
    visibleItems.value.clear()
    prefetchedItems.value.clear()
  }

  onUnmounted(() => {
    cleanup()
  })

  return {
    cleanup,
    observeItem,
    prefetchedItems,
    setupObserver,
    unobserveItem,
    visibleItems,
  }
}
