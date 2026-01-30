import { v4 as uuidv4 } from 'uuid'
import { readonly, ref, type Ref } from 'vue'

import { getApiClient } from '../index'
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
    const result =
      await getApiClient().registerClientCapabilities(
        authStore.serverUrl,
        authStore.token,
        sessionState.value.deviceId,
      )
    if (result.status === 'error') {
      logger.error('Failed to register client capabilities:', result.error)
      return
    }

    sessionState.value.isRegistered = true
    logger.info('Client capabilities registered with Jellyfin')
  } catch (error) {
    logger.error('Failed to register client capabilities:', error)
  }
}

const reportPlaybackStart = async (
  authStore: ReturnType<typeof useAuthStore>,
  itemId: string,
  positionTicks?: number,
): Promise<void> => {
  if (!authStore.serverUrl || !authStore.token || !sessionState.value.isRegistered)
    return

  try {
    const ticks = positionTicks ? Math.floor(positionTicks * 10_000_000) : null
    const result = await getApiClient().reportPlaybackStart(authStore.serverUrl, authStore.token, itemId, ticks)
    if (result.status === 'error') {
      logger.error('Failed to report playback start:', result.error)
      return
    }
    logger.debug('Playback start reported', { itemId, positionTicks })
  } catch (error) {
    logger.error('Failed to report playback start:', error)
  }
}

const reportPlaybackProgress = async (
  authStore: ReturnType<typeof useAuthStore>,
  itemId: string,
  positionTicks: number,
  eventName?: string,
  isPaused?: boolean,
): Promise<void> => {
  if (!authStore.serverUrl || !authStore.token || !sessionState.value.isRegistered)
    return

  try {
    const result = await getApiClient().reportPlaybackProgress(
      authStore.serverUrl,
      authStore.token,
      itemId,
      Math.floor(positionTicks * 10_000_000),
      eventName ?? null,
      isPaused ?? null,
    )
    if (result.status === 'error') {
      logger.debug('Failed to report playback progress:', result.error)
      return
    }
  } catch (error) {
    logger.debug('Failed to report playback progress:', error)
  }
}

const reportPlaybackStop = async (
  authStore: ReturnType<typeof useAuthStore>,
  itemId: string,
  positionTicks?: number,
): Promise<void> => {
  if (!authStore.serverUrl || !authStore.token || !sessionState.value.isRegistered)
    return

  try {
    const ticks = positionTicks ? Math.floor(positionTicks * 10_000_000) : null
    const result = await getApiClient().reportPlaybackStop(authStore.serverUrl, authStore.token, itemId, ticks)
    if (result.status === 'error') {
      logger.error('Failed to report playback stop:', result.error)
      return
    }

    logger.debug('Playback stop reported', { itemId, positionTicks })

    sessionState.value.playSessionId = `play-${Date.now()}-${Math.random().toString(36).slice(2, 11)}`
  } catch (error) {
    logger.error('Failed to report playback stop:', error)
  }
}

const markItemPlayed = async (
  authStore: ReturnType<typeof useAuthStore>,
  itemId: string,
): Promise<void> => {
  if (!authStore.serverUrl || !authStore.token || !authStore.userId)
    return

  try {
    const result = await getApiClient().markItemPlayed(authStore.serverUrl, authStore.token, authStore.userId, itemId)
    if (result.status === 'error') {
      logger.error('Failed to mark item as played:', result.error)
      return
    }

    logger.debug('Item marked as played', { itemId })
  } catch (error) {
    logger.error('Failed to mark item as played:', error)
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
    positionTicks: number,
    eventName?: string,
    isPaused?: boolean,
  ) => Promise<void>
  reportPlaybackStart:    (
    itemId: string,
    positionTicks?: number,
  ) => Promise<void>
  reportPlaybackStop: (
    itemId: string,
    positionTicks?: number,
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
    reportPlaybackProgress: (itemId, positionTicks, eventName, isPaused) =>
      reportPlaybackProgress(authStore, itemId, positionTicks, eventName, isPaused),
    reportPlaybackStart: (itemId, positionTicks) => reportPlaybackStart(authStore, itemId, positionTicks),
    reportPlaybackStop:  (itemId, positionTicks) => reportPlaybackStop(authStore, itemId, positionTicks),
    sessionState:        readonly(sessionState),
  }
}
