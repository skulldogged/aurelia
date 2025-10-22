import { invoke } from '@tauri-apps/api/core'

import { logger } from '@/lib/logger'

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

interface PluginResponse {
  message?: string
  success:  boolean
}

const UPDATE_COMMAND = 'plugin:android-now-playing|update_now_playing'
const CLEAR_COMMAND = 'plugin:android-now-playing|clear_now_playing'

let lastErrorSignature: null | string = null

const resetError = (): void => {
  lastErrorSignature = null
}

const logPluginError = (context: 'clear' | 'update', message?: string): void => {
  const finalMessage = message ?? 'Unknown error'
  const signature = `plugin:${context}:${finalMessage}`
  if (lastErrorSignature === signature) return
  lastErrorSignature = signature
  logger.error(`Android now playing plugin reported error during ${context}:`, finalMessage)
}

const logInvokeError = (context: 'clear' | 'update', error: unknown): void => {
  const message = error instanceof Error ? error.message : String(error)
  const signature = `invoke:${context}:${message}`
  if (lastErrorSignature === signature) return
  lastErrorSignature = signature
  logger.error(`Failed to ${context} Android now playing service:`, error)
}

const handleResult = (response: PluginResponse, context: 'clear' | 'update'): boolean => {
  if (response.success) {
    resetError()
    return true
  }

  logPluginError(context, response.message)
  return false
}

export const AndroidNowPlayingService = {
  clear: async (): Promise<boolean> => {
    try {
      const result = await invoke<PluginResponse>(CLEAR_COMMAND)
      return handleResult(result, 'clear')
    } catch (error) {
      logInvokeError('clear', error)
      return false
    }
  },

  update: async (payload: AndroidNowPlayingPayload): Promise<boolean> => {
    try {
      const result = await invoke<PluginResponse>(UPDATE_COMMAND, { payload })
      return handleResult(result, 'update')
    } catch (error) {
      logInvokeError('update', error)
      return false
    }
  },
}
