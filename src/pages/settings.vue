<script setup lang="ts">
  import type { Credentials } from '@/bindings'

  import SettingsView from '@/views/SettingsView.vue'

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
  <SettingsView
    @clear-cache='() => emit("clear-cache")'
    @logout='() => emit("logout")'
    @sync-library='() => emit("sync-library")'
    v-bind='$attrs'
    :credentials='props.credentials'
    :is-clearing='props.isClearing'
    :is-syncing='props.isSyncing'
  />
</template>
