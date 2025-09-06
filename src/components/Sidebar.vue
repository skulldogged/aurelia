<script setup lang="ts">
  import { Home, Music, Users, Disc, Settings } from 'lucide-vue-next'

  defineProps<{
    currentView: string
    isCollapsed: boolean
  }>()

  defineEmits<{
    navigate: [view: string]
  }>()
</script>

<template>
  <div
    :class="[
      'bg-gray-300 dark:bg-black flex flex-col flex-shrink-0 transition-all duration-300 ease-in-out',
      isCollapsed ? 'w-16' : 'w-48',
    ]"
  >
    <nav class='flex flex-col flex-grow mx-2 mb-2'>
      <div class='flex-grow space-y-2'>
        <router-link
          :class="[
            'flex items-center h-10 rounded-md text-sm font-medium',
            currentView === 'home'
              ? 'bg-primary text-primary-foreground'
              : 'hover:bg-muted',
          ]"
          to='/'
        >
          <div class='w-12 flex-shrink-0 flex justify-center items-center'>
            <Home class='w-5 h-5' />
          </div>
          <div
            :class="[
              'overflow-hidden transition-all duration-200 ease-in-out',
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
              ? 'bg-primary text-primary-foreground'
              : 'hover:bg-muted',
          ]"
          to='/songs'
        >
          <div class='w-12 flex-shrink-0 flex justify-center items-center'>
            <Music class='w-5 h-5' />
          </div>
          <div
            :class="[
              'overflow-hidden transition-all duration-200 ease-in-out',
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
              ? 'bg-primary text-primary-foreground'
              : 'hover:bg-muted',
          ]"
          to='/artists'
        >
          <div class='w-12 flex-shrink-0 flex justify-center items-center'>
            <Users class='w-5 h-5' />
          </div>
          <div
            :class="[
              'overflow-hidden transition-all duration-200 ease-in-out',
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
              ? 'bg-primary text-primary-foreground'
              : 'hover:bg-muted',
          ]"
          to='/albums'
        >
          <div class='w-12 flex-shrink-0 flex justify-center items-center'>
            <Disc class='w-5 h-5' />
          </div>
          <div
            :class="[
              'overflow-hidden transition-all duration-200 ease-in-out',
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
            ? 'bg-primary text-primary-foreground'
            : 'hover:bg-muted',
        ]"
        to='/settings'
      >
        <div class='w-12 flex-shrink-0 flex justify-center items-center'>
          <Settings class='w-5 h-5' />
        </div>
        <div
          :class="[
            'overflow-hidden transition-all duration-200 ease-in-out',
            isCollapsed ? 'max-w-0 opacity-0' : 'max-w-full opacity-100',
          ]"
        >
          <span class='whitespace-nowrap'>Settings</span>
        </div>
      </router-link>
    </nav>
  </div>
</template>
