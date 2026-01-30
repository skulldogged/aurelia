import { readonly, ref, type Ref } from 'vue'

import type { Credentials } from '../lib/api/types'

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

export type AuthStatus = 'error' | 'loggedIn' | 'loggedOut' | 'pending'

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

const authStatus = ref<AuthStatus>('pending')
const credentials = ref<Credentials | null>(null)
const error = ref<AuthError | null>(null)

const initializeAuth = async (authStore: ReturnType<typeof useAuthStore>): Promise<void> => {
  logger.debug('Checking for saved credentials...')

  await withCustomState<Credentials | null, string>(
    () => getApiClient().getSavedCredentials(),
    {
      onError: errorString => {
        logger.error('Failed to load saved credentials:', errorString)
        error.value = categorizeAuthError(errorString)
        authStatus.value = 'error'
      },
      onStart: () => {
        authStatus.value = 'pending'
      },
      onSuccess: savedCredentials => {
        logger.debug('Got saved credentials:', savedCredentials)

        if (savedCredentials && savedCredentials.token) {
          logger.debug('Found saved credentials:', savedCredentials)
          credentials.value = savedCredentials
          authStore.setCredentials(savedCredentials)
          authStatus.value = 'loggedIn'
          error.value = null
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
}

const logout = (authStore: ReturnType<typeof useAuthStore>): void => {
  credentials.value = null
  authStore.clearCredentials()
  authStatus.value = 'loggedOut'
  error.value = null
  logger.info('User logged out')
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
  if (authStatus.value === 'pending')
    initializeAuth(authStore)

  return {
    authStatus:  readonly(authStatus),
    clearError,
    credentials: readonly(credentials),
    error:       readonly(error),
    login:       loginCredentials => login(authStore, loginCredentials),
    logout:      () => logout(authStore),
  }
}
