<script setup lang="ts">
  import {
    Check,
    Link,
    LogOut,
    Plus,
    RefreshCw,
    Server,
    Trash2,
    User,
  } from 'lucide-vue-next'
  import { computed, onMounted, ref } from 'vue'

  import type { Credentials } from '../../generated'

  import { apiClient } from '../../api/apiClient'
  import {
    buildProfileId,
    getActiveProfileId,
    loadProfiles,
    removeProfile,
    setActiveProfileId,
    type AuthProfile,
  } from '../../lib/profileStorage'
  import AddProfileDialog from '../auth/AddProfileDialog.vue'
  import Button from '../ui/Button.vue'

  const props = defineProps<{
    credentials: Credentials | null
  }>()

  const emit = defineEmits<{
    (e: 'logout'): void
  }>()

  const aureliaServerUrl = ref('')
  const profileActionError = ref('')
  const profiles = ref<AuthProfile[]>([])
  const removingProfileId = ref<null | string>(null)
  const showAddProfileDialog = ref(false)
  let saveTimeout: null | ReturnType<typeof setTimeout> = null
  const switchingProfileId = ref<null | string>(null)

  const activeProfileId = computed(() => {
    const explicitActive = getActiveProfileId()
    if (explicitActive) return explicitActive
    if (!props.credentials) return null
    return buildProfileId(props.credentials)
  })

  const refreshProfiles = (): void => {
    profiles.value = loadProfiles()
  }

  onMounted(async () => {
    refreshProfiles()

    try {
      const result = await apiClient.getSetting('aurelia_server_url')
      if (result.status === 'ok' && result.data) {
        aureliaServerUrl.value = result.data
      }
    } catch {
      // Setting not found, leave empty
    }
  })

  const switchProfile = async (profile: AuthProfile): Promise<void> => {
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

  const removeSavedProfile = async (profile: AuthProfile): Promise<void> => {
    profileActionError.value = ''
    removingProfileId.value = profile.id

    try {
      const isActive = activeProfileId.value === profile.id
      const remaining = removeProfile(profile.id)
      profiles.value = remaining

      if (!isActive) {
        return
      }

      if (remaining.length === 0) {
        const clearResult = await apiClient.clearSavedCredentials()
        if (clearResult.status === 'error') {
          throw new Error(String(clearResult.error))
        }
        setActiveProfileId(null)
        emit('logout')
        return
      }

      const fallbackProfile = remaining[0]
      const saveResult = await apiClient.saveCredentials(fallbackProfile.credentials)
      if (saveResult.status === 'error') {
        throw new Error(String(saveResult.error))
      }

      setActiveProfileId(fallbackProfile.id)
      window.location.reload()
    } catch (error) {
      profileActionError.value = `Failed to remove profile: ${String(error)}`
      refreshProfiles()
    } finally {
      removingProfileId.value = null
    }
  }

  const onAureliaUrlInput = (event: Event): void => {
    const value = (event.target as HTMLInputElement).value
    aureliaServerUrl.value = value

    // Debounce save
    if (saveTimeout) clearTimeout(saveTimeout)
    saveTimeout = setTimeout(async () => {
      try {
        if (value.trim()) {
          await apiClient.saveSetting('aurelia_server_url', value.trim())
        } else {
          await apiClient.deleteSetting('aurelia_server_url')
        }
      } catch {
        // Ignore save errors
      }
    }, 500)
  }

  const onProfileAdded = (): void => {
    refreshProfiles()
  }

  const handleLogout = async (): Promise<void> => {
    try {
      await apiClient.clearSavedCredentials()
    } catch {
      // Ignore backend logout errors and continue local logout.
    }
    setActiveProfileId(null)
    emit('logout')
  }
</script>

<template>
  <div class='space-y-8'>
    <!-- Connection Status -->
    <div class='space-y-6'>
      <!-- Connection Status -->
      <div
        class='
          flex items-center space-x-3 p-4 bg-background/40 rounded-lg
          border border-border/20
        '
      >
        <div :class='credentials ? "bg-green-500" : "bg-red-500"' class='size-3 rounded-full' />
        <div>
          <p class='font-medium'>
            {{ credentials ? 'Connected' : 'Not Connected' }}
          </p>
          <p class='text-sm text-muted-foreground'>
            {{ credentials ? 'Server connection active' : 'No server connection' }}
          </p>
        </div>
      </div>

      <!-- Server Info -->
      <div class='grid md:grid-cols-2 gap-6'>
        <div class='space-y-2'>
          <label class='text-sm font-medium text-muted-foreground flex items-center space-x-2'>
            <Link class='size-4' />
            <span>Media Server</span>
          </label>
          <p
            class='
              text-sm font-mono bg-background/40 p-3 rounded-lg
              border border-border/20
            '
          >
            {{ credentials?.serverUrl || 'Not connected' }}
          </p>
        </div>
        <div class='space-y-2'>
          <label class='text-sm font-medium text-muted-foreground flex items-center space-x-2'>
            <User class='size-4' />
            <span>Username</span>
          </label>
          <p
            class='
              text-sm bg-background/40 p-3 rounded-lg
              border border-border/20
            '
          >
            {{ credentials?.username || 'Not connected' }}
          </p>
        </div>
      </div>

      <!-- Aurelia Server URL -->
      <div class='space-y-2'>
        <label class='text-sm font-medium text-muted-foreground flex items-center space-x-2'>
          <Server class='size-4' />
          <span>Aurelia Server</span>
        </label>
        <input
          @input='onAureliaUrlInput'
          :value='aureliaServerUrl'
          class='
            w-full text-sm font-mono bg-background/40 p-3 rounded-lg
            border border-border/20 outline-none
            focus:border-primary/50 transition-colors
          '
          placeholder='https://aurelia.example.com'
          type='url'
        >
        <p class='text-xs text-muted-foreground'>
          URL of your Aurelia web server for synced lyrics from sidecar files. Leave empty if not using one.
        </p>
      </div>

      <!-- Saved Profiles -->
      <div class='space-y-3'>
        <div class='flex items-center justify-between'>
          <label class='text-sm font-medium text-muted-foreground'>
            Saved Profiles
          </label>
          <Button
            @click='showAddProfileDialog = true'
            size='sm'
            variant='outline'
          >
            <Plus class='size-3 mr-1' />
            Add Profile
          </Button>
        </div>
        <div class='space-y-2'>
          <div
            v-for='profile in profiles'
            :key='profile.id'
            class='
              border border-border/20 rounded-lg p-3
              flex items-center justify-between gap-3
            '
          >
            <div class='min-w-0'>
              <p class='text-sm font-medium truncate'>
                {{ profile.label }}
              </p>
              <p class='text-xs text-muted-foreground truncate'>
                {{ profile.credentials.serverUrl }}
              </p>
            </div>

            <div class='flex items-center gap-2'>
              <Button
                @click='switchProfile(profile)'
                :disabled='switchingProfileId === profile.id || activeProfileId === profile.id'
                size='sm'
                variant='outline'
              >
                <RefreshCw v-if='switchingProfileId === profile.id' class='size-3 mr-1 animate-spin' />
                <Check v-else-if='activeProfileId === profile.id' class='size-3 mr-1' />
                {{ activeProfileId === profile.id ? 'Active' : 'Switch' }}
              </Button>
              <Button
                @click='removeSavedProfile(profile)'
                :disabled='removingProfileId === profile.id'
                size='sm'
                variant='destructive'
              >
                <Trash2 v-if='removingProfileId !== profile.id' class='size-3 mr-1' />
                {{ removingProfileId === profile.id ? 'Removing...' : 'Remove' }}
              </Button>
            </div>
          </div>
          <p v-if='profiles.length === 0' class='text-xs text-muted-foreground'>
            No saved profiles yet. Add one to enable quick switching.
          </p>
          <p v-if='profileActionError' class='text-xs text-destructive'>
            {{ profileActionError }}
          </p>
        </div>
      </div>

      <!-- Actions -->
      <div class='flex justify-end pt-2 border-t border-border/20'>
        <Button
          @click='handleLogout'
          :disabled='!credentials'
          class='px-6'
          variant='destructive'
        >
          <LogOut class='size-4 mr-2' />
          Logout
        </Button>
      </div>
    </div>

    <AddProfileDialog
      @profile-added='onProfileAdded'
      @update:open='showAddProfileDialog = $event'
      :open='showAddProfileDialog'
    />
  </div>
</template>
