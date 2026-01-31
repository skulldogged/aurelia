import { readonly, ref, type Ref } from 'vue'

import type { Credentials } from '../generated'

import { getApiClient } from '../index'
import { getAuthLogout, setAuthLogout } from '../lib/auth-interceptor'
import { logger } from '../lib/logger'
import { withCustomState } from '../lib/result'
import { useAuthStore } from '../stores'

export interface AuthError {
  code?:        string
  isRetryable?: boolean
  message:      string
  type:         AuthErrorType
}

export type AuthErrorType = 'auth' | 'config' | 'network' | 'unknown'

export type AuthStatus = 'error' | 'initializing' | 'loggedIn' | 'loggedOut' | 'pending' | 'verifying'

const STORAGE_KEY = 'aurelia-auth-credentials'

/// Load credentials from localStorage synchronously on module init
const loadCredentialsFromStorage = (): Credentials | null => {
  try {
    const stored = localStorage.getItem(STORAGE_KEY)
    if (stored) {
      return JSON.parse(stored) as Credentials
    }
  } catch {
    // Ignore parse errors
  }
  return null
}

/// Save credentials to localStorage
const saveCredentialsToStorage = (creds: Credentials | null): void => {
  try {
    if (creds) {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(creds))
    } else {
      localStorage.removeItem(STORAGE_KEY)
    }
  } catch {
    // Ignore storage errors
  }
}

/// Categorize error messages from backend into structured error types
const categorizeAuthError = (errorMessage: string): AuthError => {
  const lowerMessage = errorMessage.toLowerCase()

  const errorPatterns: Array<{
    code:        string
    isRetryable: boolean
    keywords:    string[]
    message:     string
    type:        AuthErrorType
  }> = [
    {
      code:        'NETWORK_ERROR',
      isRetryable: true,
      keywords:    ['network', 'connection', 'timeout', 'unreachable'],
      message:     'Unable to connect to the server. Please check your internet connection and server URL.',
      type:        'network',
    },
    {
      code:        'AUTH_FAILED',
      isRetryable: false,
      keywords:    ['authentication', 'login', 'credentials', 'password', 'unauthorized'],
      message:     'Invalid username or password. Please check your credentials.',
      type:        'auth',
    },
    {
      code:        'CONFIG_ERROR',
      isRetryable: false,
      keywords:    ['configuration', 'config', 'corrupted', 'directory', 'file'],
      message:     'Application configuration issue. Please try restarting the application.',
      type:        'config',
    },
  ]

  const matchedPattern = errorPatterns.find(pattern =>
    pattern.keywords.some(keyword => lowerMessage.includes(keyword)),
  )

  return matchedPattern || {
    code:        'UNKNOWN_ERROR',
    isRetryable: true,
    message:     errorMessage,
    type:        'unknown',
  }
}

// Synchronously load credentials from localStorage on module init
const storedCreds = loadCredentialsFromStorage()
const authStatus = ref<AuthStatus>(storedCreds ? 'verifying' : 'initializing')
const credentials = ref<Credentials | null>(storedCreds)
const error = ref<AuthError | null>(null)

const verifyStoredCredentials = async (authStore: ReturnType<typeof useAuthStore>): Promise<void> => {
  logger.debug('Verifying stored credentials...')

  await withCustomState<Credentials | null, string>(
    () => getApiClient().getSavedCredentials(),
    {
      onError: errorString => {
        logger.error('Failed to verify credentials:', errorString)
        // If verification fails, logout
        logout(authStore)
      },
      onSuccess: savedCredentials => {
        logger.debug('Verified credentials:', savedCredentials)

        if (savedCredentials && savedCredentials.token) {
          // Credentials still valid, sync with backend state
          credentials.value = savedCredentials
          authStore.setCredentials(savedCredentials)
          authStatus.value = 'loggedIn'
          // Sync localStorage in case backend has different creds
          saveCredentialsToStorage(savedCredentials)
          logger.info('Credentials verified successfully')
        } else {
          // Backend has no credentials, logout
          logger.debug('Backend has no credentials, logging out')
          logout(authStore)
        }
      },
    },
  )
}

const initializeAuth = async (authStore: ReturnType<typeof useAuthStore>): Promise<void> => {
  logger.debug('Checking for saved credentials...')
  // Immediately set to pending to indicate we're actively checking
  authStatus.value = 'pending'

  await withCustomState<Credentials | null, string>(
    () => getApiClient().getSavedCredentials(),
    {
      onError: errorString => {
        logger.error('Failed to load saved credentials:', errorString)
        error.value = categorizeAuthError(errorString)
        authStatus.value = 'error'
      },
      onSuccess: savedCredentials => {
        logger.debug('Got saved credentials:', savedCredentials)

        if (savedCredentials && savedCredentials.token) {
          logger.debug('Found saved credentials:', savedCredentials)
          credentials.value = savedCredentials
          authStore.setCredentials(savedCredentials)
          authStatus.value = 'loggedIn'
          error.value = null
          // Sync to localStorage for fast load on next visit
          saveCredentialsToStorage(savedCredentials)
          logger.info('Authentication successful - credentials loaded from disk')
          logger.debug('Auth store populated:', {
            hasToken:  !!authStore.token,
            serverUrl: authStore.serverUrl,
            userId:    authStore.userId,
            username:  authStore.username,
          })
        } else {
          logger.debug('No saved credentials found')
          authStatus.value = 'loggedOut'
          error.value = null
        }
      },
    },
  )
}

const login = (authStore: ReturnType<typeof useAuthStore>, loginCredentials: Credentials): void => {
  credentials.value = loginCredentials
  authStore.setCredentials(loginCredentials)
  authStatus.value = 'loggedIn'
  error.value = null
  // Save to localStorage for fast load on next visit
  saveCredentialsToStorage(loginCredentials)
  logger.info('User logged in - credentials saved to localStorage')
}

const logout = (authStore: ReturnType<typeof useAuthStore>): void => {
  credentials.value = null
  authStore.clearCredentials()
  authStatus.value = 'loggedOut'
  error.value = null
  // Clear from localStorage
  saveCredentialsToStorage(null)
  logger.info('User logged out - credentials cleared from localStorage')
}

const registerLogoutHandler = (authStore: ReturnType<typeof useAuthStore>): void => {
  // Only register if not already registered (avoid re-registration)
  if (!getAuthLogout()) {
    setAuthLogout(() => logout(authStore))
    logger.debug('Auth logout handler registered')
  }
}

const clearError = (): void => {
  error.value = null
  if (authStatus.value === 'error')
    authStatus.value = 'loggedOut'
}

export interface Auth {
  authStatus:  Readonly<Ref<AuthStatus>>
  clearError:  () => void
  credentials: Readonly<Ref<Credentials | null>>
  error:       Readonly<Ref<AuthError | null>>
  login:       (loginCredentials: Credentials) => void
  logout:      () => void
}

export const useAuth = (): Auth => {
  const authStore = useAuthStore()

  // Register logout handler for interceptor use
  registerLogoutHandler(authStore)

  // Initialize auth on first use
  if (authStatus.value === 'initializing') {
    initializeAuth(authStore)
  } else if (authStatus.value === 'verifying') {
    // We have cached credentials, populate the store immediately so API calls work
    // then verify them in the background
    if (credentials.value) {
      authStore.setCredentials(credentials.value)
    }
    verifyStoredCredentials(authStore)
  }

  return {
    authStatus:  readonly(authStatus),
    clearError,
    credentials: readonly(credentials),
    error:       readonly(error),
    login:       loginCredentials => login(authStore, loginCredentials),
    logout:      () => logout(authStore),
  }
}
