<script setup lang="ts">
  import { onMounted, onUnmounted, ref, watch } from 'vue'
  import { useRoute, useRouter } from 'vue-router'

  import SettingsPageTopBar from '@/components/desktop/SettingsPageTopBar.vue'
  import AboutSettings from '@/components/settings/AboutSettings.vue'
  import AppearanceSettings from '@/components/settings/AppearanceSettings.vue'
  import IntegrationsSettings from '@/components/settings/IntegrationsSettings.vue'
  import LibrarySettings from '@/components/settings/LibrarySettings.vue'
  import ServerSettings from '@/components/settings/ServerSettings.vue'
  import { Tabs, TabsContent } from '@/components/ui/tabs'
  import { useTopBar } from '@/composables/useTopBar'

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

  // Use top bar for title display
  const { clearTopBarContent, setTopBarContent } = useTopBar()

  // Get initial tab from query param or default to 'appearance'
  const activeTab = ref(route.query.tab as string || 'appearance')

  // Watch for query param changes
  watch(() => route.query.tab, newTab => {
    if (newTab && typeof newTab === 'string') {
      activeTab.value = newTab
    }
  })

  // Watch for tab changes and update query param + top bar
  watch(activeTab, newTab => {
    router.replace({ query: { tab: newTab } })
    // Update top bar props when tab changes
    setTopBarContent({
      component: SettingsPageTopBar,
      id:        'settings-page',
      props:     {
        ...{
          activeTab:            newTab,
          'onUpdate:activeTab': (value: string) => {
            activeTab.value = value
          },
        },
      },
    })
  })

  // Set up top bar content when component mounts
  onMounted(() => {
    const propsValue = {
      activeTab:            activeTab.value,
      'onUpdate:activeTab': (value: string) => {
        activeTab.value = value
      },
    }
    setTopBarContent({
      component: SettingsPageTopBar,
      id:        'settings-page',
      props:     propsValue,
    })
  })

  // Clean up top bar content when component unmounts
  onUnmounted(() => {
    clearTopBarContent()
  })
</script>

<template>
  <div class='flex flex-col px-6 md:px-10 lg:px-16'>
    <!-- Settings Section - Edge-to-edge layout -->
    <div class='flex justify-center w-full'>
      <div class='w-full max-w-7xl py-8'>
        <!-- Settings Content -->
        <Tabs
          v-model='activeTab'
          class='w-full'
          default-value='appearance'
        >
          <TabsContent
            class='mt-6'
            value='appearance'
          >
            <AppearanceSettings />
          </TabsContent>

          <TabsContent
            class='mt-6'
            value='integrations'
          >
            <IntegrationsSettings />
          </TabsContent>

          <TabsContent
            class='mt-6'
            value='server'
          >
            <ServerSettings @logout='$emit("logout")' :credentials='credentials' />
          </TabsContent>

          <TabsContent
            class='mt-6'
            value='library'
          >
            <LibrarySettings
              @clear-cache='$emit("clear-cache")'
              @sync-library='$emit("sync-library")'
              :is-clearing='isClearing'
              :is-syncing='isSyncing'
            />
          </TabsContent>

          <TabsContent
            class='mt-6'
            value='about'
          >
            <AboutSettings />
          </TabsContent>
        </Tabs>
      </div>
    </div>
  </div>
</template>
