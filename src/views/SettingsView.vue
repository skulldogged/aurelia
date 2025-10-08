<script setup lang="ts">
  import AppearanceSettings from '@/components/settings/AppearanceSettings.vue'
  import LastFmSettings from '@/components/settings/LastFmSettings.vue'
  import LibrarySettings from '@/components/settings/LibrarySettings.vue'
  import ListenBrainzSettings from '@/components/settings/ListenBrainzSettings.vue'
  import ServerSettings from '@/components/settings/ServerSettings.vue'
  import SystemTraySettings from '@/components/settings/SystemTraySettings.vue'
  import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'

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
  <div class='mx-auto max-w-7xl p-4'>
    <div class='mb-8'>
      <h1 class='mb-2 text-4xl font-bold text-foreground'>
        Settings
      </h1>
    </div>

    <Tabs default-value='appearance'>
      <TabsList class='mb-6'>
        <TabsTrigger value='appearance'>
          Appearance
        </TabsTrigger>
        <TabsTrigger value='scrobbling'>
          Scrobbling
        </TabsTrigger>
        <TabsTrigger value='server'>
          Server
        </TabsTrigger>
        <TabsTrigger value='library'>
          Library
        </TabsTrigger>
      </TabsList>

      <TabsContent class='space-y-6' value='appearance'>
        <AppearanceSettings />
        <SystemTraySettings />
      </TabsContent>

      <TabsContent class='space-y-6' value='scrobbling'>
        <div class='grid grid-cols-1 lg:grid-cols-2 gap-6'>
          <LastFmSettings />
          <ListenBrainzSettings />
        </div>
      </TabsContent>

      <TabsContent value='server'>
        <ServerSettings @logout='$emit("logout")' :credentials='credentials' />
      </TabsContent>

      <TabsContent value='library'>
        <LibrarySettings
          @clear-cache='$emit("clear-cache")'
          @sync-library='$emit("sync-library")'
          :is-clearing='isClearing'
          :is-syncing='isSyncing'
        />
      </TabsContent>
    </Tabs>
  </div>
</template>
