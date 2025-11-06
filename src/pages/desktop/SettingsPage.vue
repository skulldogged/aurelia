<script setup lang="ts">
  import type { Component } from 'vue'

  import { BookOpen, Info, Palette, Plug, Server } from 'lucide-vue-next'
  import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
  import { useRoute, useRouter } from 'vue-router'

  import SettingsPageTopBar from '@/components/desktop/SettingsPageTopBar.vue'
  import AboutSettings from '@/components/settings/AboutSettings.vue'
  import AppearanceSettings from '@/components/settings/AppearanceSettings.vue'
  import IntegrationsSettings from '@/components/settings/IntegrationsSettings.vue'
  import LibrarySettings from '@/components/settings/LibrarySettings.vue'
  import ServerSettings from '@/components/settings/ServerSettings.vue'
  import { useTopBar } from '@/composables/useTopBar'

  interface Credentials {
    serverUrl: string
    token:     string
    userId:    string
    username:  string
  }

  interface SettingTab {
    description: string
    icon:        Component
    id:          string
    label:       string
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

  // Settings navigation items
  const settingsTabs: SettingTab[] = [
    { description: 'Customize the app', icon: Palette, id: 'appearance', label: 'Appearance' },
    { description: 'Music services', icon: Plug, id: 'integrations', label: 'Integrations' },
    { description: 'Server connection', icon: Server, id: 'server', label: 'Server' },
    { description: 'Manage library', icon: BookOpen, id: 'library', label: 'Library' },
    { description: 'About this app', icon: Info, id: 'about', label: 'About' },
  ]

  // Get initial tab from query param or default to 'appearance'
  const activeTab = ref(route.query.tab as string || 'appearance')

  const currentTabInfo = computed(
    () => settingsTabs.find(tab => tab.id === activeTab.value),
  )

  // Watch for query param changes
  watch(() => route.query.tab, newTab => {
    if (newTab && typeof newTab === 'string')
      activeTab.value = newTab
  })

  // Watch for tab changes and update query param + top bar
  watch(activeTab, newTab => {
    router.replace({ query: { tab: newTab } })
    // Update top bar props when tab changes
    setTopBarContent({
      component: SettingsPageTopBar,
      id:        'settings-page',
      props:     {
        activeTabLabel: currentTabInfo.value?.label || 'Settings',
      },
    })
  })

  // Set up top bar content when component mounts
  onMounted(() => {
    setTopBarContent({
      component: SettingsPageTopBar,
      id:        'settings-page',
      props:     {
        activeTabLabel: currentTabInfo.value?.label || 'Settings',
      },
    })
  })

  // Clean up top bar content when component unmounts
  onUnmounted(() => {
    clearTopBarContent()
  })
</script>

<template>
  <div class='flex gap-8 h-full'>
    <!-- Sidebar Navigation -->
    <aside class='w-56 shrink-0 border-r border-border/20 pt-4 pb-8 px-4'>
      <div class='space-y-1'>
        <button
          v-for='tab in settingsTabs'
          @click='activeTab = tab.id'
          :key='tab.id'
          :class='[
            "w-full flex items-center gap-3 px-4 py-3 rounded-lg transition-all text-left group",
            activeTab === tab.id
              ? "bg-accent/10 text-accent"
              : "text-foreground/70 hover:bg-accent/5 hover:text-foreground"
          ]'
        >
          <component :is='tab.icon' class='h-5 w-5 shrink-0' />
          <div class='flex-1 min-w-0'>
            <div class='font-medium text-sm leading-snug'>
              {{ tab.label }}
            </div>
            <div
              :class='[
                "text-xs leading-snug mt-0.5",
                activeTab === tab.id ? "text-accent/70" : "text-muted-foreground"
              ]'
            >
              {{ tab.description }}
            </div>
          </div>
        </button>
      </div>
    </aside>

    <!-- Main Content Area -->
    <main class='flex-1 py-8 pr-8 overflow-y-auto'>
      <!-- Dynamic Content -->
      <div class='space-y-6'>
        <!-- Current Section -->
        <component
          :is='activeTab === "appearance" ? AppearanceSettings
            : activeTab === "integrations" ? IntegrationsSettings
              : activeTab === "server" ? ServerSettings
                : activeTab === "library" ? LibrarySettings
                  : activeTab === "about" ? AboutSettings
                    : AppearanceSettings'
          @clear-cache='$emit("clear-cache")'
          @logout='$emit("logout")'
          @sync-library='$emit("sync-library")'
          :credentials='credentials'
          :is-clearing='isClearing'
          :is-syncing='isSyncing'
        />
      </div>
    </main>
  </div>
</template>

<style scoped>
  main {
    max-width: 900px;
  }
</style>
