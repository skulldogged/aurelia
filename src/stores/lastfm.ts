import { defineStore } from 'pinia'
import { ref } from 'vue'

import type { LastFmCredentials } from '@/bindings'

import { lastfmLogger } from '@/lib/logger'

const STORAGE_KEY = 'lastfm-credentials'

const getStoredCredentials = (): LastFmCredentials | null => {
  try {
    const stored = localStorage.getItem(STORAGE_KEY)
    if (!stored)
      return null

    const parsed = JSON.parse(stored)
    lastfmLogger.debug('Loaded Last.fm credentials from localStorage')
    return parsed
  } catch (error) {
    lastfmLogger.warn('Failed to load Last.fm credentials from localStorage:', error)
    return null
  }
}

const setStoredCredentials = (credentials: LastFmCredentials | null): void => {
  try {
    if (credentials) {
      lastfmLogger.debug('Saving Last.fm credentials to localStorage')
      localStorage.setItem(STORAGE_KEY, JSON.stringify(credentials))
    } else {
      lastfmLogger.debug('Removing Last.fm credentials from localStorage')
      localStorage.removeItem(STORAGE_KEY)
    }
  } catch (error) {
    lastfmLogger.warn('Failed to save Last.fm credentials to localStorage:', error)
  }
}

export const useLastFmStore = defineStore('lastfm', () => {
  const credentials = ref<LastFmCredentials | null>(getStoredCredentials())
  const isEnabled = ref(false)
  const isScrobblingEnabled = ref(true)

  const isAuthenticated = (): boolean =>
    credentials.value !== null && credentials.value.session_key !== null

  const setCredentials = (newCredentials: LastFmCredentials | null): void => {
    credentials.value = newCredentials
    setStoredCredentials(newCredentials)
  }

  const clearCredentials = (): void => {
    credentials.value = null
    setStoredCredentials(null)
    isEnabled.value = false
  }

  const setEnabled = (enabled: boolean): void => {
    isEnabled.value = enabled
  }

  const setScrobblingEnabled = (enabled: boolean): void => {
    isScrobblingEnabled.value = enabled
  }

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
