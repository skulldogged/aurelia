<script setup lang="ts">
  import { Loader2 } from 'lucide-vue-next'
  import { onMounted, ref } from 'vue'

  import type { BackendProvider, Credentials } from '../generated'

  import Button from '../components/ui/Button.vue'
  import { Input } from '../components/ui/input'
  import Label from '../components/ui/Label.vue'
  import { useSession } from '../composables/useSession'
  import { ApiError } from '../effect/errors'
  import { runAureliaEffect } from '../effect/runtime'
  import {
    authenticateEffect,
    detectProviderEffect,
    getSavedCredentialsEffect,
    saveCredentialsEffect,
  } from '../effect/services/api'
  import { logger } from '../lib/logger'
  import { setActiveProfileId, upsertProfile } from '../lib/profileStorage'
  const { initializeSession, sessionState } = useSession()

  interface LoginForm {
    password:  string
    serverUrl: string
    username:  string
  }

  const form = ref<LoginForm>({
    password:  '',
    serverUrl: '',
    username:  '',
  })

  const loading = ref(false)
  const detectingProvider = ref(false)
  const detectedProvider = ref<BackendProvider | null>(null)
  const providerSelection = ref<'auto' | BackendProvider>('auto')
  const error = ref('')

  const emit = defineEmits<{
    login: [credentials: Credentials]
  }>()

  const detectProvider = async (): Promise<void> => {
    if (!form.value.serverUrl.trim()) return
    detectingProvider.value = true
    try {
      detectedProvider.value = await runAureliaEffect(detectProviderEffect(form.value.serverUrl))
    } catch (cause) {
      const message = cause instanceof ApiError ? cause.message : String(cause)
      logger.warn(`Provider detection failed: ${message}`)
      detectedProvider.value = null
    } finally {
      detectingProvider.value = false
    }
  }

  const resolveProvider = (): BackendProvider =>
    providerSelection.value === 'auto'
      ? (detectedProvider.value ?? 'jellyfin')
      : providerSelection.value

  const handleLogin = async (): Promise<void> => {
    error.value = ''
    loading.value = true

    try {
      if (providerSelection.value === 'auto' && !detectedProvider.value)
        await detectProvider()

      const credentials = await runAureliaEffect(authenticateEffect({
        deviceId:  sessionState.value.deviceId,
        password:  form.value.password,
        provider:  resolveProvider(),
        serverUrl: form.value.serverUrl,
        username:  form.value.username,
      }))

      try {
        await runAureliaEffect(saveCredentialsEffect(credentials))
        const profile = upsertProfile(credentials)
        setActiveProfileId(profile.id)

        emit('login', credentials)
      } catch (saveError) {
        const saveErrorMessage = saveError instanceof ApiError
          ? saveError.message
          : String(saveError)
        error.value = `Login successful but failed to save credentials: ${saveErrorMessage}`
      }
    } catch (loginError) {
      const loginErrorMessage = loginError instanceof ApiError
        ? loginError.message
        : String(loginError)
      error.value = `Login failed: ${loginErrorMessage}`
    } finally {
      loading.value = false
    }
  }

  onMounted(async () => {
    await initializeSession()
    try {
      const savedCredentials = await runAureliaEffect(getSavedCredentialsEffect())
      if (savedCredentials) {
        const profile = upsertProfile(savedCredentials)
        setActiveProfileId(profile.id)
        form.value.serverUrl = savedCredentials.serverUrl
        form.value.username = savedCredentials.username
        detectedProvider.value = savedCredentials.provider ?? 'jellyfin'
      }
    } catch (savedCredentialsError) {
      logger.error('Failed to get saved credentials:', savedCredentialsError)
    }
  })
</script>

<template>
  <div class='h-full bg-background flex items-center justify-center p-4'>
    <div class='max-w-md w-full p-8'>
      <div class='text-center mb-8'>
        <h1 class='text-3xl font-bold text-foreground mb-2'>
          Aurelia
        </h1>
        <p class='text-muted-foreground'>
          Connect to your media server
        </p>
      </div>

      <form @submit.prevent='handleLogin' class='space-y-6'>
        <div class='grid w-full items-center gap-1.5'>
          <Label for='provider'>Provider</Label>
          <div class='flex gap-2'>
            <select
              id='provider'
              v-model='providerSelection'
              class='w-full border border-input bg-background rounded-md h-10 px-3'
            >
              <option value='auto'>Auto-detect</option>
              <option value='jellyfin'>Jellyfin</option>
              <option value='navidrome'>Navidrome</option>
            </select>
            <Button
              type='button'
              variant='outline'
              :disabled='detectingProvider || !form.serverUrl'
              @click='detectProvider'
            >
              {{ detectingProvider ? 'Detecting...' : 'Detect' }}
            </Button>
          </div>
          <p v-if='detectedProvider' class='text-xs text-muted-foreground'>
            Detected: {{ detectedProvider }}
          </p>
        </div>

        <div class='grid w-full items-center gap-1.5'>
          <Label for='serverUrl'>Server URL</Label>
          <Input
            id='serverUrl'
            v-model='form.serverUrl'
            placeholder='https://your-server.com'
            type='url'
            required
          />
        </div>

        <div class='grid w-full items-center gap-1.5'>
          <Label for='username'>Username</Label>
          <Input
            id='username'
            v-model='form.username'
            placeholder='Enter your username'
            type='text'
            required
          />
        </div>

        <div class='grid w-full items-center gap-1.5'>
          <Label for='password'>Password</Label>
          <Input
            id='password'
            v-model='form.password'
            autocomplete='current-password'
            placeholder='Enter your password'
            type='password'
            required
          />
        </div>

        <Button :disabled='loading' class='w-full' type='submit'>
          <Loader2 v-if='loading' class='mr-2 h-4 w-4 animate-spin' />
          Connect
        </Button>
      </form>

      <div v-if='error' class='mt-4 p-3 bg-destructive/10 border border-destructive/20 rounded-md'>
        <p class='text-destructive-foreground text-sm'>
          {{ error }}
        </p>
      </div>
    </div>
  </div>
</template>
