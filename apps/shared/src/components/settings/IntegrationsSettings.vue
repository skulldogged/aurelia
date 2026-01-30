<script setup lang="ts">
  import { ExternalLink, XCircle } from 'lucide-vue-next'
  import { computed, onBeforeUnmount, ref } from 'vue'

  import { getApiClient } from '../../index'
  import Button from '../ui/Button.vue'
  import { Input } from '../ui/input'
  import Label from '../ui/Label.vue'
  import Switch from '../ui/Switch.vue'
  import { useLastFm } from '../../composables/useLastFm'
  import { useListenBrainz } from '../../composables/useListenBrainz'
  import { logger } from '../../lib/logger'
  import { isTauri } from '../../lib/platform'
  import { useLastFmStore, useListenBrainzStore } from '../../stores'

  // Type for unlisten function
  type UnlistenFn = () => void

  const lastfmStore = useLastFmStore()
  const listenbrainzStore = useListenBrainzStore()
  const { authenticate: authenticateLastFm, clearSession: clearLastFmSession } = useLastFm()
  const { clearSession: clearListenBrainzSession, validateToken } = useListenBrainz()

  // Last.fm state
  const apiKey = ref('')
  const apiSecret = ref('')
  const isLastFmAuthenticating = ref(false)
  const isWaitingForCallback = ref(false)
  const lastfmError = ref<null | string>(null)
  let unlisten: null | UnlistenFn = null

  // ListenBrainz state
  const userToken = ref('')
  const isValidating = ref(false)
  const listenbrainzError = ref<null | string>(null)

  const isLastFmAuthenticated = computed(() => lastfmStore.isAuthenticated())
  const isListenBrainzAuthenticated = computed(() => listenbrainzStore.isAuthenticated())
  const listenbrainzUsername = computed(() => listenbrainzStore.credentials?.username ?? 'Unknown')

  // Helper to open URL cross-platform
  const openUrlCrossPlatform = async (url: string): Promise<void> => {
    if (isTauri()) {
      const { openUrl } = await import('@tauri-apps/plugin-opener')
      await openUrl(url)
    } else {
      window.open(url, '_blank', 'noopener,noreferrer')
    }
  }

  // Last.fm handlers
  const handleLastFmAuthenticate = async (): Promise<void> => {
    if (!apiKey.value || !apiSecret.value) {
      lastfmError.value = 'Please fill in API key and secret'
      return
    }

    // Last.fm OAuth callback only works on desktop (requires local server)
    if (!isTauri()) {
      lastfmError.value = 'Last.fm authentication is only available in the desktop app'
      return
    }

    lastfmError.value = null
    isWaitingForCallback.value = true

    try {
      const { listen } = await import('@tauri-apps/api/event')
      unlisten = await listen<string>('lastfm://token-received', async event => {
        logger.info('Received token from callback event')
        isWaitingForCallback.value = false
        isLastFmAuthenticating.value = true

        try {
          await authenticateLastFm(apiKey.value, apiSecret.value, event.payload)
          apiKey.value = ''
          apiSecret.value = ''
        } catch (err) {
          lastfmError.value = err instanceof Error ? err.message : 'Authentication failed'
        } finally {
          isLastFmAuthenticating.value = false
        }
      })

      const styles = getComputedStyle(document.documentElement)
      const primaryColor = styles.getPropertyValue('--accent').trim() || '#667eea'
      const backgroundColor = styles.getPropertyValue('--background').trim() || '#1a1b26'
      const textColor = styles.getPropertyValue('--foreground').trim() || '#cdd6f4'

      const result = await getApiClient().lastfmStartAuthServer(primaryColor, backgroundColor, textColor)
      if (result.status === 'error') {
        lastfmError.value = result.error
        isWaitingForCallback.value = false
        return
      }

      logger.info('Started Last.fm callback server')

      const callbackUrl = encodeURIComponent('http://127.0.0.1:3000')
      const url = `https://www.last.fm/api/auth/?api_key=${apiKey.value}&cb=${callbackUrl}`
      await openUrlCrossPlatform(url)

      setTimeout(() => {
        if (isWaitingForCallback.value) {
          isWaitingForCallback.value = false
          lastfmError.value = 'Authorization timeout. Please try again.'
          if (unlisten) {
            unlisten()
            unlisten = null
          }
        }
      }, 300000)
    } catch (err) {
      isWaitingForCallback.value = false
      lastfmError.value = err instanceof Error ? err.message : 'Failed to start authentication'
    }
  }

  const handleLastFmDisconnect = async (): Promise<void> => {
    try {
      await clearLastFmSession()
      lastfmError.value = null
    } catch (err) {
      lastfmError.value = err instanceof Error ? err.message : 'Failed to disconnect'
    }
  }

  const handleToggleLastFmScrobbling = (checked: boolean): void => {
    lastfmStore.setScrobblingEnabled(checked)
  }

  // ListenBrainz handlers
  const handleListenBrainzConnect = async (): Promise<void> => {
    if (!userToken.value.trim()) {
      listenbrainzError.value = 'Please enter your user token'
      return
    }

    listenbrainzError.value = null
    isValidating.value = true

    try {
      await validateToken(userToken.value.trim())
      userToken.value = ''
      logger.info('Successfully connected')
    } catch (err) {
      listenbrainzError.value = err instanceof Error ? err.message : 'Failed to validate token'
      logger.error('Connection failed:', err)
    } finally {
      isValidating.value = false
    }
  }

  const handleListenBrainzDisconnect = async (): Promise<void> => {
    try {
      await clearListenBrainzSession()
      listenbrainzError.value = null
    } catch (err) {
      listenbrainzError.value = err instanceof Error ? err.message : 'Failed to disconnect'
      logger.error('Disconnect failed:', err)
    }
  }

  const handleToggleListenBrainzScrobbling = (checked: boolean): void => {
    listenbrainzStore.setScrobblingEnabled(checked)
  }

  const openTokenPage = async (): Promise<void> => {
    await openUrlCrossPlatform('https://listenbrainz.org/settings/')
  }

  onBeforeUnmount(() => {
    if (unlisten) {
      unlisten()
      unlisten = null
    }
  })
</script>

<template>
  <div class='space-y-8'>
    <!-- Content -->
    <div class='grid grid-cols-1 lg:grid-cols-2 gap-8'>
      <!-- Last.fm Section -->
      <div class='space-y-6'>
        <h3 class='text-lg font-semibold'>
          Last.fm
        </h3>

        <div v-if='!isLastFmAuthenticated' class='space-y-4'>
          <p class='text-sm text-muted-foreground'>
            Connect your Last.fm account to automatically scrobble tracks as you listen.
          </p>
          <div class='space-y-2'>
            <Label for='api-key'>API Key</Label>
            <Input
              id='api-key'
              v-model='apiKey'
              class='bg-background/40 border-border/20'
              placeholder='Enter your Last.fm API key'
              type='text'
            />
          </div>

          <div class='space-y-2'>
            <Label for='api-secret'>API Secret</Label>
            <Input
              id='api-secret'
              v-model='apiSecret'
              class='bg-background/40 border-border/20'
              placeholder='Enter your Last.fm API secret'
              type='password'
            />
          </div>

          <Button
            @click='handleLastFmAuthenticate'
            :disabled='isLastFmAuthenticating || isWaitingForCallback || !apiKey || !apiSecret'
            class='w-full'
          >
            <template v-if='isWaitingForCallback'>
              Waiting for authorization...
            </template>
            <template v-else-if='isLastFmAuthenticating'>
              Authenticating...
            </template>
            <template v-else>
              <ExternalLink class='size-4 mr-2' />
              Connect to Last.fm
            </template>
          </Button>

          <div
            v-if='lastfmError'
            class='text-sm text-destructive p-3 bg-destructive/10 rounded-lg border border-destructive/20'
          >
            {{ lastfmError }}
          </div>

          <div class='p-4 bg-background/40 rounded-lg border border-border/20'>
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
          <div
            class='
              flex items-center space-x-3 p-4 bg-background/40 rounded-lg
              border border-border/20
            '
          >
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
              <p class='font-medium'>
                Connected to Last.fm
              </p>
              <p class='text-sm text-muted-foreground'>
                Signed in as {{ lastfmStore.credentials?.username }}
              </p>
            </div>
          </div>

          <div
            class='
              flex items-center justify-between p-4 bg-background/40 rounded-lg
              border border-border/20 hover:border-border/40 transition-colors
            '
          >
            <Label class='text-sm cursor-pointer' for='lastfm-scrobbling-switch'>
              Enable scrobbling
            </Label>
            <Switch
              @update:checked='handleToggleLastFmScrobbling'
              id='lastfm-scrobbling-switch'
              :checked='lastfmStore.isScrobblingEnabled'
            />
          </div>

          <Button
            @click='handleLastFmDisconnect'
            class='w-full'
            variant='destructive'
          >
            <XCircle class='size-4 mr-2' />
            Disconnect from Last.fm
          </Button>
        </div>
      </div>

      <!-- ListenBrainz Section -->
      <div class='space-y-6'>
        <h3 class='text-lg font-semibold'>
          ListenBrainz
        </h3>

        <div v-if='!isListenBrainzAuthenticated' class='space-y-4'>
          <p class='text-sm text-muted-foreground'>
            ListenBrainz is a free and open-source alternative to Last.fm for tracking your listening history.
          </p>

          <div class='space-y-4'>
            <div class='space-y-2'>
              <div class='flex items-center justify-between'>
                <Label for='listenbrainz-token'>User Token</Label>
                <Button @click='openTokenPage' size='sm' variant='ghost'>
                  <ExternalLink class='mr-2 size-4' />
                  Get Token
                </Button>
              </div>
              <Input
                @keyup.enter='handleListenBrainzConnect'
                id='listenbrainz-token'
                v-model='userToken'
                :disabled='isValidating'
                class='bg-background/40 border-border/20'
                placeholder='Enter your ListenBrainz user token'
                type='password'
              />
              <p class='text-xs text-muted-foreground'>
                You can find your user token in your ListenBrainz settings
              </p>
            </div>

            <Button
              @click='handleListenBrainzConnect'
              :disabled='!userToken.trim() || isValidating'
              class='w-full'
            >
              <template v-if='isValidating'>
                Validating...
              </template>
              <template v-else>
                <ExternalLink class='size-4 mr-2' />
                Connect to ListenBrainz
              </template>
            </Button>

            <div
              v-if='listenbrainzError'
              class='text-sm text-destructive p-3 bg-destructive/10 rounded-lg border border-destructive/20'
            >
              {{ listenbrainzError }}
            </div>
          </div>
        </div>

        <div v-else class='space-y-4'>
          <div
            class='
              flex items-center space-x-3 p-4 bg-background/40 rounded-lg
              border border-border/20
            '
          >
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
              <p class='font-medium'>
                Connected to ListenBrainz
              </p>
              <p class='text-sm text-muted-foreground'>
                Signed in as {{ listenbrainzUsername }}
              </p>
            </div>
          </div>

          <div
            class='
              flex items-center justify-between p-4 bg-background/40 rounded-lg
              border border-border/20 hover:border-border/40 transition-colors
            '
          >
            <Label class='text-sm cursor-pointer' for='listenbrainz-scrobbling-switch'>
              Enable scrobbling
            </Label>
            <Switch
              @update:checked='handleToggleListenBrainzScrobbling'
              id='listenbrainz-scrobbling-switch'
              :checked='listenbrainzStore.isScrobblingEnabled'
            />
          </div>

          <Button
            @click='handleListenBrainzDisconnect'
            class='w-full'
            variant='destructive'
          >
            <XCircle class='size-4 mr-2' />
            Disconnect from ListenBrainz
          </Button>
        </div>
      </div>
    </div>
  </div>
</template>
