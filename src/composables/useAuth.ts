import { ref, onMounted, readonly } from 'vue'
import type { Credentials } from '@/bindings'
import { commands } from '@/bindings'
import { useAuthStore } from '@/stores'
import { authLogger } from '@/lib/logger'

export type AuthStatus = 'pending' | 'loggedIn' | 'loggedOut' | 'error'

export type AuthErrorType = 'network' | 'auth' | 'config' | 'unknown'

export interface AuthError {
  message:      string
  type:         AuthErrorType
  code?:        string
  isRetryable?: boolean
}

/// Categorize error messages from backend into structured error types
const categorizeAuthError = (errorMessage: string): AuthError => {
  const lowerMessage = errorMessage.toLowerCase()

  // Network-related errors
  if (lowerMessage.includes('network') ||
      lowerMessage.includes('connection') ||
      lowerMessage.includes('timeout') ||
      lowerMessage.includes('unreachable')) {
    return {
      message:     'Unable to connect to the server. Please check your internet connection and server URL.',
      type:        'network',
      code:        'NETWORK_ERROR',
      isRetryable: true,
    }
  }

  // Authentication errors
  if (lowerMessage.includes('authentication') ||
      lowerMessage.includes('login') ||
      lowerMessage.includes('credentials') ||
      lowerMessage.includes('password') ||
      lowerMessage.includes('unauthorized')) {
    return {
      message:     'Invalid username or password. Please check your credentials.',
      type:        'auth',
      code:        'AUTH_FAILED',
      isRetryable: false,
    }
  }

  // Configuration errors
  if (lowerMessage.includes('configuration') ||
      lowerMessage.includes('config') ||
      lowerMessage.includes('corrupted') ||
      lowerMessage.includes('directory') ||
      lowerMessage.includes('file')) {
    return {
      message:     'Application configuration issue. Please try restarting the application.',
      type:        'config',
      code:        'CONFIG_ERROR',
      isRetryable: false,
    }
  }

  // Default unknown error
  return {
    message:     errorMessage,
    type:        'unknown',
    code:        'UNKNOWN_ERROR',
    isRetryable: true,
  }
}

export const useAuth = () => {
  const authStore = useAuthStore()
  const authStatus = ref<AuthStatus>('pending')
  const credentials = ref<Credentials | null>(null)
  const error = ref<AuthError | null>(null)

  onMounted(async () => {
    authLogger.debug('Checking for saved credentials...')
    try {
      const savedCredentialsResult = await commands.getSavedCredentials()
      authLogger.debug('Got saved credentials result:', savedCredentialsResult)

      if (savedCredentialsResult.status === 'error') {
        authLogger.error('Failed to load saved credentials:', savedCredentialsResult.error)
        error.value = categorizeAuthError(savedCredentialsResult.error)
        authStatus.value = 'error'
        return
      }

      if (savedCredentialsResult.data && savedCredentialsResult.data.token) {
        authLogger.debug('Found saved credentials:', savedCredentialsResult.data)
        credentials.value = savedCredentialsResult.data
        authStore.setCredentials(savedCredentialsResult.data)
        authStatus.value = 'loggedIn'
        error.value = null
        authLogger.debug('Auth store populated:', {
          serverUrl: authStore.serverUrl,
          hasToken:  !!authStore.token,
          userId:    authStore.userId,
          username:  authStore.username,
        })
      } else {
        authLogger.debug('No saved credentials found')
        authStatus.value = 'loggedOut'
        error.value = null
      }
    } catch (err) {
      authLogger.error('Error loading credentials:', err)
      const errorMessage = err instanceof Error ? err.message : 'Unknown authentication error'
      error.value = categorizeAuthError(errorMessage)
      authStatus.value = 'error'
    }
  })

  const login = (loginCredentials: Credentials) => {
    credentials.value = loginCredentials
    authStore.setCredentials(loginCredentials)
    authStatus.value = 'loggedIn'
    error.value = null
  }

  const logout = () => {
    credentials.value = null
    authStore.clearCredentials()
    authStatus.value = 'loggedOut'
    error.value = null
  }

  const clearError = () => {
    error.value = null
    if (authStatus.value === 'error')
      authStatus.value = 'loggedOut'
  }

  return {
    authStatus:  readonly(authStatus),
    credentials: readonly(credentials),
    error:       readonly(error),
    login,
    logout,
    clearError,
  }
}
