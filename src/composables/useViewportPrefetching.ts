import { ref, onMounted, onUnmounted, type Ref } from 'vue'

interface IntersectionObserverOptions {
  root?: Element | null
  rootMargin?: string
  threshold?: number
}

interface PrefetchConfig {
  enabled?: boolean
  preloadCount?: number
  observerMargin?: string
}

export const useViewportPrefetching = (
  items: Ref<Array<{ id: string; [key: string]: any }>>,
  config: PrefetchConfig = {}
) => {
  const {
    enabled = true,
    preloadCount = 5,
    observerMargin = '200px'
  } = config

  const visibleItems = ref<Set<string>>(new Set())
  const prefetchedItems = ref<Set<string>>(new Set())
  const observerRef = ref<IntersectionObserver | null>(null)

  const setupObserver = (container: Element | null) => {
    if (!enabled || !container) return

    observerRef.value = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
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
        root: container,
        rootMargin: observerMargin,
        threshold: 0.1
      }
    )
  }

  const prefetchNearbyItems = (currentItemId: string) => {
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
          detail: { itemId: item.id, item }
        }))
      }
    }
  }

  const observeItem = (element: HTMLElement, itemId: string) => {
    if (!observerRef.value) return

    element.dataset.itemId = itemId
    observerRef.value.observe(element)
  }

  const unobserveItem = (element: HTMLElement) => {
    if (!observerRef.value) return
    observerRef.value.unobserve(element)
  }

  const cleanup = () => {
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
    visibleItems,
    prefetchedItems,
    setupObserver,
    observeItem,
    unobserveItem,
    cleanup
  }
}