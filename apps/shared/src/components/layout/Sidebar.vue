<script setup lang="ts">
  import { Check, ChevronsUpDown, Disc, Home, ListMusic, LogOut, Music, Plus, RefreshCw, Search, Settings, Users } from 'lucide-vue-next'
  import { computed, onMounted, ref } from 'vue'
  import { useRouter } from 'vue-router'

  import type { Credentials } from '../../generated'

  import { apiClient } from '../../api/apiClient'
  import AddProfileDialog from '../auth/AddProfileDialog.vue'
  import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuLabel,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
  } from '../ui/dropdown-menu'
  import {
    buildProfileId,
    getActiveProfileId,
    loadProfiles,
    setActiveProfileId,
    type AuthProfile,
  } from '../../lib/profileStorage'
  import { getPlatform, Platform } from '../../lib/platform'
  import { useAuthStore, usePlayerStore } from '../../stores'

  const isMacos = computed(() => getPlatform() === Platform.MacOS)
  const router = useRouter()

  const props = defineProps<{
    currentView: string
    isCollapsed: boolean
  }>()

  const emit = defineEmits<{
    'global-search': []
    logout:          []
    navigate:        [view: string]
  }>()

  // Now that we've removed window transparency, use solid background
  const sidebarBgClass = computed(() => 'bg-background-dark')

  const sidebarWidthClass = computed(() => {
    if (props.isCollapsed)
      return isMacos.value ? 'w-[81px]' : 'w-[65px]'

    return 'w-48'
  })

  const navItemClass = computed(() => (view: string) => {
    const isActive = props.currentView === view
    const baseClasses = 'nav-item flex items-center rounded-lg text-sm font-medium transition-all duration-200'

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

  const navIconClass = computed(() =>
    // Desktop - always use fixed width to prevent shifting during collapse animation
    'w-12 shrink-0 flex justify-center items-center',
  )

  // Offset by -1px to compensate for the button's left border
  // so the search icon aligns with nav icons (which have no border)
  const searchIconClass = computed(() => 'w-12 shrink-0 flex justify-center items-center -ml-px')

  const playerStore = usePlayerStore()
  const authStore = useAuthStore()
  const profileActionError = ref('')
  const profiles = ref<AuthProfile[]>([])
  const showAddProfileDialog = ref(false)
  const switchingProfileId = ref<null | string>(null)

  const shadowBottomClass = computed(() =>
    playerStore.playlist.length > 0 ? 'bottom-20' : 'bottom-0',
  )

  const currentCredentials = computed<Credentials | null>(() => {
    if (!authStore.token || !authStore.serverUrl || !authStore.userId)
      return null

    return {
      provider:  authStore.provider,
      serverUrl: authStore.serverUrl,
      token:     authStore.token,
      userId:    authStore.userId,
      username:  authStore.username,
    }
  })

  const activeProfileId = computed(() => {
    const explicitActive = getActiveProfileId()
    if (explicitActive) return explicitActive
    if (!currentCredentials.value) return null
    return buildProfileId(currentCredentials.value)
  })

  const activeProfileLabel = computed(() => {
    const active = profiles.value.find(profile => profile.id === activeProfileId.value)
    if (active) return active.label
    if (currentCredentials.value?.username)
      return `${currentCredentials.value.username} (${currentCredentials.value.provider ?? 'jellyfin'})`
    return 'Account'
  })

  const refreshProfiles = (): void => {
    profiles.value = loadProfiles()
  }

  onMounted(() => {
    refreshProfiles()
  })

  const switchProfile = async (profile: AuthProfile): Promise<void> => {
    if (profile.id === activeProfileId.value) return

    profileActionError.value = ''
    switchingProfileId.value = profile.id

    try {
      const saveResult = await apiClient.saveCredentials(profile.credentials)
      if (saveResult.status === 'error') {
        throw new Error(String(saveResult.error))
      }
      setActiveProfileId(profile.id)
      window.location.reload()
    } catch (error) {
      profileActionError.value = `Failed to switch profile: ${String(error)}`
    } finally {
      switchingProfileId.value = null
    }
  }

  const onProfileAdded = (): void => {
    profileActionError.value = ''
    refreshProfiles()
  }

  const handleLogout = async (): Promise<void> => {
    profileActionError.value = ''

    try {
      await apiClient.clearSavedCredentials()
    } catch {
      // Ignore backend logout errors and continue local logout.
    }

    setActiveProfileId(null)
    emit('logout')
  }

  const openServerSettings = (): void => {
    router.push('/settings')
  }
</script>

<template>
  <div
    :class="[
      sidebarBgClass,
      'sidebar flex shrink-0 overflow-visible transition-[width] duration-200 ease',
      'flex-col h-full',
      sidebarWidthClass,
    ]"
  >
    <div
      :class="['absolute z-10 top-12 pointer-events-none outer-shadow-right', shadowBottomClass]"
    />
    <div :class="['flex', 'flex-col', 'h-full', isMacos && 'pt-10']">
      <!-- Search -->
      <div class='m-2 mb-2'>
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
      <nav class='flex flex-col grow m-2 mt-0'>
        <div class='grow space-y-2'>
          <RouterLink
            :class="navItemClass('home')"
            to='/'
          >
            <div :class='navIconClass'>
              <Home class='size-5' />
            </div>
            <div
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
              :class="[
                'overflow-hidden transition-all duration-150 ease-in-out',
                props.isCollapsed ? 'max-w-0 opacity-0' : 'max-w-full opacity-100',
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
      </nav>

      <div class='m-2 mt-0 border-t border-border/20 pt-2'>
        <DropdownMenu>
          <DropdownMenuTrigger as-child>
            <button
              class='
                w-full h-10 rounded-md border border-border/40 bg-background/60
                hover:bg-accent/10 transition-colors flex items-center text-sm
              '
              type='button'
            >
              <div :class='navIconClass'>
                <Users class='size-5 text-muted-foreground' />
              </div>
              <div
                :class="[
                  'overflow-hidden transition-all duration-150 ease-in-out flex items-center justify-between w-full pr-2',
                  isCollapsed ? 'max-w-0 opacity-0' : 'max-w-full opacity-100',
                ]"
              >
                <span class='truncate text-left text-foreground/90'>
                  {{ activeProfileLabel }}
                </span>
                <ChevronsUpDown class='size-4 text-muted-foreground shrink-0 ml-2' />
              </div>
            </button>
          </DropdownMenuTrigger>

          <DropdownMenuContent
            :align='isCollapsed ? "start" : "end"'
            :side='isCollapsed ? "right" : "top"'
            class='w-72'
          >
            <DropdownMenuLabel>Profiles</DropdownMenuLabel>
            <DropdownMenuItem
              v-for='profile in profiles'
              @click='switchProfile(profile)'
              :disabled='switchingProfileId === profile.id || activeProfileId === profile.id'
              :key='profile.id'
              class='gap-2'
            >
              <RefreshCw v-if='switchingProfileId === profile.id' class='size-3 animate-spin' />
              <Check v-else-if='activeProfileId === profile.id' class='size-3' />
              <span v-else class='size-3' />
              <span class='truncate'>
                {{ profile.label }}
              </span>
            </DropdownMenuItem>
            <DropdownMenuItem v-if='profiles.length === 0' disabled>
              No saved profiles
            </DropdownMenuItem>

            <DropdownMenuSeparator />

            <DropdownMenuItem @click='showAddProfileDialog = true' class='gap-2'>
              <Plus class='size-3' />
              Add Profile
            </DropdownMenuItem>
            <DropdownMenuItem @click='openServerSettings' class='gap-2'>
              <Settings class='size-3' />
              Manage Profiles
            </DropdownMenuItem>

            <DropdownMenuSeparator />

            <DropdownMenuItem @click='handleLogout' class='gap-2 text-destructive focus:text-destructive'>
              <LogOut class='size-3' />
              Log Out
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>

        <p
          v-if='profileActionError && !isCollapsed'
          class='text-xs text-destructive mt-2 px-1'
        >
          {{ profileActionError }}
        </p>
      </div>
    </div>

    <AddProfileDialog
      @profile-added='onProfileAdded'
      @update:open='showAddProfileDialog = $event'
      :open='showAddProfileDialog'
    />
  </div>
</template>
