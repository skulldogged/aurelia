<script setup lang="ts">
  import { Disc, Home, ListMusic, Music, Search, Settings, Users } from 'lucide-vue-next'
  import { computed } from 'vue'

  import { useOrientation } from '@/composables/useOrientation'
  import { useBlurStore } from '@/stores'

  const props = defineProps<{
    currentView:       string
    isCollapsed:       boolean
    isMobilePortrait?: boolean
  }>()

  const emit = defineEmits<{
    'global-search': []
    navigate:        [view: string]
  }>()

  const blurStore = useBlurStore()

  const { isLandscape } = useOrientation()

  const isMobileLandscape = computed(() => isLandscape.value)

  const sidebarBgClass = computed(
    () => blurStore.selectedBlurMode.name !== 'none'
      ? 'bg-transparent'
      : 'bg-background-dark',
  )

  const navItemClass = computed(() => (view: string) => [
    isMobileLandscape.value
      ? [
        'flex items-center justify-center rounded-md text-sm font-medium px-3 py-4',
        props.currentView === view
          ? 'bg-accent text-accent-foreground'
          : 'hover:bg-accent/20',
      ]
      : props.isMobilePortrait
        ? [
          'flex items-center justify-center rounded-md text-sm font-medium flex-1 h-10',
          props.currentView === view
            ? 'bg-accent text-accent-foreground'
            : 'hover:bg-accent/20',
        ]
        : [ // This is the desktop case
          'flex items-center h-10 rounded-md text-sm font-medium pl-3 gap-x-3',
          props.currentView === view
            ? 'bg-accent text-accent-foreground'
            : 'hover:bg-accent/20',
        ],
  ])

  const navIconClass = computed(() =>
    props.isMobilePortrait || isMobileLandscape
      ? 'flex justify-center items-center'
      : 'w-12 flex-shrink-0 flex justify-center items-center',
  )

</script>

<template>
  <div
    :class="[
      sidebarBgClass,
      'flex flex-shrink-0 ease-in-out',
      isMobileLandscape
        ? 'flex-col h-full justify-around items-center py-4'
        : props.isMobilePortrait
          ? 'flex-row w-full justify-around items-center px-4'
          : 'flex-col h-full',
      !(props.isMobilePortrait || isMobileLandscape) && (props.isCollapsed ? 'w-16' : 'w-48'),
    ]"
  >
    <!-- Search -->
    <div v-if='!(props.isMobilePortrait || isMobileLandscape)' class='m-2 mb-2'>
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
    <nav
      :class="isMobileLandscape
        ? 'flex flex-col flex-1 justify-around'
        : props.isMobilePortrait
          ? 'flex flex-1'
          : 'flex flex-col flex-grow m-2 mt-0'"
    >
      <div v-if='!(props.isMobilePortrait || isMobileLandscape)' :class="'flex-grow space-y-2'">
        <router-link
          :class="navItemClass('home')"
          to='/'
        >
          <div :class='navIconClass'>
            <Home class='size-5' />
          </div>
          <div
            v-if='!(props.isMobilePortrait || isMobileLandscape)'
            :class="[
              'overflow-hidden transition-all duration-150 ease-in-out',
              props.isCollapsed ? 'max-w-0 opacity-0' : 'max-w-full opacity-100',
            ]"
          >
            <span class='whitespace-nowrap'>Home</span>
          </div>
        </router-link>
        <router-link
          :class="navItemClass('songs')"
          to='/songs'
        >
          <div :class='navIconClass'>
            <Music class='size-5' />
          </div>
          <div
            v-if='!isMobilePortrait'
            :class="[
              'overflow-hidden transition-all duration-150 ease-in-out',
              isCollapsed ? 'max-w-0 opacity-0' : 'max-w-full opacity-100',
            ]"
          >
            <span class='whitespace-nowrap'>Songs</span>
          </div>
        </router-link>
        <router-link
          :class="navItemClass('artists')"
          to='/artists'
        >
          <div :class='navIconClass'>
            <Users class='size-5' />
          </div>
          <div
            v-if='!(props.isMobilePortrait || isMobileLandscape)'
            :class="[
              'overflow-hidden transition-all duration-150 ease-in-out',
              props.isCollapsed ? 'max-w-0 opacity-0' : 'max-w-full opacity-100',
            ]"
          >
            <span class='whitespace-nowrap'>Artists</span>
          </div>
        </router-link>
        <router-link
          :class="navItemClass('albums')"
          to='/albums'
        >
          <div :class='navIconClass'>
            <Disc class='size-5' />
          </div>
          <div
            v-if='!(props.isMobilePortrait || isMobileLandscape)'
            :class="[
              'overflow-hidden transition-all duration-150 ease-in-out',
              props.isCollapsed ? 'max-w-0 opacity-0' : 'max-w-full opacity-100',
            ]"
          >
            <span class='whitespace-nowrap'>Albums</span>
          </div>
        </router-link>
        <router-link
          :class="navItemClass('playlists')"
          to='/playlists'
        >
          <div :class='navIconClass'>
            <ListMusic class='size-5' />
          </div>
          <div
            v-if='!(props.isMobilePortrait || isMobileLandscape)'
            :class="[
              'overflow-hidden transition-all duration-150 ease-in-out',
              props.isCollapsed ? 'max-w-0 opacity-0' : 'max-w-full opacity-100',
            ]"
          >
            <span class='whitespace-nowrap'>Playlists</span>
          </div>
        </router-link>
      </div>
      <router-link
        v-if='!(props.isMobilePortrait || isMobileLandscape)'
        :class="navItemClass('settings')"
        to='/settings'
      >
        <div :class='navIconClass'>
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
      <!-- Mobile portrait navigation -->
      <template v-if='props.isMobilePortrait || isMobileLandscape'>
        <router-link
          :class="navItemClass('home')"
          to='/'
        >
          <div :class='navIconClass'>
            <Home class='size-5' />
          </div>
        </router-link>
        <router-link
          :class="navItemClass('songs')"
          to='/songs'
        >
          <div :class='navIconClass'>
            <Music class='size-5' />
          </div>
        </router-link>
        <router-link
          :class="navItemClass('artists')"
          to='/artists'
        >
          <div :class='navIconClass'>
            <Users class='size-5' />
          </div>
        </router-link>
        <router-link
          :class="navItemClass('albums')"
          to='/albums'
        >
          <div :class='navIconClass'>
            <Disc class='size-5' />
          </div>
        </router-link>
        <router-link
          :class="navItemClass('playlists')"
          to='/playlists'
        >
          <div :class='navIconClass'>
            <ListMusic class='size-5' />
          </div>
        </router-link>
        <router-link
          :class="navItemClass('settings')"
          to='/settings'
        >
          <div :class='navIconClass'>
            <Settings class='size-5' />
          </div>
        </router-link>
      </template>
    </nav>
  </div>
</template>
