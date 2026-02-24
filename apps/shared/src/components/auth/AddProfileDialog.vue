<script setup lang="ts">
  import { Loader2 } from 'lucide-vue-next'
  import { computed, ref, watch } from 'vue'

  import type { BackendProvider } from '../../generated'

  import { ApiError } from '../../effect/errors'
  import { runAureliaEffect } from '../../effect/runtime'
  import { authenticateEffect, detectProviderEffect, saveCredentialsEffect } from '../../effect/services/api'
  import { isTauri } from '../../lib/platform'
  import { setActiveProfileId, type AuthProfile, upsertProfile } from '../../lib/profileStorage'
  import Button from '../ui/Button.vue'
  import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '../ui/dialog'
  import { Input } from '../ui/input'
  import Label from '../ui/Label.vue'

  interface AddProfileForm {
    password:  string
    serverUrl: string
    username:  string
  }

  const props = withDefaults(defineProps<{
    open: boolean
    switchOnAdd?: boolean
  }>(), {
    switchOnAdd: false,
  })

  const emit = defineEmits<{
    (e: 'profile-added', profile: AuthProfile): void
    (e: 'update:open', open: boolean): void
  }>()

  const form = ref<AddProfileForm>({
    password:  '',
    serverUrl: '',
    username:  '',
  })

  const loading = ref(false)
  const detectingProvider = ref(false)
  const detectedProvider = ref<BackendProvider | null>(null)
  const providerSelection = ref<'auto' | BackendProvider>('auto')
  const error = ref('')

  const open = computed({
    get: () => props.open,
    set: (value: boolean) => emit('update:open', value),
  })

  const getDeviceId = (): string => {
    let deviceId = localStorage.getItem('aurelia-device-id')
    if (!deviceId) {
      deviceId = `profile-${Date.now()}-${Math.random().toString(36).slice(2, 11)}`
      localStorage.setItem('aurelia-device-id', deviceId)
    }
    const appLabel = isTauri() ? 'desktop' : 'web'
    return `aurelia-${appLabel}-${deviceId}`
  }

  const detectProvider = async (): Promise<void> => {
    if (!form.value.serverUrl.trim()) return
    detectingProvider.value = true
    try {
      detectedProvider.value = await runAureliaEffect(detectProviderEffect(form.value.serverUrl))
    } catch {
      detectedProvider.value = null
    } finally {
      detectingProvider.value = false
    }
  }

  const resolveProvider = (): BackendProvider =>
    providerSelection.value === 'auto'
      ? (detectedProvider.value ?? 'jellyfin')
      : providerSelection.value

  const resetForm = (): void => {
    form.value = {
      password:  '',
      serverUrl: '',
      username:  '',
    }
    providerSelection.value = 'auto'
    detectedProvider.value = null
    error.value = ''
    loading.value = false
    detectingProvider.value = false
  }

  watch(() => props.open, isOpen => {
    if (!isOpen) resetForm()
  })

  const addProfile = async (): Promise<void> => {
    error.value = ''
    loading.value = true

    try {
      if (providerSelection.value === 'auto' && !detectedProvider.value) {
        await detectProvider()
      }

      const credentials = await runAureliaEffect(authenticateEffect({
        deviceId:  getDeviceId(),
        password:  form.value.password,
        provider:  resolveProvider(),
        serverUrl: form.value.serverUrl,
        username:  form.value.username,
      }))

      if (props.switchOnAdd) {
        await runAureliaEffect(saveCredentialsEffect(credentials))
      }

      const profile = upsertProfile(credentials)
      if (props.switchOnAdd) {
        setActiveProfileId(profile.id)
      }
      emit('profile-added', profile)
      open.value = false
    } catch (cause) {
      const message = cause instanceof ApiError ? cause.message : String(cause)
      error.value = `Failed to add profile: ${message}`
    } finally {
      loading.value = false
    }
  }
</script>

<template>
  <Dialog v-model:open='open'>
    <DialogContent>
      <DialogHeader>
        <DialogTitle>Add Profile</DialogTitle>
        <DialogDescription>
          Sign in to add another provider profile.
        </DialogDescription>
      </DialogHeader>

      <div class='space-y-4'>
        <div class='grid gap-2'>
          <Label for='add-profile-provider'>Provider</Label>
          <div class='flex gap-2'>
            <select
              id='add-profile-provider'
              v-model='providerSelection'
              class='w-full border border-input bg-background rounded-md h-10 px-3'
            >
              <option value='auto'>Auto-detect</option>
              <option value='jellyfin'>Jellyfin</option>
              <option value='navidrome'>Navidrome</option>
            </select>
            <Button
              @click='detectProvider'
              :disabled='detectingProvider || !form.serverUrl'
              type='button'
              variant='outline'
            >
              {{ detectingProvider ? 'Detecting...' : 'Detect' }}
            </Button>
          </div>
          <p v-if='detectedProvider' class='text-xs text-muted-foreground'>
            Detected: {{ detectedProvider }}
          </p>
        </div>

        <div class='grid gap-2'>
          <Label for='add-profile-server-url'>Server URL</Label>
          <Input
            id='add-profile-server-url'
            v-model='form.serverUrl'
            placeholder='https://your-server.com'
            required
            type='url'
          />
        </div>

        <div class='grid gap-2'>
          <Label for='add-profile-username'>Username</Label>
          <Input
            id='add-profile-username'
            v-model='form.username'
            placeholder='Enter your username'
            required
            type='text'
          />
        </div>

        <div class='grid gap-2'>
          <Label for='add-profile-password'>Password</Label>
          <Input
            id='add-profile-password'
            v-model='form.password'
            autocomplete='current-password'
            placeholder='Enter your password'
            required
            type='password'
          />
        </div>

        <p v-if='error' class='text-sm text-destructive'>
          {{ error }}
        </p>
      </div>

      <DialogFooter>
        <Button @click='open = false' type='button' variant='outline'>
          Cancel
        </Button>
        <Button @click='addProfile' :disabled='loading' type='button'>
          <Loader2 v-if='loading' class='mr-2 h-4 w-4 animate-spin' />
          Add Profile
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
