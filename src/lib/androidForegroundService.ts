import { invoke } from '@tauri-apps/api/core'

import { logger } from '@/lib/logger'

interface PluginResponse {
  message?: string
  success:  boolean
}

export interface AndroidNowPlayingPayload {
  album?:           null | string
  artists:          string[]
  artworkData?:     null | string
  artworkPath?:     null | string
  artworkUrl?:      null | string
  durationSeconds?: null | number
  hasNext:          boolean
  hasPrevious:      boolean
  id?:              null | string
  isPlaying:        boolean
  isShuffled:       boolean
  positionSeconds?: null | number
  repeatMode?:      null | string
  title:            string
}

const UPDATE_COMMAND = 'plugin:android-now-playing|update_now_playing'
const CLEAR_COMMAND = 'plugin:android-now-playing|clear_now_playing'

const handleResult = (response: PluginResponse, context: string): boolean => {
  if (response.success) return true

  logger.error(`Android now playing plugin reported error during ${context}:`, response.message)
  return false
}

export const AndroidNowPlayingService = {
  async update(payload: AndroidNowPlayingPayload): Promise<boolean> {
    try {
      const result = await invoke<PluginResponse>(UPDATE_COMMAND, { payload })
      return handleResult(result, 'update')
    } catch (error) {
      logger.error('Failed to update Android now playing service:', error)
      return false
    }
  },

  async clear(): Promise<boolean> {
    try {
      const result = await invoke<PluginResponse>(CLEAR_COMMAND)
      return handleResult(result, 'clear')
    } catch (error) {
      logger.error('Failed to clear Android now playing service:', error)
      return false
    }
  },
}
