import { ref, watch, onUnmounted } from 'vue'

const cache = new Map<string, string>()
const activeFetches = new Map<string, Promise<void>>()

// Helper to fetch and cache a single image
const fetchAndCacheImage = async (url: string) => {
  if (!url || cache.has(url)) {
    return
  }

  // Avoid fetching the same URL multiple times concurrently
  if (activeFetches.has(url)) {
    return activeFetches.get(url)
  }

  const fetchPromise = (async () => {
    try {
      const response = await fetch(url)
      if (!response.ok) {
        throw new Error(`HTTP error! Status: ${response.status}`)
      }
      const blob = await response.blob()
      const objectUrl = URL.createObjectURL(blob)
      cache.set(url, objectUrl)
    } catch (error) {
      console.error(`Failed to cache image: ${url}`, error)
      // Optional: Handle error, maybe set a placeholder image URL in the cache
    } finally {
      // Clean up the active fetch record once done
      activeFetches.delete(url)
    }
  })()

  activeFetches.set(url, fetchPromise)
  return fetchPromise
}

export const useImageCache = (getUrls: () => string[]) => {
  const imageUrls = ref<string[]>(getUrls())
  const cachedUrls = ref<{ [key: string]: string }>({})

  const blobUrlsToRevoke = new Set<string>()

  const updateCache = async () => {
    const urls = imageUrls.value
    const newCachedUrls: { [key: string]: string } = {}
    const promises: Promise<void>[] = []

    for (const url of urls) {
      if (cache.has(url)) {
        newCachedUrls[url] = cache.get(url)!
      } else {
        // Start fetching images that are not in the cache
        promises.push(fetchAndCacheImage(url).then(() => {
            // Once an image is fetched, update the reactive object
            if (cache.has(url)) {
              cachedUrls.value = { ...cachedUrls.value, [url]: cache.get(url)! }
              blobUrlsToRevoke.add(cache.get(url)!)
            }
        }))
      }
    }

    cachedUrls.value = newCachedUrls
    // We don't need to wait for all images to be fetched here,
    // the UI will update reactively as each one completes.
  }

  watch(
    () => getUrls(),
    (newUrls) => {
      // Simple deep comparison for arrays of strings
      if (JSON.stringify(imageUrls.value) !== JSON.stringify(newUrls)) {
        imageUrls.value = newUrls
        updateCache()
      }
    },
    { immediate: true },
  )

  // Clean up Blob URLs to prevent memory leaks
  onUnmounted(() => {
    blobUrlsToRevoke.forEach(url => URL.revokeObjectURL(url))
  })

  return {
    cachedUrls,
  }
}
