import { readonly, ref, type Ref } from 'vue'

import type { Credentials } from '@/bindings'

import { commands } from '@/bindings'
import { authLogger } from '@/lib/logger'
import { withCustomState } from '@/lib/result'
import { useAuthStore } from '@/stores'

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

  // Network-related errors
  if (lowerMessage.includes('network') ||
      lowerMessage.includes('connection') ||
      lowerMessage.includes('timeout') ||
      lowerMessage.includes('unreachable')) {
    return {
      code:        'NETWORK_ERROR',
      isRetryable: true,
      message:     'Unable to connect to the server. Please check your internet connection and server URL.',
      type:        'network',
    }
  }

  // Authentication errors
  if (lowerMessage.includes('authentication') ||
      lowerMessage.includes('login') ||
      lowerMessage.includes('credentials') ||
      lowerMessage.includes('password') ||
      lowerMessage.includes('unauthorized')) {
    return {
      code:        'AUTH_FAILED',
      isRetryable: false,
      message:     'Invalid username or password. Please check your credentials.',
      type:        'auth',
    }
  }

  // Configuration errors
  if (lowerMessage.includes('configuration') ||
      lowerMessage.includes('config') ||
      lowerMessage.includes('corrupted') ||
      lowerMessage.includes('directory') ||
      lowerMessage.includes('file')) {
    return {
      code:        'CONFIG_ERROR',
      isRetryable: false,
      message:     'Application configuration issue. Please try restarting the application.',
      type:        'config',
    }
  }

  // Default unknown error
  return {
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
  authLogger.debug('Checking for saved credentials...')

  await withCustomState(
    () => commands.getSavedCredentials(),
    {
      onError: errorString => {
        authLogger.error('Failed to load saved credentials:', errorString)
        error.value = categorizeAuthError(errorString)
        authStatus.value = 'error'
      },
      onStart: () => {
        authStatus.value = 'pending'
      },
      onSuccess: savedCredentials => {
        authLogger.debug('Got saved credentials:', savedCredentials)

        if (savedCredentials && savedCredentials.token) {
          authLogger.debug('Found saved credentials:', savedCredentials)
          credentials.value = savedCredentials
          authStore.setCredentials(savedCredentials)
          authStatus.value = 'loggedIn'
          error.value = null
          authLogger.debug('Auth store populated:', {
            hasToken:  !!authStore.token,
            serverUrl: authStore.serverUrl,
            userId:    authStore.userId,
            username:  authStore.username,
          })
        } else {
          authLogger.debug('No saved credentials found')
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
