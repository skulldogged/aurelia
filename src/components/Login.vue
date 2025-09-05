<template>
  <div class="h-full bg-background flex items-center justify-center p-4">
    <div class="max-w-md w-full p-8">
      <div class="text-center mb-8">
        <h1 class="text-3xl font-bold text-foreground mb-2">Jellyfin Music Player</h1>
        <p class="text-muted-foreground">Connect to your Jellyfin server</p>
      </div>

      <form @submit.prevent="handleLogin" class="space-y-6">
        <div class="grid w-full items-center gap-1.5">
          <Label for="serverUrl">Server URL</Label>
          <Input id="serverUrl" v-model="form.serverUrl" type="url" required placeholder="https://your-server.com" />
        </div>

        <div class="grid w-full items-center gap-1.5">
          <Label for="username">Username</Label>
          <Input id="username" v-model="form.username" type="text" required placeholder="Enter your username" />
        </div>

        <div class="grid w-full items-center gap-1.5">
          <Label for="password">Password</Label>
          <Input id="password" v-model="form.password" type="password" required placeholder="Enter your password" />
        </div>

        <div class="flex items-center space-x-2">
          <Checkbox id="remember" v-model:checked="form.remember" />
          <Label for="remember">Remember me</Label>
        </div>

        <Button type="submit" :disabled="loading" class="w-full">
          <Loader2 v-if="loading" class="mr-2 h-4 w-4 animate-spin" />
          Connect
        </Button>
      </form>

      <div v-if="error" class="mt-4 p-3 bg-destructive/10 border border-destructive/20 rounded-md">
        <p class="text-destructive-foreground text-sm">{{ error }}</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { Loader2 } from 'lucide-vue-next'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Checkbox } from '@/components/ui/checkbox'

interface LoginForm {
  serverUrl: string
  username: string
  password: string
  remember: boolean
}

const form = ref<LoginForm>({
  serverUrl: '',
  username: '',
  password: '',
  remember: true
})

const loading = ref(false)
const error = ref('')

const emit = defineEmits<{
  login: [credentials: { serverUrl: string; username: string; token: string; userId: string }]
}>()

// Load saved credentials on component mount
onMounted(async () => {
  try {
    const saved = await invoke<{ serverUrl: string; username: string; token: string }>('get_saved_credentials')
    if (saved) {
      form.value.serverUrl = saved.serverUrl
      form.value.username = saved.username
      // We can't auto-login here because we don't have the user ID from the saved credentials.
      // The user will need to re-enter their password to log in and get a new token/userId.
    }
  } catch (err) {
    // No saved credentials, continue normally
  }
})

const handleLogin = async () => {
  loading.value = true
  error.value = ''

  try {
    const result = await invoke<{ token: string; userId: string }>('login_to_jellyfin', {
      serverUrl: form.value.serverUrl,
      username: form.value.username,
      password: form.value.password,
    })

    if (form.value.remember) {
      await invoke('save_credentials', {
        serverUrl: form.value.serverUrl,
        username: form.value.username,
        token: result.token,
        userId: result.userId,
      })
    }

    emit('login', {
      serverUrl: form.value.serverUrl,
      username: form.value.username,
      token: result.token,
      userId: result.userId,
    })
  } catch (err) {
    error.value = err as string
  } finally {
    loading.value = false
  }
}
</script>
