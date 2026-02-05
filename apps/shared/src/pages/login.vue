<script setup lang="ts">
  import { Loader2 } from 'lucide-vue-next'
  import { onMounted, ref } from 'vue'

  import Button from '../components/ui/Button.vue'
  import { Input } from '../components/ui/input'
  import Label from '../components/ui/Label.vue'
  import { useSession } from '../composables/useSession'
  import { ApiError } from '../effect/errors'
  import { runAureliaEffect } from '../effect/runtime'
  import {
    getSavedCredentialsEffect,
    loginToJellyfinEffect,
    saveCredentialsEffect,
  } from '../effect/services/api'
  import { logger } from '../lib/logger'
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
  const error = ref('')

  const emit = defineEmits<{
    login: [credentials: { serverUrl: string; token: string; userId: string; username: string; }]
  }>()

  const handleLogin = async (): Promise<void> => {
    error.value = ''
    loading.value = true

    try {
      const loginData = await runAureliaEffect(loginToJellyfinEffect(
        form.value.serverUrl,
        form.value.username,
        form.value.password,
        sessionState.value.deviceId,
      ))

      try {
        await runAureliaEffect(saveCredentialsEffect(
          form.value.serverUrl,
          form.value.username,
          loginData.token,
          loginData.userId,
        ))

        // Credentials saved successfully
        emit('login', {
          serverUrl: form.value.serverUrl,
          token:     loginData.token,
          userId:    loginData.userId,
          username:  form.value.username,
        })
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
        form.value.serverUrl = savedCredentials.serverUrl
        form.value.username = savedCredentials.username
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
          Connect to your Jellyfin server
        </p>
      </div>

      <form @submit.prevent='handleLogin' class='space-y-6'>
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
