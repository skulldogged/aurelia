import { convertFileSrc } from '@tauri-apps/api/core'

import { commands, type Result } from '@/bindings'
import { logger } from '@/lib/logger'
import { isMobile } from '@/lib/platform'
import { err, ok } from '@/lib/result'

// In-memory cache for asset URLs to avoid flicker
const assetUrlCache = new Map<string, string>()

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
  // Temporarily disable image loading on mobile to debug OOM issues
  if (isMobile()) return null

  if (!itemId || !serverUrl || !token) return null

  const cacheKey = generateCacheKey(itemId, imageType, width, quality)
  if (assetUrlCache.has(cacheKey)) {
    return assetUrlCache.get(cacheKey)!
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const result = await (commands as any).getImage(itemId, imageType, serverUrl, token, width ?? null, quality ?? null)
  if (result.status === 'error' || !result.data) {
    if (result.status === 'error')
      logger.warn(`Failed to get image for ${itemId}: ${result.error}`)

    return null
  }

  const assetUrl = convertFileSrc(result.data)
  assetUrlCache.set(cacheKey, assetUrl)

  return assetUrl
}

const clearImageFromCache = async (itemId: string, imageType: string = 'Primary'): Promise<void> => {
  // Clear all variations from memory cache
  const prefix = `${itemId}_${imageType}`
  for (const key of assetUrlCache.keys()) {
    if (key.startsWith(prefix)) {
      assetUrlCache.delete(key)
    }
  }
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
