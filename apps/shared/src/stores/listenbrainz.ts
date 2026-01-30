import { defineStore } from 'pinia'
import { ref } from 'vue'

import type { ListenBrainzCredentials } from '../lib/api/types'

import { logger } from '../lib/logger'

export const useListenBrainzStore = defineStore('listenbrainz', () => {
  const credentials = ref<ListenBrainzCredentials | null>(null)
  const isEnabled = ref(false)
  const isScrobblingEnabled = ref(true)

  // Load credentials from localStorage on init
  const storedCreds = localStorage.getItem('listenbrainz-credentials')
  if (storedCreds) {
    try {
      credentials.value = JSON.parse(storedCreds)
      isEnabled.value = true
    } catch (error) {
      logger.error('Failed to parse stored ListenBrainz credentials:', error)
    }
  }

  const setCredentials = (creds: ListenBrainzCredentials): void => {
    credentials.value = creds
    isEnabled.value = true
    localStorage.setItem('listenbrainz-credentials', JSON.stringify(creds))
  }

  const clearCredentials = (): void => {
    credentials.value = null
    isEnabled.value = false
    isScrobblingEnabled.value = true
    localStorage.removeItem('listenbrainz-credentials')
  }

  const setEnabled = (enabled: boolean): void => {
    isEnabled.value = enabled
  }

  const setScrobblingEnabled = (enabled: boolean): void => {
    isScrobblingEnabled.value = enabled
  }

  const isAuthenticated = (): boolean =>
    credentials.value !== null && credentials.value.userToken !== ''

  return {
    clearCredentials,
    credentials,
    isAuthenticated,
    isEnabled,
    isScrobblingEnabled,
    setCredentials,
    setEnabled,
    setScrobblingEnabled,
  }
})
