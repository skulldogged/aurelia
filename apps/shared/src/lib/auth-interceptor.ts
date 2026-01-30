import type { Auth } from '../composables/useAuth'

let authLogout: Auth['logout'] | null = null

export const getAuthLogout = (): Auth['logout'] | null => authLogout

export const setAuthLogout = (logoutFunc: Auth['logout']): void => {
  authLogout = logoutFunc
}
