<template>
  <div class='p-8 max-w-7xl mx-auto space-y-12'>
    <!-- Header Section -->
    <div class='mb-8'>
      <h1 class='text-4xl font-bold mb-2 text-foreground'>
        Settings
      </h1>
      <p class='text-muted-foreground'>
        Customize your music experience
      </p>
    </div>

    <AppearanceSettings />
    <ServerSettings @logout='$emit("logout")' :credentials='credentials' />
    <LibrarySettings @clear-cache='$emit("clear-cache")' @sync-library='$emit("sync-library")' />
  </div>
</template>

<script setup lang="ts">
  import AppearanceSettings from '@/components/settings/AppearanceSettings.vue'
  import ServerSettings from '@/components/settings/ServerSettings.vue'
  import LibrarySettings from '@/components/settings/LibrarySettings.vue'

  interface Credentials {
    serverUrl: string
    username:  string
    token:     string
    userId:    string
  }

  defineProps<{
    credentials: Credentials | null
  }>()

  defineEmits<{
    (e: 'logout'): void
    (e: 'sync-library'): void
    (e: 'clear-cache'): void
  }>()
</script>
