import { v4 as uuidv4 } from 'uuid'
import { readonly, ref, type Ref } from 'vue'

import { ApiError } from '../effect/errors'
import { runAureliaEffect } from '../effect/runtime'
import {
  markItemPlayedEffect,
  registerClientCapabilitiesEffect,
  reportPlaybackProgressEffect,
  reportPlaybackStartEffect,
  reportPlaybackStopEffect,
} from '../effect/services/api'
import { logger } from '../lib/logger'
import { isTauri } from '../lib/platform'
import { useAuthStore } from '../stores'

interface SessionState {
  appVersion:    string
  deviceId:      string
  deviceName:    string
  isRegistered:  boolean
  playSessionId: null | string
  sessionId:     null | string
}

const sessionState = ref<SessionState>({
  appVersion:    '',
  deviceId:      '',
  deviceName:    '',
  isRegistered:  false,
  playSessionId: null,
  sessionId:     null,
})

const initializeSession = async (): Promise<void> => {
  try {
    let label = 'web'
    let version = '0.0.0'

    // Check if running in Tauri - use dynamic import
    if (isTauri()) {
      const { getCurrentWebviewWindow } = await import('@tauri-apps/api/webviewWindow')
      const { getVersion } = await import('@tauri-apps/api/app')
      const webview = getCurrentWebviewWindow()
      label = webview.label
      version = await getVersion()
    }

    let deviceId = localStorage.getItem('aurelia-device-id')
    if (!deviceId) {
      deviceId = uuidv4()
      localStorage.setItem('aurelia-device-id', deviceId)
    }

    sessionState.value.deviceId = `aurelia-${label}-${deviceId}`
    sessionState.value.deviceName = `Aurelia Music (${label})`
    sessionState.value.appVersion = version

    sessionState.value.sessionId = `session-${Date.now()}-${Math.random().toString(36).slice(2, 11)}`
    sessionState.value.playSessionId = `play-${Date.now()}-${Math.random().toString(36).slice(2, 11)}`

    logger.info('Session initialized', {
      deviceId:   sessionState.value.deviceId,
      deviceName: sessionState.value.deviceName,
      sessionId:  sessionState.value.sessionId,
    })
  } catch (error) {
    logger.error('Failed to initialize session:', error)
  }
}

const registerCapabilities = async (authStore: ReturnType<typeof useAuthStore>): Promise<void> => {
  if (!authStore.serverUrl || !authStore.token || sessionState.value.isRegistered)
    return

  logger.debug(
    'Registering client capabilities with server:',
    authStore.serverUrl,
    'token length:',
    authStore.token.length,
  )

  try {
    await runAureliaEffect(registerClientCapabilitiesEffect(
      authStore.serverUrl,
      authStore.token,
      sessionState.value.deviceId,
    ))

    sessionState.value.isRegistered = true
    logger.info('Client capabilities registered with Jellyfin')
  } catch (cause) {
    const errorMessage = cause instanceof ApiError
      ? cause.message
      : String(cause)
    logger.error('Failed to register client capabilities:', errorMessage)
  }
}

const reportPlaybackStart = async (
  authStore: ReturnType<typeof useAuthStore>,
  itemId: string,
  positionSeconds?: number,
): Promise<void> => {
  if (!authStore.serverUrl || !authStore.token || !sessionState.value.isRegistered)
    return

  try {
    // Convert seconds to ticks (10,000,000 ticks per second) and ensure it's an integer
    const positionTicks = positionSeconds !== undefined
      ? Math.floor(positionSeconds * 10_000_000)
      : undefined

    await runAureliaEffect(reportPlaybackStartEffect(itemId, positionTicks))
    logger.debug('Playback start reported', { itemId, positionSeconds, positionTicks })
  } catch (cause) {
    const errorMessage = cause instanceof ApiError
      ? cause.message
      : String(cause)
    logger.error('Failed to report playback start:', errorMessage)
  }
}

const reportPlaybackProgress = async (
  authStore: ReturnType<typeof useAuthStore>,
  itemId: string,
  positionSeconds: number,
  _eventName?: string,
  isPaused?: boolean,
): Promise<void> => {
  if (!authStore.serverUrl || !authStore.token || !sessionState.value.isRegistered)
    return

  try {
    // Convert seconds to ticks (10,000,000 ticks per second) and ensure it's an integer
    const positionTicks = Math.floor(positionSeconds * 10_000_000)

    await runAureliaEffect(reportPlaybackProgressEffect(itemId, positionTicks, isPaused ?? false))
  } catch (cause) {
    const errorMessage = cause instanceof ApiError
      ? cause.message
      : String(cause)
    logger.debug('Failed to report playback progress:', errorMessage)
  }
}

const reportPlaybackStop = async (
  authStore: ReturnType<typeof useAuthStore>,
  itemId: string,
  positionSeconds?: number,
): Promise<void> => {
  if (!authStore.serverUrl || !authStore.token || !sessionState.value.isRegistered)
    return

  try {
    // Convert seconds to ticks (10,000,000 ticks per second) and ensure it's an integer
    const positionTicks = positionSeconds !== undefined
      ? Math.floor(positionSeconds * 10_000_000)
      : 0

    await runAureliaEffect(reportPlaybackStopEffect(itemId, positionTicks))

    logger.debug('Playback stop reported', { itemId, positionSeconds, positionTicks })

    sessionState.value.playSessionId = `play-${Date.now()}-${Math.random().toString(36).slice(2, 11)}`
  } catch (cause) {
    const errorMessage = cause instanceof ApiError
      ? cause.message
      : String(cause)
    logger.error('Failed to report playback stop:', errorMessage)
  }
}

const markItemPlayed = async (
  authStore: ReturnType<typeof useAuthStore>,
  itemId: string,
): Promise<void> => {
  if (!authStore.serverUrl || !authStore.token)
    return

  try {
    await runAureliaEffect(markItemPlayedEffect(itemId))
    logger.debug('Item marked as played', { itemId })
  } catch (cause) {
    const errorMessage = cause instanceof ApiError
      ? cause.message
      : String(cause)
    logger.error('Failed to mark item as played:', errorMessage)
  }
}

const generateNewSession = (): void => {
  sessionState.value.sessionId = `session-${Date.now()}-${Math.random().toString(36).slice(2, 11)}`
  sessionState.value.playSessionId = `play-${Date.now()}-${Math.random().toString(36).slice(2, 11)}`
  logger.debug('New session generated', {
    playSessionId: sessionState.value.playSessionId,
    sessionId:     sessionState.value.sessionId,
  })
}

export interface Session {
  generateNewSession:     () => void
  initializeSession:      () => Promise<void>
  markItemPlayed:         (itemId: string) => Promise<void>
  registerCapabilities:   () => Promise<void>
  reportPlaybackProgress: (
    itemId: string,
    positionSeconds: number,
    eventName?: string,
    isPaused?: boolean,
  ) => Promise<void>
  reportPlaybackStart:    (
    itemId: string,
    positionSeconds?: number,
  ) => Promise<void>
  reportPlaybackStop: (
    itemId: string,
    positionSeconds?: number,
  ) => Promise<void>
  sessionState: Readonly<Ref<SessionState>>
}

export const useSession = (): Session => {
  const authStore = useAuthStore()

  return {
    generateNewSession,

    initializeSession,
    markItemPlayed:         itemId => markItemPlayed(authStore, itemId),
    registerCapabilities:   () => registerCapabilities(authStore),
    reportPlaybackProgress: (itemId, positionSeconds, eventName, isPaused) =>
      reportPlaybackProgress(authStore, itemId, positionSeconds, eventName, isPaused),
    reportPlaybackStart: (itemId, positionSeconds) => reportPlaybackStart(authStore, itemId, positionSeconds),
    reportPlaybackStop:  (itemId, positionSeconds) => reportPlaybackStop(authStore, itemId, positionSeconds),
    sessionState:        readonly(sessionState),
  }
}
