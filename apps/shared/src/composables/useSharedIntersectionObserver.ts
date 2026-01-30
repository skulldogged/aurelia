/**
 * Shared IntersectionObserver for efficient visibility detection across many elements.
 *
 * Instead of creating one IntersectionObserver per element (which can be thousands
 * for large grids), this module provides a single shared observer that manages
 * all visibility tracking.
 */

type IntersectionCallback = (isIntersecting: boolean) => void

// Single shared observer instance
let sharedObserver: IntersectionObserver | null = null
const observedElements = new Map<Element, IntersectionCallback>()

// Configuration matching the original ImageLoader settings
const OBSERVER_OPTIONS: IntersectionObserverInit = {
  root:       null,
  rootMargin: '200px',
  threshold:  0.05,
}

/**
 * Get or create the shared IntersectionObserver
 */
const getObserver = (): IntersectionObserver => {
  if (!sharedObserver) {
    sharedObserver = new IntersectionObserver(entries => {
      for (const entry of entries) {
        const callback = observedElements.get(entry.target)
        if (callback) {
          callback(entry.isIntersecting)
        }
      }
    }, OBSERVER_OPTIONS)
  }
  return sharedObserver
}

/**
 * Start observing an element for visibility changes
 *
 * @param element - The DOM element to observe
 * @param callback - Function called when visibility changes
 * @returns Cleanup function to stop observing
 */
export const observeElement = (
  element: Element,
  callback: IntersectionCallback,
): (() => void) => {
  const observer = getObserver()

  // Store the callback for this element
  observedElements.set(element, callback)

  // Start observing
  observer.observe(element)

  // Return cleanup function
  return () => {
    observer.unobserve(element)
    observedElements.delete(element)

    // If no elements are being observed, clean up the observer
    if (observedElements.size === 0 && sharedObserver) {
      sharedObserver.disconnect()
      sharedObserver = null
    }
  }
}

/**
 * Vue composable for using the shared IntersectionObserver
 */
export const useSharedIntersectionObserver = (): {
  observeElement: (element: Element, callback: IntersectionCallback) => (() => void)
} => ({
  observeElement,
})
