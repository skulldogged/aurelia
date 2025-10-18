import { convertFileSrc } from '@tauri-apps/api/core'

import { commands, type Result } from '@/bindings'
import { logger } from '@/lib/logger'
import { err, ok } from '@/lib/result'

// In-memory cache for asset URLs to avoid flicker
const assetUrlCache = new Map<string, string>()

const getImageUrlFromCache = (itemId: string, imageType: string = 'Primary'): string | undefined => {
  const cacheKey = `${itemId}_${imageType}`
  return assetUrlCache.get(cacheKey)
}

const getImageUrl = async (
  itemId: string,
  serverUrl: string,
  token: string,
  imageType: string = 'Primary',
): Promise<null | string> => {
  if (!itemId || !serverUrl || !token) return null

  const cacheKey = `${itemId}_${imageType}`
  if (assetUrlCache.has(cacheKey)) {
    return assetUrlCache.get(cacheKey)!
  }

  const result = await commands.getImage(itemId, imageType, serverUrl, token)
  if (result.status === 'error' || !result.data) {
    if (result.status === 'error') {
      logger.warn(`Failed to get image for ${itemId}: ${result.error}`)
    }
    return null
  }

  const assetUrl = convertFileSrc(result.data)
  assetUrlCache.set(cacheKey, assetUrl)

  return assetUrl
}

const clearImageFromCache = async (itemId: string, imageType: string = 'Primary'): Promise<void> => {
  const cacheKey = `${itemId}_${imageType}`
  assetUrlCache.delete(cacheKey)
  await commands.clearImageFromCache(itemId, imageType)
}

const getImageCacheStats = async (): Promise<Result<{
  cache_dir:  string
  file_count: number
  total_size: number
}, string>> => {
  const statsResult = await commands.getImageCacheStats()
  if (statsResult.status === 'error')
    return err(statsResult.error)

  try {
    const stats = JSON.parse(statsResult.data)
    return ok(stats)
  } catch (error) {
    return err(`Failed to parse cache stats: ${error}`)
  }
}

export interface ImageLoader {
  clearImageFromCache: (itemId: string, imageType?: string) => Promise<void>
  getImageCacheStats: () => Promise<Result<{
    cache_dir:  string
    file_count: number
    total_size: number
  }, string>>
  getImageUrl: (itemId: string, serverUrl: string, token: string, imageType?: string) => Promise<null | string>
  getImageUrlFromCache: (itemId: string, imageType?: string) => string | undefined
}

export const useImageLoader = (): ImageLoader => ({
  clearImageFromCache,
  getImageCacheStats,
  getImageUrl,
  getImageUrlFromCache,
})
