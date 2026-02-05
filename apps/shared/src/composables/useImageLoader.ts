import type { Result } from '../lib/api/result'

import { ApiError, runAureliaEffect } from '../effect'
import { clearImageFromCacheEffect, getImageCacheStatsEffect, getImageEffect } from '../effect/services/api'
import { logger } from '../lib/logger'
import { LRUCache } from '../lib/lru-cache'
import { isTauri } from '../lib/platform'

// LRU cache for asset URLs with bounded size to prevent memory leaks
// 2000 entries covers typical browsing patterns while limiting memory usage
const MAX_CACHE_SIZE = 2000
const assetUrlCache = new LRUCache<string, string>(MAX_CACHE_SIZE)

const generateCacheKey = (itemId: string, imageType: string, width?: number, quality?: number): string => {
  let key = `${itemId}_${imageType}`
  if (width) key += `_w${width}`
  if (quality) key += `_q${quality}`
  return key
}

const getImageUrlFromCache = (
  itemId: string,
  imageType: string = 'Primary',
  width?: number,
  quality?: number,
): string | undefined =>
  assetUrlCache.get(generateCacheKey(itemId, imageType, width, quality))

const getImageUrl = async (
  itemId: string,
  serverUrl: string,
  token: string,
  imageType: string = 'Primary',
  width?: number,
  quality?: number,
): Promise<null | string> => {
  if (!itemId || !serverUrl || !token) return null

  const cacheKey = generateCacheKey(itemId, imageType, width, quality)
  if (assetUrlCache.has(cacheKey)) {
    return assetUrlCache.get(cacheKey)!
  }

  try {
    const imagePath = await runAureliaEffect(
      getImageEffect(itemId, imageType, serverUrl, token, width, quality),
    )
    if (!imagePath)
      return null

    // On web, data is already a URL. On desktop, it's a file path that needs conversion.
    let assetUrl: string
    if (isTauri()) {
      const { convertFileSrc } = await import('@tauri-apps/api/core')
      assetUrl = convertFileSrc(imagePath)
    } else {
      assetUrl = imagePath
    }

    assetUrlCache.set(cacheKey, assetUrl)
    return assetUrl
  } catch (cause) {
    const errorMessage = cause instanceof ApiError
      ? cause.message
      : String(cause)
    logger.warn(`Failed to get image for ${itemId}: ${errorMessage}`)
    return null
  }
}

const clearImageFromCache = async (itemId: string, imageType: string = 'Primary'): Promise<void> => {
  // Clear all variations from memory cache using LRU deleteByPrefix
  const prefix = `${itemId}_${imageType}`
  assetUrlCache.deleteByPrefix(prefix)
  try {
    await runAureliaEffect(clearImageFromCacheEffect(itemId, imageType))
  } catch (cause) {
    logger.warn('Failed to clear image from cache', cause)
  }
}

const getImageCacheStats = async (): Promise<Result<{
  cache_dir:  string
  file_count: number
  total_size: number
}, string>> => {
  try {
    const rawStats = await runAureliaEffect(getImageCacheStatsEffect())
    const stats = JSON.parse(rawStats)
    return { data: stats, status: 'ok' }
  } catch (error) {
    if (error instanceof ApiError)
      return { error: error.message, status: 'error' }
    return { error: `Failed to parse cache stats: ${error}`, status: 'error' }
  }
}

export interface ImageLoader {
  clearImageFromCache: (itemId: string, imageType?: string) => Promise<void>
  getImageCacheStats: () => Promise<Result<{
    cache_dir:  string
    file_count: number
    total_size: number
  }, string>>
  getImageUrl: (
    itemId: string,
    serverUrl: string,
    token: string,
    imageType?: string,
    width?: number,
    quality?: number,
  ) => Promise<null | string>
  getImageUrlFromCache: (
    itemId: string,
    imageType?: string,
    width?: number,
    quality?: number,
  ) => string | undefined
}

export const useImageLoader = (): ImageLoader => ({
  clearImageFromCache,
  getImageCacheStats,
  getImageUrl,
  getImageUrlFromCache,
})
