<template>
  <!-- Server Section -->
  <section class='space-y-6'>
    <div class='flex items-center space-x-3'>
      <div class='p-2 bg-primary/10 rounded-lg'>
        <Server class='w-5 h-5 text-primary' />
      </div>
      <h2 class='text-2xl font-semibold'>
        Server Connection
      </h2>
    </div>

    <div class='bg-card/50 backdrop-blur-sm border border-border/50 rounded-xl p-6 shadow-lg'>
      <div class='space-y-6'>
        <!-- Connection Status -->
        <div class='flex items-center space-x-3 p-4 bg-background/50 rounded-lg border border-border/30'>
          <div :class='credentials ? "bg-green-500" : "bg-red-500"' class='w-3 h-3 rounded-full' />
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
              <Link class='w-4 h-4' />
              <span>Server URL</span>
            </label>
            <p class='text-sm font-mono bg-background/50 p-3 rounded-lg border border-border/30'>
              {{ credentials?.serverUrl || 'Not connected' }}
            </p>
          </div>
          <div class='space-y-2'>
            <label class='text-sm font-medium text-muted-foreground flex items-center space-x-2'>
              <User class='w-4 h-4' />
              <span>Username</span>
            </label>
            <p class='text-sm bg-background/50 p-3 rounded-lg border border-border/30'>
              {{ credentials?.username || 'Not connected' }}
            </p>
          </div>
        </div>

        <!-- Actions -->
        <div class='flex justify-end pt-4 border-t border-border/30'>
          <Button
            @click='$emit("logout")'
            :disabled='!credentials'
            class='px-6'
            variant='destructive'
          >
            <LogOut class='w-4 h-4 mr-2' />
            Logout
          </Button>
        </div>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
  import { Button } from '@/components/ui/button'
  import {
    Server,
    Link,
    User,
    LogOut,
  } from 'lucide-vue-next'

  interface Credentials {
    serverUrl: string
    username:  string
    token:     string
    userId:    string
  }

  defineProps<{
    credentials: Credentials | null
  }>()

  defineEmits<{
    (e: 'logout'): void
  }>()
</script>
