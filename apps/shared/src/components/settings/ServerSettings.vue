<script setup lang="ts">
  import { onMounted, ref } from 'vue'

  import {
    Link,
    LogOut,
    Server,
    User,
  } from 'lucide-vue-next'

  import { getApiClient } from '../../api/apiClient'
  import Button from '../ui/Button.vue'

  interface Credentials {
    serverUrl: string
    token:     string
    userId:    string
    username:  string
  }

  defineProps<{
    credentials: Credentials | null
  }>()

  defineEmits<{
    (e: 'logout'): void
  }>()

  const aureliaServerUrl = ref('')
  let saveTimeout: ReturnType<typeof setTimeout> | null = null

  onMounted(async () => {
    try {
      const result = await getApiClient().getSetting('aurelia_server_url')
      if (result.ok && result.data) {
        aureliaServerUrl.value = result.data
      }
    } catch {
      // Setting not found, leave empty
    }
  })

  function onAureliaUrlInput(event: Event) {
    const value = (event.target as HTMLInputElement).value
    aureliaServerUrl.value = value

    // Debounce save
    if (saveTimeout) clearTimeout(saveTimeout)
    saveTimeout = setTimeout(async () => {
      try {
        if (value.trim()) {
          await getApiClient().saveSetting('aurelia_server_url', value.trim())
        } else {
          await getApiClient().deleteSetting('aurelia_server_url')
        }
      } catch {
        // Ignore save errors
      }
    }, 500)
  }
</script>

<template>
  <div class='space-y-8'>
    <!-- Connection Status -->
    <div class='space-y-6'>
      <!-- Connection Status -->
      <div
        class='
          flex items-center space-x-3 p-4 bg-background/40 rounded-lg
          border border-border/20
        '
      >
        <div :class='credentials ? "bg-green-500" : "bg-red-500"' class='size-3 rounded-full' />
        <div>
          <p class='font-medium'>
            {{ credentials ? 'Connected' : 'Not Connected' }}
          </p>
          <p class='text-sm text-muted-foreground'>
            {{ credentials ? 'Server connection active' : 'No server connection' }}
          </p>
        </div>
      </div>

      <!-- Server Info -->
      <div class='grid md:grid-cols-2 gap-6'>
        <div class='space-y-2'>
          <label class='text-sm font-medium text-muted-foreground flex items-center space-x-2'>
            <Link class='size-4' />
            <span>Jellyfin Server</span>
          </label>
          <p
            class='
              text-sm font-mono bg-background/40 p-3 rounded-lg
              border border-border/20
            '
          >
            {{ credentials?.serverUrl || 'Not connected' }}
          </p>
        </div>
        <div class='space-y-2'>
          <label class='text-sm font-medium text-muted-foreground flex items-center space-x-2'>
            <User class='size-4' />
            <span>Username</span>
          </label>
          <p
            class='
              text-sm bg-background/40 p-3 rounded-lg
              border border-border/20
            '
          >
            {{ credentials?.username || 'Not connected' }}
          </p>
        </div>
      </div>

      <!-- Aurelia Server URL -->
      <div class='space-y-2'>
        <label class='text-sm font-medium text-muted-foreground flex items-center space-x-2'>
          <Server class='size-4' />
          <span>Aurelia Server</span>
        </label>
        <input
          :value='aureliaServerUrl'
          @input='onAureliaUrlInput'
          type='url'
          placeholder='https://aurelia.example.com'
          class='
            w-full text-sm font-mono bg-background/40 p-3 rounded-lg
            border border-border/20 outline-none
            focus:border-primary/50 transition-colors
          '
        >
        <p class='text-xs text-muted-foreground'>
          URL of your Aurelia web server for synced lyrics from sidecar files. Leave empty if not using one.
        </p>
      </div>

      <!-- Actions -->
      <div class='flex justify-end pt-2 border-t border-border/20'>
        <Button
          @click='$emit("logout")'
          :disabled='!credentials'
          class='px-6'
          variant='destructive'
        >
          <LogOut class='size-4 mr-2' />
          Logout
        </Button>
      </div>
    </div>
  </div>
</template>
