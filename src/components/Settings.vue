<template>
  <div class='p-8 space-y-6'>
    <div class='mb-8'>
      <h1 class='text-4xl font-bold mb-4'>
        Settings
      </h1>
    </div>

    <div class='space-y-4'>
      <h2 class='text-xl font-semibold'>
        Appearance
      </h2>
      <div class='p-4 border rounded-lg'>
        <label class='block text-sm font-medium mb-2'>Theme</label>
        <div class='flex items-center space-x-2'>
          <Button
            @click="mode = 'light'"
            :variant="mode === 'light' ? 'secondary' : 'ghost'"
            class='flex items-center space-x-2'
          >
            <Sun class='h-4 w-4' />
            <span>Light</span>
          </Button>
          <Button
            @click="mode = 'dark'"
            :variant="mode === 'dark' ? 'secondary' : 'ghost'"
            class='flex items-center space-x-2'
          >
            <Moon class='h-4 w-4' />
            <span>Dark</span>
          </Button>
          <Button
            @click="mode = 'auto'"
            :variant="mode === 'auto' ? 'secondary' : 'ghost'"
            class='flex items-center space-x-2'
          >
            <Laptop class='h-4 w-4' />
            <span>System</span>
          </Button>
        </div>
      </div>
    </div>

    <div class='space-y-4'>
      <h2 class='text-xl font-semibold'>
        Server
      </h2>
      <div class='p-4 border rounded-lg space-y-4'>
        <div>
          <label class='block text-sm font-medium text-muted-foreground'>Server URL</label>
          <p class='text-sm'>
            {{ credentials?.serverUrl || 'Not connected' }}
          </p>
        </div>
        <div>
          <label class='block text-sm font-medium text-muted-foreground'>Username</label>
          <p class='text-sm'>
            {{ credentials?.username || 'Not connected' }}
          </p>
        </div>
        <Button @click='handleLogout' :disabled='!credentials' variant='destructive'>
          Logout
        </Button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
  import { useColorMode } from '@vueuse/core'
  import { Sun, Moon, Laptop } from 'lucide-vue-next'
  import { Button } from '@/components/ui/button'

  const mode = useColorMode()

  interface Credentials {
    serverUrl: string
    username:  string
    token:     string
    userId:    string
  }

  defineProps<{
    credentials: Credentials | null
  }>()

  const emit = defineEmits<{
    (e: 'logout'): void
  }>()

  const handleLogout = () => {
    emit('logout')
  }
</script>
