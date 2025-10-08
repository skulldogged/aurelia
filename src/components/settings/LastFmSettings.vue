<script setup lang="ts">
  import { invoke } from '@tauri-apps/api/core'
  import { listen, type UnlistenFn } from '@tauri-apps/api/event'
  import { openUrl } from '@tauri-apps/plugin-opener'
  import { ExternalLink, XCircle } from 'lucide-vue-next'
  import { siLastdotfm } from 'simple-icons'
  import { computed, onBeforeUnmount, ref } from 'vue'

  import { Button } from '@/components/ui/button'
  import { Input } from '@/components/ui/input'
  import { Label } from '@/components/ui/label'
  import { useLastFm } from '@/composables/useLastFm'
  import { lastfmLogger } from '@/lib/logger'
  import { useLastFmStore } from '@/stores'

  const lastfmStore = useLastFmStore()
  const { authenticate, clearSession } = useLastFm()

  const apiKey = ref('')
  const apiSecret = ref('')
  const isAuthenticating = ref(false)
  const isWaitingForCallback = ref(false)
  const error = ref<null | string>(null)
  let unlisten: null | UnlistenFn = null

  const isAuthenticated = computed(() => lastfmStore.isAuthenticated())

  const lastfmIcon = computed(() => {
    const accentColor = getComputedStyle(document.documentElement).getPropertyValue('--accent').trim()
    // Replace the default black color with accent color
    const svg = siLastdotfm.svg.replace(/<svg/, `<svg fill="${accentColor}"`)
    return `data:image/svg+xml,${encodeURIComponent(svg)}`
  })

  const handleAuthenticate = async (): Promise<void> => {
    if (!apiKey.value || !apiSecret.value) {
      error.value = 'Please fill in API key and secret'
      return
    }

    error.value = null
    isWaitingForCallback.value = true

    try {
      // Listen for token event
      unlisten = await listen<string>('lastfm://token-received', async event => {
        lastfmLogger.info('Received token from callback event')
        isWaitingForCallback.value = false
        isAuthenticating.value = true

        try {
          // Authenticate with the token
          await authenticate(apiKey.value, apiSecret.value, event.payload)

          // Clear form on success
          apiKey.value = ''
          apiSecret.value = ''
        } catch (err) {
          error.value = err instanceof Error ? err.message : 'Authentication failed'
        } finally {
          isAuthenticating.value = false
        }
      })

      // Get theme colors from CSS variables
      const styles = getComputedStyle(document.documentElement)
      const primaryColor = styles.getPropertyValue('--accent').trim() || '#667eea'
      const backgroundColor = styles.getPropertyValue('--background').trim() || '#1a1b26'
      const textColor = styles.getPropertyValue('--foreground').trim() || '#cdd6f4'

      // Start the callback server with theme colors
      await invoke('lastfm_start_auth_server', {
        backgroundColor,
        primaryColor,
        textColor,
      })
      lastfmLogger.info('Started Last.fm callback server')

      // Open Last.fm authorization page
      const callbackUrl = encodeURIComponent('http://127.0.0.1:3000')
      const url = `https://www.last.fm/api/auth/?api_key=${apiKey.value}&cb=${callbackUrl}`
      await openUrl(url)

      // Set a timeout in case user closes browser without authorizing
      setTimeout(() => {
        if (isWaitingForCallback.value) {
          isWaitingForCallback.value = false
          error.value = 'Authorization timeout. Please try again.'
          if (unlisten) {
            unlisten()
            unlisten = null
          }
        }
      }, 300000) // 5 minutes
    } catch (err) {
      isWaitingForCallback.value = false
      error.value = err instanceof Error ? err.message : 'Failed to start authentication'
    }
  }

  const handleDisconnect = async (): Promise<void> => {
    try {
      await clearSession()
      error.value = null
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to disconnect'
    }
  }

  const handleToggleScrobbling = (checked: boolean): void => {
    lastfmStore.setScrobblingEnabled(checked)
  }

  // Cleanup listener on unmount
  onBeforeUnmount(() => {
    if (unlisten) {
      unlisten()
      unlisten = null
    }
  })
</script>

<template>
  <section class='space-y-6'>
    <div class='flex items-center space-x-3'>
      <div class='p-2 bg-accent/10 rounded-lg'>
        <img
          :src='lastfmIcon'
          alt='Last.fm'
          class='size-5'
        >
      </div>
      <h2 class='text-2xl font-semibold'>
        Last.fm Integration
      </h2>
    </div>

    <div class='bg-card/50 backdrop-blur-sm border border-border/50 rounded-xl p-6 shadow-lg'>
      <div v-if='!isAuthenticated' class='space-y-4'>
        <div class='flex items-center space-x-3 mb-4'>
          <div class='p-2 bg-primary/10 rounded-lg'>
            <img
              :src='lastfmIcon'
              alt='Last.fm'
              class='size-5'
            >
          </div>
          <h3 class='text-lg font-medium'>
            Connect to Last.fm
          </h3>
        </div>

        <p class='text-sm text-muted-foreground mb-4'>
          Scrobble your music to Last.fm to track your listening history and discover new music.
        </p>

        <div class='space-y-4'>
          <div class='space-y-2'>
            <Label for='api-key'>API Key</Label>
            <Input
              id='api-key'
              v-model='apiKey'
              placeholder='Enter your Last.fm API key'
              type='text'
            />
          </div>

          <div class='space-y-2'>
            <Label for='api-secret'>API Secret</Label>
            <Input
              id='api-secret'
              v-model='apiSecret'
              placeholder='Enter your Last.fm API secret'
              type='password'
            />
          </div>

          <div class='bg-background/50 border border-border/30 rounded-lg p-4 space-y-3'>
            <p class='text-sm font-medium text-foreground'>
              How it works:
            </p>
            <ol class='text-sm text-muted-foreground list-decimal list-inside space-y-1'>
              <li>Enter your API credentials above</li>
              <li>Click "Connect to Last.fm" below</li>
              <li>Authorize the app in your browser</li>
              <li>You'll be automatically connected!</li>
            </ol>
          </div>

          <Button
            @click='handleAuthenticate'
            :disabled='isAuthenticating || isWaitingForCallback || !apiKey || !apiSecret'
            class='w-full'
          >
            <template v-if='isWaitingForCallback'>
              Waiting for authorization...
            </template>
            <template v-else-if='isAuthenticating'>
              Authenticating...
            </template>
            <template v-else>
              <ExternalLink class='size-4 mr-2' />
              Connect to Last.fm
            </template>
          </Button>

          <div
            v-if='error'
            class='text-sm text-destructive p-3 bg-destructive/10 rounded-lg
                   border border-destructive/20'
          >
            {{ error }}
          </div>
        </div>

        <div class='mt-4 p-4 bg-background/50 rounded-lg border border-border/30'>
          <p class='text-xs text-muted-foreground'>
            Don't have API credentials?
            <a
              class='text-accent hover:underline'
              href='https://www.last.fm/api/account/create'
              target='_blank'
            >
              Create an API account on Last.fm
            </a>
          </p>
        </div>
      </div>

      <div v-else class='space-y-4'>
        <div class='flex items-center space-x-3 mb-4'>
          <div class='p-2 bg-success/10 rounded-lg'>
            <svg
              class='size-5 text-success'
              fill='none'
              stroke='currentColor'
              stroke-width='2'
              viewBox='0 0 24 24'
              xmlns='http://www.w3.org/2000/svg'
            >
              <path d='M5 13l4 4L19 7' stroke-linecap='round' stroke-linejoin='round' />
            </svg>
          </div>
          <div>
            <h3 class='text-lg font-medium'>
              Connected to Last.fm
            </h3>
            <p class='text-sm text-muted-foreground'>
              Signed in as {{ lastfmStore.credentials?.username }}
            </p>
          </div>
        </div>

        <div class='flex items-center space-x-3 p-3 bg-background/50 rounded-lg border border-border/30'>
          <div class='relative flex items-center justify-center'>
            <input
              @change='handleToggleScrobbling(($event.target as HTMLInputElement).checked)'
              id='scrobbling-checkbox'
              :checked='lastfmStore.isScrobblingEnabled'
              class='peer h-5 w-5 shrink-0 appearance-none rounded-sm border border-input
                     ring-offset-background focus-visible:outline-none focus-visible:ring-2
                     focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed
                     disabled:opacity-50 checked:bg-accent checked:text-accent-foreground checked:border-accent'
              type='checkbox'
            >
            <div
              class='absolute inset-0 flex items-center justify-center text-accent-foreground
                       opacity-0 peer-checked:opacity-100 pointer-events-none'
            >
              <svg
                class='h-3 w-3'
                fill='none'
                viewBox='0 0 12 12'
                xmlns='http://www.w3.org/2000/svg'
              >
                <path
                  d='M10.5 3L4.5 9L2 6.5'
                  stroke='currentColor'
                  stroke-linecap='round'
                  stroke-linejoin='round'
                  stroke-width='1.5'
                />
              </svg>
            </div>
          </div>
          <div class='flex items-center space-x-2 flex-1'>
            <Label class='text-sm font-medium cursor-pointer' for='scrobbling-checkbox'>
              Enable scrobbling
            </Label>
          </div>
        </div>

        <Button
          @click='handleDisconnect'
          class='w-full'
          variant='destructive'
        >
          <XCircle class='size-4 mr-2' />
          Disconnect from Last.fm
        </Button>
      </div>
    </div>
  </section>
</template>
