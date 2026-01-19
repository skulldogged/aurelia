import { defineStore } from 'pinia'
import { readonly, ref } from 'vue'

import type { Credentials } from '@/lib/api/bindings'

export const useAuthStore = defineStore('auth', () => {
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
    },

    isAuthenticated: (): boolean => !!token.value && !!serverUrl.value && !!userId.value,

    setCredentials: (credentials: Credentials): void => {
      serverUrl.value = credentials.serverUrl
      token.value = credentials.token
      userId.value = credentials.userId
      username.value = credentials.username
    },

    // eslint-disable-next-line perfectionist/sort-objects
    serverUrl: readonly(serverUrl),
    token:     readonly(token),
    userId:    readonly(userId),
    username:  readonly(username),
  }
})
