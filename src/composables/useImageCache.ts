import { ref, watch } from 'vue'

const cache = new Map<string, string>()
const queue = ref<string[]>([])
const isProcessing = ref(false)

const processQueue = async () => {
  if (queue.value.length === 0) {
    isProcessing.value = false
    return
  }

  isProcessing.value = true
  const url = queue.value.shift()

  if (url && !cache.has(url)) {
    try {
      const response = await fetch(url)
      if (!response.ok) {
        throw new Error(`Failed to fetch image: ${response.statusText}`)
      }
      const blob = await response.blob()
      const objectUrl = URL.createObjectURL(blob)
      cache.set(url, objectUrl)
    }
    catch (error) {
      console.error(`Error caching image ${url}:`, error)
      // Optionally, add the URL back to the queue to retry later
    }
  }

  // Process next item in the queue
  setTimeout(processQueue, 50) // Small delay to prevent network congestion
}

export const useImageCache = (urls: () => string[]) => {
  const imageUrls = ref(urls())
  const cachedUrls = ref<{ [key: string]: string }>({})

  watch(
    () => urls(),
    newUrls => {
      imageUrls.value = newUrls
      const newCachedUrls: { [key: string]: string } = {}

      for (const url of newUrls) {
        if (cache.has(url)) {
          newCachedUrls[url] = cache.get(url)!
        }
        else {
          if (!queue.value.includes(url)) {
            queue.value.push(url)
          }
        }
      }

      cachedUrls.value = newCachedUrls

      if (!isProcessing.value) {
        processQueue()
      }
    },
    { immediate: true, deep: true },
  )

  // This interval will periodically check for new items in the cache and update the component's reactive state.
  setInterval(() => {
    const newCachedUrls: { [key: string]: string } = { ...cachedUrls.value }
    let updated = false
    for (const url of imageUrls.value) {
      if (cache.has(url) && newCachedUrls[url] !== cache.get(url)) {
        newCachedUrls[url] = cache.get(url)!
        updated = true
      }
    }
    if (updated) {
      cachedUrls.value = newCachedUrls
    }
  }, 200) // check for new cached images every 200ms

  return {
    cachedUrls,
  }
}
