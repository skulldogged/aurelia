<script setup lang="ts">
  import AppearanceSettings from '@/components/settings/AppearanceSettings.vue'
  import LibrarySettings from '@/components/settings/LibrarySettings.vue'
  import ServerSettings from '@/components/settings/ServerSettings.vue'

  interface Credentials {
    serverUrl: string
    token:     string
    userId:    string
    username:  string
  }

  defineProps<{
    credentials: Credentials | null
    isClearing:  boolean
    isSyncing:   boolean
  }>()

  defineEmits<{
    (e: 'logout'): void
    (e: 'sync-library'): void
    (e: 'clear-cache'): void
  }>()
</script>

<template>
  <div class='p-4 max-w-7xl mx-auto space-y-12'>
    <div class='mb-8'>
      <h1 class='text-4xl font-bold mb-2 text-foreground'>
        Settings
      </h1>
    </div>

    <AppearanceSettings />
    <ServerSettings @logout='$emit("logout")' :credentials='credentials' />
    <LibrarySettings
      @clear-cache='$emit("clear-cache")'
      @sync-library='$emit("sync-library")'
      :is-clearing='isClearing'
      :is-syncing='isSyncing'
    />
  </div>
</template>
