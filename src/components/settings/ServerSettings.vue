<script setup lang="ts">
  import {
    Link,
    LogOut,
    Server,
    User,
  } from 'lucide-vue-next'

  import Button from '@/components/ui/Button.vue'

  interface Credentials {
    serverUrl: string
    token:     string
    userId:    string
    username:  string
  }

  defineProps<{
    credentials: Credentials | null
  }>()

  defineEmits<{
    (e: 'logout'): void
  }>()
</script>

<template>
  <div class='space-y-6'>
    <!-- Header -->
    <div class='flex items-center space-x-3 pb-4 border-b border-border/30'>
      <div class='p-2 bg-primary/10 rounded-lg'>
        <Server class='size-5 text-primary' />
      </div>
      <div>
        <h2 class='text-xl font-semibold'>
          Server Connection
        </h2>
        <p class='text-sm text-muted-foreground'>
          Manage your Jellyfin server connection and credentials
        </p>
      </div>
    </div>

    <!-- Content -->
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
            <span>Server URL</span>
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

      <!-- Actions -->
      <div class='flex justify-end pt-2'>
        <Button
          @click='$emit("logout")'
          :disabled='!credentials'
          class='px-6'
          variant='destructive'
        >
          <LogOut class='size-4 mr-2' />
          Logout
        </Button>
      </div>
    </div>
  </div>
</template>
