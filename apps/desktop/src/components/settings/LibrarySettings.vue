<script setup lang="ts">
  import {
    AlertTriangle,
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
  <div class='space-y-8'>
    <!-- Sync Library Card -->
    <div class='space-y-6'>
      <!-- Sync Library Card -->
      <div
        class='
          bg-background/40 border border-border/20 rounded-lg p-8
          hover:border-border/40 transition-colors
        '
      >
        <div class='flex items-start space-x-6'>
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
          bg-background/40 border border-border/20 rounded-lg p-8
          hover:border-border/40 transition-colors
        '
      >
        <div class='flex items-start space-x-6'>
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
