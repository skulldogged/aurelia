import { ref, readonly } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { getVersion } from '@tauri-apps/api/app'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
import { useAuthStore } from '@/stores'
import { logger } from '@/lib/logger'

interface SessionState {
  sessionId:     string | null
  playSessionId: string | null
  isRegistered:  boolean
  deviceId:      string
  deviceName:    string
  appVersion:    string
}

export const useSession = () => {
  const authStore = useAuthStore()

  // Session state
  const sessionState = ref<SessionState>({
    sessionId:     null,
    playSessionId: null,
    isRegistered:  false,
    deviceId:      '',
    deviceName:    '',
    appVersion:    '',
  })

  // Initialize session management
  const initializeSession = async () => {
    try {
      // Generate unique device ID
      const webview = getCurrentWebviewWindow()
      const label = webview.label
      const version = await getVersion()

      sessionState.value.deviceId = `tauri-music-player-${label}-${Date.now()}`
      sessionState.value.deviceName = `Tauri Music Player (${label})`
      sessionState.value.appVersion = version

      // Generate initial session IDs
      sessionState.value.sessionId = `session-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
      sessionState.value.playSessionId = `play-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`

      logger.info('Session initialized', {
        deviceId:   sessionState.value.deviceId,
        deviceName: sessionState.value.deviceName,
        sessionId:  sessionState.value.sessionId,
      })
    } catch (error) {
      logger.error('Failed to initialize session:', error)
    }
  }

  // Register client capabilities with Jellyfin
  const registerCapabilities = async () => {
    if (!authStore.serverUrl || !authStore.token || sessionState.value.isRegistered) {
      return
    }

    try {
      await invoke('register_client_capabilities', {
        serverUrl:  authStore.serverUrl,
        token:      authStore.token,
        deviceName: sessionState.value.deviceName,
        deviceId:   sessionState.value.deviceId,
        appVersion: sessionState.value.appVersion,
      })

      sessionState.value.isRegistered = true
      logger.info('Client capabilities registered with Jellyfin')
    } catch (error) {
      logger.error('Failed to register client capabilities:', error)
      // Don't throw - registration failure shouldn't break playback
    }
  }

  // Report playback start
  const reportPlaybackStart = async (
    itemId: string,
    positionTicks?: number,
  ) => {
    if (!authStore.serverUrl || !authStore.token || !sessionState.value.isRegistered) {
      return
    }

    try {
      await invoke('report_playback_start', {
        serverUrl:     authStore.serverUrl,
        token:         authStore.token,
        itemId,
        positionTicks: positionTicks ? Math.floor(positionTicks * 10000) : undefined,
      })
      logger.debug('Playback start reported', { itemId, positionTicks })
    } catch (error) {
      logger.error('Failed to report playback start:', error)
    }
  }

  // Report playback progress
  const reportPlaybackProgress = async (
    itemId: string,
    positionTicks: number,
    eventName?: string,
    isPaused?: boolean,
  ) => {
    if (!authStore.serverUrl || !authStore.token || !sessionState.value.isRegistered) {
      return
    }

    try {
      await invoke('report_playback_progress', {
        serverUrl:     authStore.serverUrl,
        token:         authStore.token,
        itemId,
        positionTicks: Math.floor(positionTicks * 10000),
        eventName,
        isPaused,
      })
    } catch (error) {
      logger.debug('Failed to report playback progress:', error)
    }
  }

  // Report playback stop
  const reportPlaybackStop = async (itemId: string, positionTicks?: number) => {
    if (!authStore.serverUrl || !authStore.token || !sessionState.value.isRegistered) {
      return
    }

    try {
      await invoke('report_playback_stop', {
        serverUrl:     authStore.serverUrl,
        token:         authStore.token,
        itemId,
        positionTicks: positionTicks ? Math.floor(positionTicks * 10000) : undefined,
      })

      logger.debug('Playback stop reported', { itemId, positionTicks })

      // Generate new play session ID for next playback
      sessionState.value.playSessionId = `play-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
    } catch (error) {
      logger.error('Failed to report playback stop:', error)
    }
  }

  // Mark item as played
  const markItemPlayed = async (itemId: string) => {
    if (!authStore.serverUrl || !authStore.token || !authStore.userId) {
      return
    }

    try {
      await invoke('mark_item_played', {
        serverUrl: authStore.serverUrl,
        token:     authStore.token,
        userId:    authStore.userId,
        itemId,
      })

      logger.debug('Item marked as played', { itemId })
    } catch (error) {
      logger.error('Failed to mark item as played:', error)
    }
  }

  // Generate new session ID (useful when restarting playback session)
  const generateNewSession = () => {
    sessionState.value.sessionId = `session-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
    sessionState.value.playSessionId = `play-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
    logger.debug('New session generated', {
      sessionId:     sessionState.value.sessionId,
      playSessionId: sessionState.value.playSessionId,
    })
  }

  return {
    // State
    sessionState: readonly(sessionState),

    // Methods
    initializeSession,
    registerCapabilities,
    reportPlaybackStart,
    reportPlaybackProgress,
    reportPlaybackStop,
    markItemPlayed,
    generateNewSession,
  }
}
