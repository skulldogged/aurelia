<script setup lang="ts">
  import { BookOpen, Info, Palette, Plug, Server } from 'lucide-vue-next'
  import { ref, watch } from 'vue'
  import { useRoute, useRouter } from 'vue-router'

  import AboutSettings from '@/components/settings/AboutSettings.vue'
  import AppearanceSettings from '@/components/settings/AppearanceSettings.vue'
  import IntegrationsSettings from '@/components/settings/IntegrationsSettings.vue'
  import LibrarySettings from '@/components/settings/LibrarySettings.vue'
  import ServerSettings from '@/components/settings/ServerSettings.vue'
  import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
  import { isMobile } from '@/lib/platform'

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
    if (newTab && typeof newTab === 'string') {
      activeTab.value = newTab
    }
  })

  // Watch for tab changes and update query param
  watch(activeTab, newTab => {
    router.replace({ query: { tab: newTab } })
  })
</script>

<template>
  <div class='mx-auto p-4'>
    <div class='mb-8'>
      <h1 class='mb-2 text-4xl font-bold text-foreground'>
        Settings
      </h1>
    </div>

    <Tabs v-model='activeTab' default-value='appearance'>
      <TabsList :class="['mb-6', isMobile() ? 'w-full h-16 grid grid-cols-5 gap-1 p-2' : '']">
        <TabsTrigger :class="[isMobile() ? 'h-12 px-3' : '']" value='appearance'>
          <Palette v-if='isMobile()' class='h-6 w-6' />
          <span v-if='!isMobile()'>Appearance</span>
        </TabsTrigger>
        <TabsTrigger :class="[isMobile() ? 'h-12 px-3' : '']" value='integrations'>
          <Plug v-if='isMobile()' class='h-6 w-6' />
          <span v-if='!isMobile()'>Integrations</span>
        </TabsTrigger>
        <TabsTrigger :class="[isMobile() ? 'h-12 px-3' : '']" value='server'>
          <Server v-if='isMobile()' class='h-6 w-6' />
          <span v-if='!isMobile()'>Server</span>
        </TabsTrigger>
        <TabsTrigger :class="[isMobile() ? 'h-12 px-3' : '']" value='library'>
          <BookOpen v-if='isMobile()' class='h-6 w-6' />
          <span v-if='!isMobile()'>Library</span>
        </TabsTrigger>
        <TabsTrigger :class="[isMobile() ? 'h-12 px-3' : '']" value='about'>
          <Info v-if='isMobile()' class='h-6 w-6' />
          <span v-if='!isMobile()'>About</span>
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
