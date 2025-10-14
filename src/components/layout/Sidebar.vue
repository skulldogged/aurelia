<script setup lang="ts">
  import { Disc, Home, ListMusic, Music, Search, Settings, Users } from 'lucide-vue-next'
  import { computed } from 'vue'

  import { useBlurStore } from '@/stores'

  defineProps<{
    currentView: string
    isCollapsed: boolean
  }>()

  const emit = defineEmits<{
    'global-search': []
    navigate:        [view: string]
  }>()

  const blurStore = useBlurStore()

  const sidebarBgClass = computed(
    () => blurStore.selectedBlurMode.name !== 'none'
      ? 'bg-transparent'
      : 'bg-background-dark',
  )

</script>

<template>
  <div
    :class="[
      sidebarBgClass,
      'flex flex-col flex-shrink-0 ease-in-out',
      isCollapsed ? 'w-16' : 'w-48',
    ]"
  >
    <!-- Search -->
    <div class='m-2 mb-2'>
      <button
        @click="emit('global-search')"
        class='flex items-center h-10 w-full rounded-md text-sm font-medium
               bg-background border border-border hover:border-accent transition-colors'
      >
        <div class='w-12 flex-shrink-0 flex justify-center items-center'>
          <Search class='size-5 text-muted-foreground' />
        </div>
        <div
          :class="[
            'overflow-hidden transition-all duration-150 ease-in-out flex justify-between items-center w-full',
            isCollapsed ? 'max-w-0 opacity-0' : 'max-w-full opacity-100',
          ]"
        >
          <span class='whitespace-nowrap text-muted-foreground'>Search...</span>
          <kbd
            class='
              pointer-events-none mr-2 inline-flex h-5 select-none items-center
              gap-1 rounded border bg-muted px-1.5 font-mono text-[10px] font-medium
              text-muted-foreground opacity-100
            '
          >
            Ctrl+K
          </kbd>
        </div>
      </button>
    </div>
    <nav class='flex flex-col flex-grow m-2 mt-0'>
      <div class='flex-grow space-y-2'>
        <router-link
          :class="[
            'flex items-center h-10 rounded-md text-sm font-medium',
            currentView === 'home'
              ? 'bg-accent text-accent-foreground'
              : 'hover:bg-accent/20',
          ]"
          to='/'
        >
          <div class='w-12 flex-shrink-0 flex justify-center items-center'>
            <Home class='size-5' />
          </div>
          <div
            :class="[
              'overflow-hidden transition-all duration-150 ease-in-out',
              isCollapsed ? 'max-w-0 opacity-0' : 'max-w-full opacity-100',
            ]"
          >
            <span class='whitespace-nowrap'>Home</span>
          </div>
        </router-link>
        <router-link
          :class="[
            'flex items-center h-10 rounded-md text-sm font-medium',
            currentView === 'songs'
              ? 'bg-accent text-accent-foreground'
              : 'hover:bg-accent/20',
          ]"
          to='/songs'
        >
          <div class='w-12 flex-shrink-0 flex justify-center items-center'>
            <Music class='size-5' />
          </div>
          <div
            :class="[
              'overflow-hidden transition-all duration-150 ease-in-out',
              isCollapsed ? 'max-w-0 opacity-0' : 'max-w-full opacity-100',
            ]"
          >
            <span class='whitespace-nowrap'>Songs</span>
          </div>
        </router-link>
        <router-link
          :class="[
            'flex items-center h-10 rounded-md text-sm font-medium',
            currentView === 'artists'
              ? 'bg-accent text-accent-foreground'
              : 'hover:bg-accent/20',
          ]"
          to='/artists'
        >
          <div class='w-12 flex-shrink-0 flex justify-center items-center'>
            <Users class='size-5' />
          </div>
          <div
            :class="[
              'overflow-hidden transition-all duration-150 ease-in-out',
              isCollapsed ? 'max-w-0 opacity-0' : 'max-w-full opacity-100',
            ]"
          >
            <span class='whitespace-nowrap'>Artists</span>
          </div>
        </router-link>
        <router-link
          :class="[
            'flex items-center h-10 rounded-md text-sm font-medium',
            currentView === 'albums'
              ? 'bg-accent text-accent-foreground'
              : 'hover:bg-accent/20',
          ]"
          to='/albums'
        >
          <div class='w-12 flex-shrink-0 flex justify-center items-center'>
            <Disc class='size-5' />
          </div>
          <div
            :class="[
              'overflow-hidden transition-all duration-150 ease-in-out',
              isCollapsed ? 'max-w-0 opacity-0' : 'max-w-full opacity-100',
            ]"
          >
            <span class='whitespace-nowrap'>Albums</span>
          </div>
        </router-link>
        <router-link
          :class="[
            'flex items-center h-10 rounded-md text-sm font-medium',
            currentView === 'playlists'
              ? 'bg-accent text-accent-foreground'
              : 'hover:bg-accent/20',
          ]"
          to='/playlists'
        >
          <div class='w-12 flex-shrink-0 flex justify-center items-center'>
            <ListMusic class='size-5' />
          </div>
          <div
            :class="[
              'overflow-hidden transition-all duration-150 ease-in-out',
              isCollapsed ? 'max-w-0 opacity-0' : 'max-w-full opacity-100',
            ]"
          >
            <span class='whitespace-nowrap'>Playlists</span>
          </div>
        </router-link>
      </div>
      <router-link
        :class="[
          'flex items-center h-10 rounded-md text-sm font-medium',
          currentView === 'settings'
            ? 'bg-accent text-accent-foreground'
            : 'hover:bg-accent/20',
        ]"
        to='/settings'
      >
        <div class='w-12 flex-shrink-0 flex justify-center items-center'>
          <Settings class='size-5' />
        </div>
        <div
          :class="[
            'overflow-hidden transition-all duration-150 ease-in-out',
            isCollapsed ? 'max-w-0 opacity-0' : 'max-w-full opacity-100',
          ]"
        >
          <span class='whitespace-nowrap'>Settings</span>
        </div>
      </router-link>
    </nav>
  </div>
</template>
