<script setup lang="ts">
  import {
    AlertTriangle,
    HardDrive,
    RefreshCw,
    Trash2,
  } from 'lucide-vue-next'

  import Button from '@/components/ui/Button.vue'

  defineEmits<{
    (e: 'sync-library'): void
    (e: 'clear-cache'): void
  }>()

  defineProps<{
    isClearing: boolean
    isSyncing:  boolean
  }>()
</script>

<template>
  <div class='space-y-6'>
    <!-- Header -->
    <div class='flex items-center space-x-3 pb-4 border-b border-border/30'>
      <div class='p-2 bg-accent/10 rounded-lg'>
        <HardDrive class='size-5 text-accent' />
      </div>
      <div>
        <h2 class='text-xl font-semibold'>
          Library Management
        </h2>
        <p class='text-sm text-muted-foreground'>
          Sync or clear your local music library cache
        </p>
      </div>
    </div>

    <!-- Content -->
    <div class='space-y-6'>
      <!-- Sync Library Card -->
      <div
        class='
          bg-background/40 border border-border/20 rounded-lg p-6
          hover:border-border/40 transition-colors
        '
      >
        <div class='flex items-start space-x-4'>
          <div class='p-3 bg-primary/10 rounded-lg shrink-0'>
            <RefreshCw class='size-6 text-primary' />
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
              <RefreshCw :class="{'animate-spin': isSyncing}" class='size-4 mr-2' />
              {{ isSyncing ? 'Syncing...' : 'Sync Library' }}
            </Button>
          </div>
        </div>
      </div>

      <!-- Clear Cache Card -->
      <div
        class='
          bg-background/40 border border-border/20 rounded-lg p-6
          hover:border-border/40 transition-colors
        '
      >
        <div class='flex items-start space-x-4'>
          <div class='p-3 bg-accent/10 rounded-lg shrink-0'>
            <AlertTriangle class='size-6 text-accent' />
          </div>
          <div class='flex-1'>
            <h3 class='text-lg font-medium mb-2'>
              Clear Music Library Cache
            </h3>
            <p class='text-sm text-muted-foreground mb-4'>
              Remove all cached data and refresh from the server. Your playlists and personal
              settings remain unchanged.
            </p>
            <Button
              @click='$emit("clear-cache")'
              :disabled='isClearing'
              class='px-6'
              variant='destructive'
            >
              <Trash2 class='size-4 mr-2' />
              {{ isClearing ? 'Clearing...' : 'Clear Cache' }}
            </Button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
