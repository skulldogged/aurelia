import { invoke } from '@tauri-apps/api/core'

// Types for cache data
interface CacheMetadata {
  timestamp: number
  size:      number
}

interface FailureData {
  timestamp: number
  error:     string
}

// Frontend caches
const dataUrlCache = new Map<string, string>()
const loadingPromises = new Map<string, Promise<string | null>>()
const cacheMetadata = new Map<string, CacheMetadata>()
const failureCache = new Map<string, FailureData>()

// Constants for cache management
const MAX_MEMORY_CACHE_SIZE = 50 // Maximum number of images in memory
const CACHE_EXPIRY_HOURS = 24 // Cache expiry time
const FAILURE_CACHE_EXPIRY_HOURS = 1 // Cache 404s for shorter time
const PERSISTENT_CACHE_KEY = 'image_cache_metadata'
const FAILURE_CACHE_KEY = 'image_failure_cache'

// Cache management functions
const loadPersistentCache = () => {
  try {
    const stored = localStorage.getItem(PERSISTENT_CACHE_KEY)
    if (stored) {
      const metadata = JSON.parse(stored)
      // Only load metadata for non-expired entries
      const now = Date.now()
      const expiryTime = CACHE_EXPIRY_HOURS * 60 * 60 * 1000 // Convert hours to ms

      Object.entries(metadata).forEach(([key, data]: [string, unknown]) => {
        const cacheData = data as CacheMetadata
        if (now - cacheData.timestamp < expiryTime)
          cacheMetadata.set(key, cacheData)
      })
    }
  } catch (error) {
    console.warn('Failed to load persistent cache:', error)
  }
}

const loadFailureCache = () => {
  try {
    const stored = localStorage.getItem(FAILURE_CACHE_KEY)
    if (stored) {
      const failures = JSON.parse(stored)
      // Only load failures for non-expired entries
      const now = Date.now()
      const expiryTime = FAILURE_CACHE_EXPIRY_HOURS * 60 * 60 * 1000 // Convert hours to ms

      Object.entries(failures).forEach(([key, data]: [string, unknown]) => {
        const failureData = data as FailureData
        if (now - failureData.timestamp < expiryTime)
          failureCache.set(key, failureData)
      })
    }
  } catch (error) {
    console.warn('Failed to load failure cache:', error)
  }
}

const savePersistentCache = () => {
  try {
    const metadata = Object.fromEntries(cacheMetadata)
    localStorage.setItem(PERSISTENT_CACHE_KEY, JSON.stringify(metadata))
  } catch (error) {
    console.warn('Failed to save persistent cache:', error)
  }
}

const saveFailureCache = () => {
  try {
    const failures = Object.fromEntries(failureCache)
    localStorage.setItem(FAILURE_CACHE_KEY, JSON.stringify(failures))
  } catch (error) {
    console.warn('Failed to save failure cache:', error)
  }
}

const evictOldCacheEntries = () => {
  if (dataUrlCache.size <= MAX_MEMORY_CACHE_SIZE) return

  // Sort by timestamp (oldest first) and remove oldest entries
  const entries = Array.from(cacheMetadata.entries())
    .sort(([, a], [, b]) => a.timestamp - b.timestamp)

  const toRemove = entries.slice(0, dataUrlCache.size - MAX_MEMORY_CACHE_SIZE + 1)

  toRemove.forEach(([key]) => {
    dataUrlCache.delete(key)
    cacheMetadata.delete(key)
  })

  savePersistentCache()
}

const updateCacheMetadata = (cacheKey: string, size: number) => {
  cacheMetadata.set(cacheKey, {
    timestamp: Date.now(),
    size,
  })
  savePersistentCache()
}

const cacheFailure = (cacheKey: string, error: string) => {
  failureCache.set(cacheKey, {
    timestamp: Date.now(),
    error,
  })
  saveFailureCache()
}

// Initialize caches on module load
loadPersistentCache()
loadFailureCache()

export const useImageLoader = () => {
  // Generate cache key for frontend memory cache
  const generateCacheKey = (itemId: string, imageType: string = 'Primary') =>
    `${itemId}_${imageType}`

  // Get cached image path or URL
  const getImageUrl = async (
    itemId: string,
    serverUrl: string,
    token: string,
    imageType: string = 'Primary',
  ): Promise<string | null> => {
    if (!itemId || !serverUrl || !token) return null

    const cacheKey = generateCacheKey(itemId, imageType)

    // Check if this image is known to fail (404, etc.)
    if (failureCache.has(cacheKey)) {
      const failureData = failureCache.get(cacheKey)!
      const now = Date.now()
      const expiryTime = FAILURE_CACHE_EXPIRY_HOURS * 60 * 60 * 1000
      if (now - failureData.timestamp < expiryTime) {
        // Still in failure cache, don't attempt to fetch
        console.debug(`Image ${cacheKey} is cached as failed: ${failureData.error}`)
        return null
      } else {
        // Failure cache expired, remove it and try again
        failureCache.delete(cacheKey)
      }
    }

    // Check if we have a valid cached version in memory
    if (dataUrlCache.has(cacheKey)) {
      const metadata = cacheMetadata.get(cacheKey)
      if (metadata) {
        const now = Date.now()
        const expiryTime = CACHE_EXPIRY_HOURS * 60 * 60 * 1000
        if (now - metadata.timestamp < expiryTime) {
          // Update access timestamp
          updateCacheMetadata(cacheKey, metadata.size)
          return dataUrlCache.get(cacheKey)!
        } else {
          // Cache expired, remove it
          dataUrlCache.delete(cacheKey)
          cacheMetadata.delete(cacheKey)
        }
      }
    }

    // Check if we're already loading this image
    if (loadingPromises.has(cacheKey))
      return loadingPromises.get(cacheKey)!

    // Create loading promise
    const loadingPromise = (async () => {
      try {
        // First check if image is already cached in backend
        const cachedDataUrl = await invoke<string | null>('get_cached_image_data_url', {
          itemId,
          imageType,
        })

        if (cachedDataUrl) {
          // Cache in memory and return
          dataUrlCache.set(cacheKey, cachedDataUrl)
          updateCacheMetadata(cacheKey, cachedDataUrl.length)
          evictOldCacheEntries()
          return cachedDataUrl
        }

        // Image not cached, cache it now
        const imageUrl = `${serverUrl.replace(/\/$/, '')}/Items/${itemId}/Images/${imageType}`
        const newCachedDataUrl = await invoke<string>('cache_image_from_url', {
          itemId,
          imageType,
          imageUrl,
          serverUrl,
          token,
        })

        // Cache in memory and return
        dataUrlCache.set(cacheKey, newCachedDataUrl)
        updateCacheMetadata(cacheKey, newCachedDataUrl.length)
        evictOldCacheEntries()
        return newCachedDataUrl
      } catch (error) {
        console.warn('Failed to load/cached image:', error)
        // Cache the failure to avoid repeated attempts
        cacheFailure(cacheKey, error instanceof Error ? error.message : String(error))

        // Return null to indicate no image available
        return null
      } finally {
        // Clean up loading promise
        loadingPromises.delete(cacheKey)
      }
    })()

    // Store the loading promise
    loadingPromises.set(cacheKey, loadingPromise)

    return loadingPromise
  }

  // Clear the image cache
  const clearImageCache = async (): Promise<void> => {
    try {
      await invoke('clear_image_cache')
      // Also clear frontend caches
      dataUrlCache.clear()
      cacheMetadata.clear()
      failureCache.clear()
      loadingPromises.clear()
      // Clear persistent storage
      localStorage.removeItem(PERSISTENT_CACHE_KEY)
      localStorage.removeItem(FAILURE_CACHE_KEY)
    } catch (error) {
      console.error('Failed to clear image cache:', error)
      throw error
    }
  }

  // Get cache statistics
  const getImageCacheStats = async (): Promise<{
    total_size:                 number
    file_count:                 number
    cache_dir:                  string
    memory_cache_size:          number
    persistent_cache_size:      number
    failure_cache_size:         number
    cache_expiry_hours:         number
    failure_cache_expiry_hours: number
  }> => {
    try {
      const stats = await invoke<string>('get_image_cache_stats')
      const backendStats = JSON.parse(stats)

      // Calculate persistent cache size (valid entries only)
      const now = Date.now()
      const expiryTime = CACHE_EXPIRY_HOURS * 60 * 60 * 1000
      const failureExpiryTime = FAILURE_CACHE_EXPIRY_HOURS * 60 * 60 * 1000

      let persistentCacheSize = 0
      let failureCacheSize = 0

      cacheMetadata.forEach(metadata => {
        if (now - metadata.timestamp < expiryTime)
          persistentCacheSize++
      })

      failureCache.forEach(failure => {
        if (now - failure.timestamp < failureExpiryTime)
          failureCacheSize++
      })

      return {
        ...backendStats,
        memory_cache_size:          dataUrlCache.size,
        persistent_cache_size:      persistentCacheSize,
        failure_cache_size:         failureCacheSize,
        cache_expiry_hours:         CACHE_EXPIRY_HOURS,
        failure_cache_expiry_hours: FAILURE_CACHE_EXPIRY_HOURS,
      }
    } catch (error) {
      console.error('Failed to get image cache stats:', error)
      throw error
    }
  }

  // Preload recently cached images into memory
  const preloadRecentImages = async (serverUrl: string, token: string, limit: number = 10): Promise<void> => {
    try {
      // Get the most recently accessed images from metadata
      const recentEntries = Array.from(cacheMetadata.entries())
        .sort(([, a], [, b]) => b.timestamp - a.timestamp) // Most recent first
        .slice(0, limit)

      if (recentEntries.length === 0) return

      // Filter out images that are in the failure cache
      const validEntries = recentEntries.filter(([cacheKey]) => !failureCache.has(cacheKey))

      if (validEntries.length === 0) return

      // Preload each image in the background
      const preloadPromises = validEntries.map(async ([cacheKey]) => {
        const [itemId, imageType] = cacheKey.split('_')

        try {
          // This will load from backend cache if available and update our memory cache
          await getImageUrl(itemId, serverUrl, token, imageType)
        } catch (error) {
          console.warn(`Failed to preload image ${cacheKey}:`, error)
        }
      })

      // Wait for all preloads to complete (but don't block the UI)
      await Promise.allSettled(preloadPromises)
    } catch (error) {
      console.warn('Failed to preload recent images:', error)
    }
  }

  return {
    getImageUrl,
    clearImageCache,
    getImageCacheStats,
    preloadRecentImages,
  }
}
