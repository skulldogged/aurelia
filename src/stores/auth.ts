import { defineStore } from 'pinia'
import { ref, readonly } from 'vue'
import type { Credentials } from '@/bindings'

export const useAuthStore = defineStore('auth', () => {
  const serverUrl = ref<string>('')
  const token = ref<string>('')
  const userId = ref<string>('')
  const username = ref<string>('')

  const setCredentials = (credentials: Credentials) => {
    serverUrl.value = credentials.serverUrl
    token.value = credentials.token
    userId.value = credentials.userId
    username.value = credentials.username
  }

  const clearCredentials = () => {
    serverUrl.value = ''
    token.value = ''
    userId.value = ''
    username.value = ''
  }

  const isAuthenticated = () => {
    return !!token.value && !!serverUrl.value && !!userId.value
  }

  return {
    serverUrl: readonly(serverUrl),
    token:     readonly(token),
    userId:    readonly(userId),
    username:  readonly(username),

    setCredentials,
    clearCredentials,
    isAuthenticated,
  }
})
