<script setup lang="ts">
  import type { Component } from 'vue'
  import type { Credentials } from '../generated'

  import { BookOpen, Info, Palette, Plug, Server } from 'lucide-vue-next'
  import { onMounted, onUnmounted, ref } from 'vue'

  import SettingsPageTopBar from '../components/desktop/SettingsPageTopBar.vue'
  import AboutSettings from '../components/settings/AboutSettings.vue'
  import AppearanceSettings from '../components/settings/AppearanceSettings.vue'
  import IntegrationsSettings from '../components/settings/IntegrationsSettings.vue'
  import LibrarySettings from '../components/settings/LibrarySettings.vue'
  import ServerSettings from '../components/settings/ServerSettings.vue'
  import { useTopBar } from '../composables/useTopBar'

  interface SettingTab {
    description: string
    icon:        Component
    id:          string
    label:       string
  }

  defineProps<{
    credentials:   Credentials | null
    isClearing:    boolean
    isSyncing:     boolean
    lastSyncTime?: null | string
  }>()

  defineEmits<{
    (e: 'logout'): void
    (e: 'sync-library'): void
    (e: 'clear-cache'): void
  }>()

  const { clearTopBarContent, setTopBarContent } = useTopBar()

  const settingsTabs: SettingTab[] = [
    { description: 'Customize the app', icon: Palette, id: 'appearance', label: 'Appearance' },
    { description: 'Music services', icon: Plug, id: 'integrations', label: 'Integrations' },
    { description: 'Server connection', icon: Server, id: 'server', label: 'Server' },
    { description: 'Manage library', icon: BookOpen, id: 'library', label: 'Library' },
    { description: 'About this app', icon: Info, id: 'about', label: 'About' },
  ]

  const sectionRefs = ref<Record<string, HTMLElement | null>>({})
  const mainContentRef = ref<HTMLElement | null>(null)
  const activeTab = ref('appearance')

  const scrollToSection = (tabId: string): void => {
    const element = sectionRefs.value[tabId]
    if (element) {
      activeTab.value = tabId
      element.scrollIntoView({ behavior: 'smooth', block: 'start' })
      setTopBarContent({
        component: SettingsPageTopBar,
        id:        'settings-page',
        props:     {
          activeTabLabel: settingsTabs.find(t => t.id === tabId)?.label || 'Settings',
        },
      })
    }
  }

  let sectionObserver: IntersectionObserver | null = null

  const observeSections = (): void => {
    if (!mainContentRef.value)
      return

    sectionObserver = new IntersectionObserver(entries => {
      const visibleSection = entries
        .filter(entry => entry.isIntersecting)
        .sort((a, b) => a.boundingClientRect.top - b.boundingClientRect.top)[0]

      if (visibleSection)
        activeTab.value = (visibleSection.target as HTMLElement).dataset.settingsSection ?? activeTab.value
    }, {
      root:       mainContentRef.value,
      rootMargin: '-96px 0px -70% 0px',
      threshold:  0,
    })

    for (const tab of settingsTabs) {
      const element = sectionRefs.value[tab.id]
      if (element)
        sectionObserver.observe(element)
    }
  }

  onMounted(() => {
    observeSections()
    setTopBarContent({
      component: SettingsPageTopBar,
      id:        'settings-page',
      props:     {
        activeTabLabel: 'Settings',
      },
    })
  })

  onUnmounted(() => {
    sectionObserver?.disconnect()
    clearTopBarContent()
  })
</script>

<template>
  <div class='flex gap-8 h-full'>
    <nav class='w-56 shrink-0 border-r border-border/20 pt-4 pb-8 px-4 space-y-1 overflow-y-auto'>
      <button
        v-for='tab in settingsTabs'
        @click='scrollToSection(tab.id)'
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
    </nav>

    <div
      ref='mainContentRef'
      class='flex-1 min-w-0 py-8 overflow-y-auto overflow-x-hidden'
    >
      <div class='flex justify-center px-8'>
        <div class='w-full max-w-4xl space-y-16'>
          <!-- Appearance Section -->
          <section
            :ref="el => sectionRefs['appearance'] = el as HTMLElement"
            data-settings-section='appearance'
            class='space-y-8'
          >
            <div class='py-4'>
              <div class='flex items-start space-x-4'>
                <div class='p-3 bg-accent/10 rounded-lg shrink-0'>
                  <Palette class='size-6 text-accent' />
                </div>
                <div>
                  <h1 class='text-3xl font-semibold'>
                    Appearance
                  </h1>
                </div>
              </div>
            </div>
            <AppearanceSettings
              @clear-cache='$emit("clear-cache")'
              @logout='$emit("logout")'
              @sync-library='$emit("sync-library")'
              :credentials='credentials'
              :is-clearing='isClearing'
              :is-syncing='isSyncing'
            />
          </section>

          <!-- Integrations Section -->
          <section
            :ref="el => sectionRefs['integrations'] = el as HTMLElement"
            data-settings-section='integrations'
            class='space-y-8'
          >
            <div class='py-4'>
              <div class='flex items-start space-x-4'>
                <div class='p-3 bg-accent/10 rounded-lg shrink-0'>
                  <Plug class='size-6 text-accent' />
                </div>
                <div>
                  <h1 class='text-3xl font-semibold'>
                    Integrations
                  </h1>
                </div>
              </div>
            </div>
            <IntegrationsSettings
              @clear-cache='$emit("clear-cache")'
              @logout='$emit("logout")'
              @sync-library='$emit("sync-library")'
              :credentials='credentials'
              :is-clearing='isClearing'
              :is-syncing='isSyncing'
            />
          </section>

          <!-- Server Section -->
          <section
            :ref="el => sectionRefs['server'] = el as HTMLElement"
            data-settings-section='server'
            class='space-y-8'
          >
            <div class='py-4'>
              <div class='flex items-start space-x-4'>
                <div class='p-3 bg-accent/10 rounded-lg shrink-0'>
                  <Server class='size-6 text-accent' />
                </div>
                <div>
                  <h1 class='text-3xl font-semibold'>
                    Server
                  </h1>
                </div>
              </div>
            </div>
            <ServerSettings
              @clear-cache='$emit("clear-cache")'
              @logout='$emit("logout")'
              @sync-library='$emit("sync-library")'
              :credentials='credentials'
              :is-clearing='isClearing'
              :is-syncing='isSyncing'
            />
          </section>

          <!-- Library Section -->
          <section
            :ref="el => sectionRefs['library'] = el as HTMLElement"
            data-settings-section='library'
            class='space-y-8'
          >
            <div class='py-4'>
              <div class='flex items-start space-x-4'>
                <div class='p-3 bg-accent/10 rounded-lg shrink-0'>
                  <BookOpen class='size-6 text-accent' />
                </div>
                <div>
                  <h1 class='text-3xl font-semibold'>
                    Library
                  </h1>
                </div>
              </div>
            </div>
            <LibrarySettings
              @clear-cache='$emit("clear-cache")'
              @sync-library='$emit("sync-library")'
              :is-clearing='isClearing'
              :is-syncing='isSyncing'
              :last-sync-time='lastSyncTime'
            />
          </section>

          <!-- About Section -->
          <section
            :ref="el => sectionRefs['about'] = el as HTMLElement"
            data-settings-section='about'
            class='space-y-8 pb-8'
          >
            <div class='py-4'>
              <div class='flex items-start space-x-4'>
                <div class='p-3 bg-accent/10 rounded-lg shrink-0'>
                  <Info class='size-6 text-accent' />
                </div>
                <div>
                  <h1 class='text-3xl font-semibold'>
                    About
                  </h1>
                </div>
              </div>
            </div>
            <AboutSettings
              @clear-cache='$emit("clear-cache")'
              @logout='$emit("logout")'
              @sync-library='$emit("sync-library")'
              :credentials='credentials'
              :is-clearing='isClearing'
              :is-syncing='isSyncing'
            />
          </section>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
[data-settings-section] {
  content-visibility: auto;
  contain-intrinsic-size: auto 48rem;
}
</style>
