<script setup lang="ts">
  import type { Credentials } from '@/bindings'

  import SettingsPageDesktop from '@/pages/desktop/SettingsPage.vue'

  const props = defineProps<{
    credentials: Credentials | null
    isClearing:  boolean
    isSyncing:   boolean
  }>()

  const emit = defineEmits<{
    (e: 'logout'): void
    (e: 'sync-library'): void
    (e: 'clear-cache'): void
  }>()
</script>

<template>
  <SettingsPageDesktop
    @clear-cache='emit("clear-cache")'
    @logout='emit("logout")'
    @sync-library='emit("sync-library")'
    v-bind='$attrs'
    :credentials='props.credentials'
    :is-clearing='props.isClearing'
    :is-syncing='props.isSyncing'
  />
</template>