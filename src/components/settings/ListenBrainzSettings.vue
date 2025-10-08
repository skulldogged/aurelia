<script setup lang="ts">
  import { ExternalLink, XCircle } from 'lucide-vue-next'
  import { siMusicbrainz } from 'simple-icons'
  import { computed, ref } from 'vue'

  import { Button } from '@/components/ui/button'
  import { Input } from '@/components/ui/input'
  import { Label } from '@/components/ui/label'
  import { useListenBrainz } from '@/composables/useListenBrainz'
  import { listenbrainzLogger } from '@/lib/logger'
  import { useListenBrainzStore } from '@/stores'

  const listenbrainzStore = useListenBrainzStore()
  const { clearSession, validateToken } = useListenBrainz()

  const userToken = ref('')
  const isValidating = ref(false)
  const error = ref<null | string>(null)

  const isAuthenticated = computed(() => listenbrainzStore.isAuthenticated())
  const username = computed(() => listenbrainzStore.credentials?.username ?? 'Unknown')

  const musicbrainzIcon = computed(() => {
    const accentColor = getComputedStyle(document.documentElement).getPropertyValue('--accent').trim()
    // Replace the default black color with accent color
    const svg = siMusicbrainz.svg.replace(/<svg/, `<svg fill="${accentColor}"`)
    return `data:image/svg+xml,${encodeURIComponent(svg)}`
  })

  const handleConnect = async (): Promise<void> => {
    if (!userToken.value.trim()) {
      error.value = 'Please enter your user token'
      return
    }

    error.value = null
    isValidating.value = true

    try {
      await validateToken(userToken.value.trim())
      userToken.value = ''
      listenbrainzLogger.info('Successfully connected')
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to validate token'
      listenbrainzLogger.error('Connection failed:', err)
    } finally {
      isValidating.value = false
    }
  }

  const handleDisconnect = async (): Promise<void> => {
    try {
      await clearSession()
      error.value = null
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to disconnect'
      listenbrainzLogger.error('Disconnect failed:', err)
    }
  }

  const handleToggleScrobbling = (checked: boolean): void => {
    listenbrainzStore.setScrobblingEnabled(checked)
  }

  const openTokenPage = (): void => {
    window.open('https://listenbrainz.org/settings/', '_blank')
  }
</script>

<template>
  <section class='space-y-6'>
    <div class='flex items-center space-x-3'>
      <div class='p-2 bg-accent/10 rounded-lg'>
        <img
          :src='musicbrainzIcon'
          alt='MusicBrainz'
          class='size-5'
        >
      </div>
      <h2 class='text-2xl font-semibold'>
        ListenBrainz Integration
      </h2>
    </div>

    <div class='bg-card/50 backdrop-blur-sm border border-border/50 rounded-xl p-6 shadow-lg'>
      <div v-if='!isAuthenticated' class='space-y-4'>
        <div class='flex items-center space-x-3 mb-4'>
          <div class='p-2 bg-primary/10 rounded-lg'>
            <img
              :src='musicbrainzIcon'
              alt='MusicBrainz'
              class='size-5'
            >
          </div>
          <h3 class='text-lg font-medium'>
            Connect to ListenBrainz
          </h3>
        </div>

        <p class='text-sm text-muted-foreground mb-4'>
          Scrobble your music to ListenBrainz to track your listening history.
          ListenBrainz is a free and open-source alternative to Last.fm.
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
              @keyup.enter='handleConnect'
              id='listenbrainz-token'
              v-model='userToken'
              :disabled='isValidating'
              placeholder='Enter your ListenBrainz user token'
              type='password'
            />
            <p class='text-xs text-muted-foreground'>
              You can find your user token in your ListenBrainz settings
            </p>
          </div>

          <Button
            @click='handleConnect'
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
            v-if='error'
            class='text-sm text-destructive p-3 bg-destructive/10 rounded-lg
                   border border-destructive/20'
          >
            {{ error }}
          </div>
        </div>

        <div class='mt-4 p-4 bg-background/50 rounded-lg border border-border/30'>
          <p class='text-sm font-medium text-foreground mb-2'>
            About ListenBrainz
          </p>
          <p class='text-sm text-muted-foreground'>
            ListenBrainz is an open-source music tracking service that records your listening history
            and provides statistics about your music taste. It's completely free and open source.
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
              Connected to ListenBrainz
            </h3>
            <p class='text-sm text-muted-foreground'>
              Signed in as {{ username }}
            </p>
          </div>
        </div>

        <div class='flex items-center space-x-3 p-3 bg-background/50 rounded-lg border border-border/30'>
          <div class='relative flex items-center justify-center'>
            <input
              @change='handleToggleScrobbling(($event.target as HTMLInputElement).checked)'
              id='listenbrainz-scrobbling-checkbox'
              :checked='listenbrainzStore.isScrobblingEnabled'
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
            <Label class='text-sm font-medium cursor-pointer' for='listenbrainz-scrobbling-checkbox'>
              Enable scrobbling
            </Label>
          </div>
        </div>

        <Button @click='handleDisconnect' class='w-full' variant='destructive'>
          <XCircle class='size-4 mr-2' />
          Disconnect from ListenBrainz
        </Button>
      </div>
    </div>
  </section>
</template>
