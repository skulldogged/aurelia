<script setup lang="ts">
  import { ref, watch } from 'vue'
  import { useRoute, useRouter } from 'vue-router'

  import AboutSettings from '@/components/settings/AboutSettings.vue'
  import AppearanceSettings from '@/components/settings/AppearanceSettings.vue'
  import IntegrationsSettings from '@/components/settings/IntegrationsSettings.vue'
  import LibrarySettings from '@/components/settings/LibrarySettings.vue'
  import ServerSettings from '@/components/settings/ServerSettings.vue'
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

  const route = useRoute()
  const router = useRouter()

  // Get initial tab from query param or default to 'appearance'
  const activeTab = ref(route.query.tab as string || 'appearance')

  // Watch for query param changes
  watch(() => route.query.tab, newTab => {
    if (newTab && typeof newTab === 'string')
      activeTab.value = newTab
  })

  // Watch for tab changes and update query param
  watch(activeTab, newTab => {
    router.replace({ query: { tab: newTab } })
  })
</script>

<template>
  <div class='p-4'>
    <Tabs
      v-model='activeTab'
      class='w-full'
      default-value='appearance'
    >
      <TabsList class='grid w-full grid-cols-5 mb-6'>
        <TabsTrigger
          class='text-xs'
          value='appearance'
        >
          Appearance
        </TabsTrigger>
        <TabsTrigger
          class='text-xs'
          value='integrations'
        >
          Integrations
        </TabsTrigger>
        <TabsTrigger
          class='text-xs'
          value='server'
        >
          Server
        </TabsTrigger>
        <TabsTrigger
          class='text-xs'
          value='library'
        >
          Library
        </TabsTrigger>
        <TabsTrigger
          class='text-xs'
          value='about'
        >
          About
        </TabsTrigger>
      </TabsList>

      <TabsContent value='appearance'>
        <AppearanceSettings />
      </TabsContent>

      <TabsContent value='integrations'>
        <IntegrationsSettings />
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

      <TabsContent value='about'>
        <AboutSettings />
      </TabsContent>
    </Tabs>
  </div>
</template>
