<script setup lang="ts">
  import { Disc, Home, ListMusic, Music, Search, Settings, Users } from 'lucide-vue-next'
  import { computed } from 'vue'

  import { useOrientation } from '@/composables/useOrientation'
  import { getPlatform, Platform } from '@/lib/platform'
  import { usePlayerStore } from '@/stores'

  const isMacos = computed(() => getPlatform() === Platform.MacOS)

  const props = defineProps<{
    currentView:       string
    isCollapsed:       boolean
    isMobilePortrait?: boolean
  }>()

  const emit = defineEmits<{
    'global-search': []
    navigate:        [view: string]
  }>()

  const { isLandscape } = useOrientation()

  const isMobileLandscape = computed(() => isLandscape.value)

  // Now that we've removed window transparency, use solid background
  const sidebarBgClass = computed(() => 'bg-background-dark')

  const sidebarWidthClass = computed(() => {
    if (props.isMobilePortrait || isMobileLandscape.value)
      return ''

    if (props.isCollapsed)
      return isMacos.value ? 'w-[81px]' : 'w-[65px]'

    return 'w-48'
  })

  const navItemClass = computed(() => (view: string) => {
    const isActive = props.currentView === view
    const baseClasses = 'nav-item flex items-center rounded-lg text-sm font-medium transition-all duration-200'

    if (isMobileLandscape.value) {
      return [
        baseClasses,
        'justify-center px-3 py-4',
        isActive
          ? 'bg-accent text-accent-foreground shadow-sm'
          : 'text-muted-foreground hover:text-foreground hover:bg-accent/20',
      ]
    }

    if (props.isMobilePortrait) {
      return [
        baseClasses,
        'justify-center flex-1 h-11',
        isActive
          ? 'bg-accent text-accent-foreground shadow-sm'
          : 'text-muted-foreground hover:text-foreground hover:bg-accent/20',
      ]
    }

    // Desktop - transition padding for smooth collapse animation
    return [
      baseClasses,
      'h-10 gap-x-3 transition-[padding] duration-200 ease',
      props.isCollapsed ? 'pl-0' : 'pl-3',
      isActive
        ? 'bg-accent text-accent-foreground shadow-sm'
        : 'text-muted-foreground hover:text-foreground hover:bg-accent/20',
    ]
  })

  const navIconClass = computed(() => {
    if (props.isMobilePortrait || isMobileLandscape.value)
      return 'flex justify-center items-center'

    // Desktop - always use fixed width to prevent shifting during collapse animation
    return 'w-12 shrink-0 flex justify-center items-center'
  })

  // Offset by -1px to compensate for the button's left border
  // so the search icon aligns with nav icons (which have no border)
  const searchIconClass = computed(() => 'w-12 shrink-0 flex justify-center items-center -ml-px')

  const playerStore = usePlayerStore()

  const shadowBottomClass = computed(() =>
    playerStore.playlist.length > 0 ? 'bottom-20' : 'bottom-0',
  )
</script>

<template>
  <div
    :class="[
      sidebarBgClass,
      'sidebar flex shrink-0 overflow-visible transition-[width] duration-200 ease',
      isMobileLandscape
        ? 'flex-col h-full justify-around items-center py-4'
        : props.isMobilePortrait
          ? 'flex-row w-full justify-around items-center px-4'
          : 'flex-col h-full',
      !(props.isMobilePortrait || isMobileLandscape) && sidebarWidthClass,
    ]"
  >
    <div
      :class="['absolute z-10 top-12 pointer-events-none outer-shadow-right', shadowBottomClass]"
    />
    <div :class="['flex', 'flex-col', 'h-full', isMacos && 'pt-10']">
      <!-- Search -->
      <div v-if='!(props.isMobilePortrait || isMobileLandscape)' class='m-2 mb-2'>
        <button
          @click="emit('global-search')"
          class='
            flex items-center h-10 w-full rounded-md text-sm font-medium
            bg-background border border-border hover:border-accent transition-colors
          '
        >
          <div :class='searchIconClass'>
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
            : 'flex flex-col grow m-2 mt-0'"
      >
        <div v-if='!(props.isMobilePortrait || isMobileLandscape)' class='grow space-y-2'>
          <RouterLink
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
          </RouterLink>
          <RouterLink
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
          </RouterLink>
          <RouterLink
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
          </RouterLink>
          <RouterLink
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
          </RouterLink>
          <RouterLink
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
          </RouterLink>
        </div>
        <RouterLink
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
        </RouterLink>
        <!-- Mobile portrait navigation -->
        <template v-if='props.isMobilePortrait || isMobileLandscape'>
          <RouterLink
            :class="navItemClass('home')"
            to='/'
          >
            <div :class='navIconClass'>
              <Home class='size-5' />
            </div>
          </RouterLink>
          <RouterLink
            :class="navItemClass('songs')"
            to='/songs'
          >
            <div :class='navIconClass'>
              <Music class='size-5' />
            </div>
          </RouterLink>
          <RouterLink
            :class="navItemClass('artists')"
            to='/artists'
          >
            <div :class='navIconClass'>
              <Users class='size-5' />
            </div>
          </RouterLink>
          <RouterLink
            :class="navItemClass('albums')"
            to='/albums'
          >
            <div :class='navIconClass'>
              <Disc class='size-5' />
            </div>
          </RouterLink>
          <RouterLink
            :class="navItemClass('playlists')"
            to='/playlists'
          >
            <div :class='navIconClass'>
              <ListMusic class='size-5' />
            </div>
          </RouterLink>
          <RouterLink
            :class="navItemClass('settings')"
            to='/settings'
          >
            <div :class='navIconClass'>
              <Settings class='size-5' />
            </div>
          </RouterLink>
        </template>
      </nav>
    </div>
  </div>
</template>
