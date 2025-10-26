import type { Auth } from '@/composables/useAuth'

let authLogout: Auth['logout'] | null = null

export function setAuthLogout(logoutFunc: Auth['logout']) {
  authLogout = logoutFunc
}

export function getAuthLogout() {
  return authLogout
}
