import { ref, onMounted, readonly } from 'vue'
import type { Credentials } from '@/bindings'
import { commands } from '@/bindings'
import { useAuthStore } from '@/stores'

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

  // Check for saved credentials on app start
  onMounted(async () => {
    console.log('🔐 useAuth: Checking for saved credentials...')
    try {
      const savedCredentialsResult = await commands.getSavedCredentials()
      console.log('🔐 useAuth: Got saved credentials result:', savedCredentialsResult)

      if (savedCredentialsResult.status === 'error') {
        console.error('🔐 useAuth: Failed to load saved credentials:', savedCredentialsResult.error)
        error.value = {
          message: 'Failed to load saved credentials',
          code:    savedCredentialsResult.error,
        }
        authStatus.value = 'error'
        return
      }

      if (savedCredentialsResult.data && savedCredentialsResult.data.token) {
        console.log('🔐 useAuth: Found saved credentials:', savedCredentialsResult.data)
        credentials.value = savedCredentialsResult.data
        authStore.setCredentials(savedCredentialsResult.data)
        authStatus.value = 'loggedIn'
        error.value = null
        console.log('🔐 useAuth: Auth store populated:', {
          serverUrl: authStore.serverUrl,
          hasToken:  !!authStore.token,
          userId:    authStore.userId,
          username:  authStore.username,
        })
      } else {
        console.log('🔐 useAuth: No saved credentials found')
        authStatus.value = 'loggedOut'
        error.value = null
      }
    } catch (err) {
      console.error('🔐 useAuth: Error loading credentials:', err)
      error.value = {
        message: err instanceof Error ? err.message : 'Unknown authentication error',
        code:    'AUTH_INIT_FAILED',
      }
      authStatus.value = 'error'
    }
  })

  // Handle login success
  const login = (loginCredentials: Credentials) => {
    credentials.value = loginCredentials
    authStore.setCredentials(loginCredentials)
    authStatus.value = 'loggedIn'
    error.value = null
  }

  // Handle login error
  const loginError = (message: string, code?: string) => {
    error.value = { message, code }
    authStatus.value = 'error'
  }

  // Handle logout
  const logout = () => {
    credentials.value = null
    authStore.clearCredentials()
    authStatus.value = 'loggedOut'
    error.value = null
  }

  // Clear error state
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
