<script setup lang="ts">
  import { Disc, Home, Music, Search, Settings, Users } from 'lucide-vue-next'
  import { ref } from 'vue'

  import { Input } from '@/components/ui/input'

  defineProps<{
    currentView: string
    isCollapsed: boolean
  }>()

  const emit = defineEmits<{
    'global-search': [query: string]
    navigate:        [view: string]
  }>()

  const globalSearchQuery = ref('')

  const handleGlobalSearch = (): void => {
    emit('global-search', globalSearchQuery.value)
  }

  const handleSearchFocus = (): void => {
    if (globalSearchQuery.value.trim())
      emit('global-search', globalSearchQuery.value)
  }
</script>

<template>
  <div
    :class="[
      'bg-background-dark flex flex-col flex-shrink-0 ease-in-out',
      isCollapsed ? 'w-16' : 'w-48',
    ]"
  >
    <!-- Search bar at the top -->
    <div class='p-2'>
      <div class='relative flex items-center h-10 rounded-md'>
        <Search class='absolute left-3 top-1/2 transform -translate-y-1/2 w-5 h-5 text-foreground/60' />
        <Input
          @focus='handleSearchFocus'
          @input='handleGlobalSearch'
          v-model='globalSearchQuery'
          :class="[
            'w-full pl-12 bg-transparent border-0 text-foreground text-sm font-medium',
            'placeholder:text-muted-foreground focus-visible:ring-0',
            isCollapsed ? 'max-w-0 opacity-0' : 'max-w-full opacity-100'
          ]"
          :placeholder="isCollapsed ? '' : 'Search music...'"
        />
      </div>
    </div>

    <nav class='flex flex-col flex-grow mx-2 mb-2'>
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
            <Home class='w-5 h-5' />
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
            <Music class='w-5 h-5' />
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
            <Users class='w-5 h-5' />
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
            <Disc class='w-5 h-5' />
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
          <Settings class='w-5 h-5' />
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