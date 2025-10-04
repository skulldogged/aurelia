import { invoke } from '@tauri-apps/api/core'

interface CacheMetadata {
  size:      number
  timestamp: number
}

interface FailureData {
  error:     string
  timestamp: number
}

const dataUrlCache = new Map<string, string>()
const loadingPromises = new Map<string, Promise<null | string>>()
const cacheMetadata = new Map<string, CacheMetadata>()
const failureCache = new Map<string, FailureData>()

const MAX_MEMORY_CACHE_SIZE = 50
const CACHE_EXPIRY_HOURS = 24
const FAILURE_CACHE_EXPIRY_HOURS = 1
const PERSISTENT_CACHE_KEY = 'image_cache_metadata'
const FAILURE_CACHE_KEY = 'image_failure_cache'

const loadPersistentCache = (): void => {
  try {
    const stored = localStorage.getItem(PERSISTENT_CACHE_KEY)
    if (stored) {
      const metadata = JSON.parse(stored)
      const now = Date.now()
      const expiryTime = CACHE_EXPIRY_HOURS * 60 * 60 * 1000

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

const loadFailureCache = (): void => {
  try {
    const stored = localStorage.getItem(FAILURE_CACHE_KEY)
    if (stored) {
      const failures = JSON.parse(stored)
      const now = Date.now()
      const expiryTime = FAILURE_CACHE_EXPIRY_HOURS * 60 * 60 * 1000

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

const savePersistentCache = (): void => {
  try {
    const metadata = Object.fromEntries(cacheMetadata)
    localStorage.setItem(PERSISTENT_CACHE_KEY, JSON.stringify(metadata))
  } catch (error) {
    console.warn('Failed to save persistent cache:', error)
  }
}

const saveFailureCache = (): void => {
  try {
    const failures = Object.fromEntries(failureCache)
    localStorage.setItem(FAILURE_CACHE_KEY, JSON.stringify(failures))
  } catch (error) {
    console.warn('Failed to save failure cache:', error)
  }
}

const evictOldCacheEntries = (): void => {
  if (dataUrlCache.size <= MAX_MEMORY_CACHE_SIZE) return

  const entries = Array.from(cacheMetadata.entries())
    .sort(([, a], [, b]) => a.timestamp - b.timestamp)

  const toRemove = entries.slice(0, dataUrlCache.size - MAX_MEMORY_CACHE_SIZE + 1)

  toRemove.forEach(([key]) => {
    dataUrlCache.delete(key)
    cacheMetadata.delete(key)
  })

  savePersistentCache()
}

const updateCacheMetadata = (cacheKey: string, size: number): void => {
  cacheMetadata.set(cacheKey, {
    size,
    timestamp: Date.now(),
  })
  savePersistentCache()
}

const cacheFailure = (cacheKey: string, error: string): void => {
  failureCache.set(cacheKey, {
    error,
    timestamp: Date.now(),
  })
  saveFailureCache()
}

loadPersistentCache()
loadFailureCache()

const generateCacheKey = (itemId: string, imageType: string = 'Primary'): string =>
  `${itemId}_${imageType}`

const clearImageFromCache = (itemId: string, imageType: string = 'Primary'): void => {
  const cacheKey = generateCacheKey(itemId, imageType)
  dataUrlCache.delete(cacheKey)
  cacheMetadata.delete(cacheKey)
  failureCache.delete(cacheKey)
  savePersistentCache()
  saveFailureCache()
}

const getImageUrl = async (
  itemId: string,
  serverUrl: string,
  token: string,
  imageType: string = 'Primary',
): Promise<null | string> => {
  if (!itemId || !serverUrl || !token) return null

  const cacheKey = generateCacheKey(itemId, imageType)

  if (failureCache.has(cacheKey)) {
    const failureData = failureCache.get(cacheKey)!
    const now = Date.now()
    const expiryTime = FAILURE_CACHE_EXPIRY_HOURS * 60 * 60 * 1000
    if (now - failureData.timestamp < expiryTime) {
      console.debug(`Image ${cacheKey} is cached as failed: ${failureData.error}`)
      return null
    } else {
      failureCache.delete(cacheKey)
    }
  }

  if (dataUrlCache.has(cacheKey)) {
    const metadata = cacheMetadata.get(cacheKey)
    if (metadata) {
      const now = Date.now()
      const expiryTime = CACHE_EXPIRY_HOURS * 60 * 60 * 1000
      if (now - metadata.timestamp < expiryTime) {
        updateCacheMetadata(cacheKey, metadata.size)
        return dataUrlCache.get(cacheKey)!
      } else {
        dataUrlCache.delete(cacheKey)
        cacheMetadata.delete(cacheKey)
      }
    }
  }

  if (loadingPromises.has(cacheKey))
    return loadingPromises.get(cacheKey)!

  const loadingPromise = (async () => {
    try {
      const cachedDataUrl = await invoke<null | string>('get_cached_image_data_url', {
        imageType,
        itemId,
      })

      if (cachedDataUrl) {
        dataUrlCache.set(cacheKey, cachedDataUrl)
        updateCacheMetadata(cacheKey, cachedDataUrl.length)
        evictOldCacheEntries()
        return cachedDataUrl
      }

      const imageUrl = `${serverUrl.replace(/\/$/, '')}/Items/${itemId}/Images/${imageType}`
      const newCachedDataUrl = await invoke<string>('cache_image_from_url', {
        imageType,
        imageUrl,
        itemId,
        serverUrl,
        token,
      })

      dataUrlCache.set(cacheKey, newCachedDataUrl)
      updateCacheMetadata(cacheKey, newCachedDataUrl.length)
      evictOldCacheEntries()
      return newCachedDataUrl
    } catch (error) {
      console.warn('Failed to load/cached image:', error)
      cacheFailure(cacheKey, error instanceof Error ? error.message : String(error))

      return null
    } finally {
      loadingPromises.delete(cacheKey)
    }
  })()

  loadingPromises.set(cacheKey, loadingPromise)

  return loadingPromise
}

const getImageCacheStats = async (): Promise<{
  cache_dir:                  string
  cache_expiry_hours:         number
  failure_cache_expiry_hours: number
  failure_cache_size:         number
  file_count:                 number
  memory_cache_size:          number
  persistent_cache_size:      number
  total_size:                 number
}> => {
  try {
    const stats = await invoke<string>('get_image_cache_stats')
    const backendStats = JSON.parse(stats)

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
      cache_expiry_hours:         CACHE_EXPIRY_HOURS,
      failure_cache_expiry_hours: FAILURE_CACHE_EXPIRY_HOURS,
      failure_cache_size:         failureCacheSize,
      memory_cache_size:          dataUrlCache.size,
      persistent_cache_size:      persistentCacheSize,
    }
  } catch (error) {
    console.error('Failed to get image cache stats:', error)
    throw error
  }
}

const preloadRecentImages = async (serverUrl: string, token: string, limit: number = 10): Promise<void> => {
  try {
    const recentEntries = Array.from(cacheMetadata.entries())
      .sort(([, a], [, b]) => b.timestamp - a.timestamp) // Most recent first
      .slice(0, limit)

    if (recentEntries.length === 0) return

    const validEntries = recentEntries.filter(([cacheKey]) => !failureCache.has(cacheKey))

    if (validEntries.length === 0) return

    const preloadPromises = validEntries.map(async ([cacheKey]) => {
      const [itemId, imageType] = cacheKey.split('_')

      try {
        await getImageUrl(itemId, serverUrl, token, imageType)
      } catch (error) {
        console.warn(`Failed to preload image ${cacheKey}:`, error)
      }
    })

    await Promise.allSettled(preloadPromises)
  } catch (error) {
    console.warn('Failed to preload recent images:', error)
  }
}

export interface ImageLoader {
  clearImageFromCache: (itemId: string, imageType?: string) => void
  getImageCacheStats:  () => Promise<{
    cache_dir:                  string
    cache_expiry_hours:         number
    failure_cache_expiry_hours: number
    failure_cache_size:         number
    file_count:                 number
    memory_cache_size:          number
    persistent_cache_size:      number
    total_size:                 number
  }>
  getImageUrl:         (itemId: string, serverUrl: string, token: string, imageType?: string) => Promise<null | string>
  preloadRecentImages: (serverUrl: string, token: string, limit?: number) => Promise<void>
}

export const useImageLoader = (): ImageLoader => ({
  clearImageFromCache,
  getImageCacheStats,
  getImageUrl,
  preloadRecentImages,
})
