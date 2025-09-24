import { getVersion } from '@tauri-apps/api/app'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
import { readonly, ref, type Ref } from 'vue'

import { logger } from '@/lib/logger'
import { useAuthStore } from '@/stores'

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
    const webview = getCurrentWebviewWindow()
    const label = webview.label
    const version = await getVersion()

    sessionState.value.deviceId = `tauri-music-player-${label}-${Date.now()}`
    sessionState.value.deviceName = `Tauri Music Player (${label})`
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

  try {
    await invoke('register_client_capabilities', {
      serverUrl: authStore.serverUrl,
      token:     authStore.token,
    })

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
    await invoke('report_playback_start', {
      itemId,
      positionTicks: positionTicks ? Math.floor(positionTicks * 10_000_000) : undefined,
      serverUrl:     authStore.serverUrl,
      token:         authStore.token,
    })
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
    await invoke('report_playback_progress', {
      eventName,
      isPaused,
      itemId,
      positionTicks: Math.floor(positionTicks * 10_000_000),
      serverUrl:     authStore.serverUrl,
      token:         authStore.token,
    })
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
    await invoke('report_playback_stop', {
      itemId,
      positionTicks: positionTicks ? Math.floor(positionTicks * 10_000_000) : undefined,
      serverUrl:     authStore.serverUrl,
      token:         authStore.token,
    })

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
    await invoke('mark_item_played', {
      itemId,
      serverUrl: authStore.serverUrl,
      token:     authStore.token,
      userId:    authStore.userId,
    })

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
