import { ref, onMounted, readonly } from 'vue'
import type { Credentials } from '@/bindings'
import { commands } from '@/bindings'
import { useAuthStore } from '@/stores'
import { authLogger } from '@/lib/logger'

export type AuthStatus = 'pending' | 'loggedIn' | 'loggedOut' | 'error'

export interface AuthError {
  message: string
  code?:   string
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
        error.value = {
          message: 'Failed to load saved credentials',
          code:    savedCredentialsResult.error,
        }
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
      error.value = {
        message: err instanceof Error ? err.message : 'Unknown authentication error',
        code:    'AUTH_INIT_FAILED',
      }
      authStatus.value = 'error'
    }
  })

  const login = (loginCredentials: Credentials) => {
    credentials.value = loginCredentials
    authStore.setCredentials(loginCredentials)
    authStatus.value = 'loggedIn'
    error.value = null
  }

  const loginError = (message: string, code?: string) => {
    error.value = { message, code }
    authStatus.value = 'error'
  }

  const logout = () => {
    credentials.value = null
    authStore.clearCredentials()
    authStatus.value = 'loggedOut'
    error.value = null
  }

  const clearError = () => {
    error.value = null
    if (authStatus.value === 'error') {
      authStatus.value = 'loggedOut'
    }
  }

  return {
    authStatus:  readonly(authStatus),
    credentials: readonly(credentials),
    error:       readonly(error),
    login,
    loginError,
    logout,
    clearError,
  }
}
