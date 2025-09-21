<template>
  <!-- Library Management Section -->
  <section class='space-y-6'>
    <div class='flex items-center space-x-3'>
      <div class='p-2 bg-accent/10 rounded-lg'>
        <HardDrive class='w-5 h-5 text-accent' />
      </div>
      <h2 class='text-2xl font-semibold'>
        Library Management
      </h2>
    </div>

    <!-- Sync Library Card -->
    <div class='bg-card/50 backdrop-blur-sm border border-border/50 rounded-xl p-6 shadow-lg'>
      <div class='flex items-start space-x-4'>
        <div class='p-3 bg-primary/10 rounded-lg flex-shrink-0'>
          <RefreshCw class='w-6 h-6 text-primary' />
        </div>
        <div class='flex-1'>
          <h3 class='text-lg font-medium mb-2'>
            Sync Music Library
          </h3>
          <p class='text-sm text-muted-foreground mb-6'>
            Update your local music library with the latest data from Jellyfin server.
            This will add new songs and update existing metadata without removing your current data.
          </p>
          <Button
            @click='$emit("sync-library")'
            :disabled='isSyncing'
            class='px-6'
            variant='default'
          >
            <RefreshCw :class="{'animate-spin': isSyncing}" class='w-4 h-4 mr-2' />
            {{ isSyncing ? 'Syncing...' : 'Sync Library' }}
          </Button>
        </div>
      </div>
    </div>

    <!-- Clear Cache Card -->
    <div class='bg-card/50 backdrop-blur-sm border border-border/50 rounded-xl p-6 shadow-lg'>
      <div class='flex items-start space-x-4'>
        <div class='p-3 bg-accent/10 rounded-lg flex-shrink-0'>
          <AlertTriangle class='w-6 h-6 text-accent' />
        </div>
        <div class='flex-1'>
          <h3 class='text-lg font-medium mb-2'>
            Clear Music Library Cache
          </h3>
          <p class='text-sm text-muted-foreground mb-6'>
            This action will clear your local music library cache and refresh all music data from the server.
            Your playlists and settings will remain unchanged.
          </p>
          <Button
            @click='$emit("clear-cache")'
            :disabled='isClearing'
            class='px-6'
            variant='destructive'
          >
            <Trash2 class='w-4 h-4 mr-2' />
            {{ isClearing ? 'Clearing...' : 'Clear Cache' }}
          </Button>
        </div>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
  import { Button } from '@/components/ui/button'
  import {
    HardDrive,
    AlertTriangle,
    Trash2,
    RefreshCw,
  } from 'lucide-vue-next'

  defineEmits<{
    (e: 'sync-library'): void
    (e: 'clear-cache'): void
  }>()

  defineProps<{
    isSyncing:  boolean
    isClearing: boolean
  }>()
</script>
