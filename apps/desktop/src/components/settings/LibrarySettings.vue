<script setup lang="ts">
  import { computed } from 'vue'
  import {
    AlertTriangle,
    Clock,
    RefreshCw,
    Trash2,
  } from 'lucide-vue-next'

  import Button from '@/components/ui/Button.vue'

  defineEmits<{
    (e: 'sync-library'): void
    (e: 'clear-cache'): void
  }>()

  const props = defineProps<{
    isClearing: boolean
    isSyncing:  boolean
    lastSyncTime?: string | null
  }>()

  // Compute relative time string from ISO timestamp
  const lastSyncedDisplay = computed(() => {
    if (!props.lastSyncTime) return null
    
    const syncDate = new Date(props.lastSyncTime)
    const now = new Date()
    const diffMs = now.getTime() - syncDate.getTime()
    const diffMins = Math.floor(diffMs / 60000)
    const diffHours = Math.floor(diffMs / 3600000)
    const diffDays = Math.floor(diffMs / 86400000)
    
    if (diffMins < 1) return 'Just now'
    if (diffMins < 60) return `${diffMins} minute${diffMins === 1 ? '' : 's'} ago`
    if (diffHours < 24) return `${diffHours} hour${diffHours === 1 ? '' : 's'} ago`
    if (diffDays < 7) return `${diffDays} day${diffDays === 1 ? '' : 's'} ago`
    
    return syncDate.toLocaleDateString()
  })
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
            <p class='text-sm text-muted-foreground mb-4'>
              Update your local music library with the latest data from Jellyfin server.
              This will add new songs and update existing metadata without removing your current data.
            </p>
            <div class='flex items-center gap-4'>
              <Button
                @click='$emit("sync-library")'
                :disabled='isSyncing'
                class='px-6'
                variant='default'
              >
                <RefreshCw :class="{'animate-spin': isSyncing}" class='size-4 mr-2' />
                {{ isSyncing ? 'Syncing...' : 'Sync Library' }}
              </Button>
              <span v-if='lastSyncedDisplay' class='text-sm text-muted-foreground flex items-center gap-1.5'>
                <Clock class='size-3.5' />
                Last synced {{ lastSyncedDisplay }}
              </span>
            </div>
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
