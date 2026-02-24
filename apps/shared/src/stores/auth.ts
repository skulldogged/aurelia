import { defineStore } from 'pinia'
import { readonly, ref } from 'vue'

import type { BackendProvider, Credentials, ProviderCapabilities } from '../generated'

export const useAuthStore = defineStore('auth', () => {
  const provider = ref<BackendProvider>('jellyfin')
  const providerCapabilities = ref<ProviderCapabilities | null>(null)
  const serverUrl = ref<string>('')
  const token = ref<string>('')
  const userId = ref<string>('')
  const username = ref<string>('')

  return {
    clearCredentials: (): void => {
      serverUrl.value = ''
      token.value = ''
      userId.value = ''
      username.value = ''
      provider.value = 'jellyfin'
      providerCapabilities.value = null
    },

    isAuthenticated: (): boolean => !!token.value && !!serverUrl.value && !!userId.value,

    setCredentials: (credentials: Credentials): void => {
      provider.value = credentials.provider ?? 'jellyfin'
      serverUrl.value = credentials.serverUrl
      token.value = credentials.token
      userId.value = credentials.userId
      username.value = credentials.username
    },

    setProviderCapabilities: (capabilities: ProviderCapabilities | null): void => {
      providerCapabilities.value = capabilities
    },

    // eslint-disable-next-line perfectionist/sort-objects
    provider: readonly(provider),
    providerCapabilities: readonly(providerCapabilities),
    serverUrl: readonly(serverUrl),
    token:     readonly(token),
    userId:    readonly(userId),
    username:  readonly(username),
  }
})
