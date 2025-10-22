<script setup lang="ts">
  import { getVersion } from '@tauri-apps/api/app'
  import { openUrl } from '@tauri-apps/plugin-opener'
  import { version as osVersion, type } from '@tauri-apps/plugin-os'
  import {
    ExternalLink,
    Github,
    Info,
  } from 'lucide-vue-next'
  import { onMounted, ref } from 'vue'

  import Button from '@/components/ui/Button.vue'

  const appVersion = ref<string>('Loading...')
  const platformInfo = ref<string>('Loading...')

  onMounted(async () => {
    try {
      appVersion.value = await getVersion()
      const osType = type()
      const osVer = osVersion()
      platformInfo.value = `${osType} ${osVer}`
    } catch {
      appVersion.value = 'Unknown'
      platformInfo.value = 'Unknown'
    }
  })

  const openLink = async (url: string): Promise<void> => {
    await openUrl(url)
  }

  const techStack = [
    { description: 'Progressive JavaScript framework', name: 'Vue 3' },
    { description: 'Typed superset of JavaScript', name: 'TypeScript' },
    { description: 'Desktop application framework', name: 'Tauri' },
    { description: 'State management', name: 'Pinia' },
    { description: 'Utility-first CSS framework', name: 'Tailwind CSS' },
    { description: 'UI component library', name: 'shadcn-vue' },
  ]
</script>

<template>
  <div class='bg-sidebar rounded-lg'>
    <!-- Header -->
    <div class='p-6'>
      <div class='flex items-center space-x-3'>
        <div class='p-2 bg-primary/10 rounded-lg'>
          <Info class='size-5 text-primary' />
        </div>
        <h2 class='text-2xl font-semibold'>
          About
        </h2>
      </div>
      <p class='text-sm text-muted-foreground mt-2'>
        Information about Aurelia and its components
      </p>
    </div>

    <!-- Content -->
    <div class='p-6 space-y-6'>
      <!-- App Info -->
      <div class='space-y-4'>
        <div class='flex items-center space-x-4'>
          <div
            class='shrink-0 size-20 bg-linear-to-br from-primary to-accent rounded-xl
                   flex items-center justify-center text-4xl font-bold text-primary-foreground'
          >
            A
          </div>
          <div class='flex-1'>
            <h3 class='text-2xl font-bold'>
              Aurelia
            </h3>
            <p class='text-sm text-muted-foreground mt-1'>
              A modern desktop music player for Jellyfin
            </p>
            <div class='flex items-center space-x-4 mt-1 text-sm'>
              <div>
                <span class='text-muted-foreground'>Version:</span>
                <span class='ml-2 font-mono'>{{ appVersion }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Links -->
      <div class='space-y-3'>
        <h3 class='text-sm font-medium text-muted-foreground'>
          Links
        </h3>
        <div class='grid md:grid-cols-2 gap-3'>
          <Button
            @click='openLink("https://github.com/pupbrained/aurelia")'
            variant='outline'
          >
            <span class='flex items-center space-x-2'>
              <Github class='size-4' />
              <span>GitHub Repository</span>
            </span>
            <ExternalLink class='size-4' />
          </Button>
          <Button
            @click='openLink("https://github.com/pupbrained/aurelia/issues")'
            variant='outline'
          >
            <span class='flex items-center space-x-2'>
              <Info class='size-4' />
              <span>Report an Issue</span>
            </span>
            <ExternalLink class='size-4' />
          </Button>
        </div>
      </div>

      <!-- Tech Stack -->
      <div class='space-y-3'>
        <h3 class='text-sm font-medium text-muted-foreground'>
          Built With
        </h3>
        <div class='grid md:grid-cols-2 gap-3'>
          <div
            v-for='tech in techStack'
            :key='tech.name'
            class='p-3 bg-popover rounded-lg border border-border/30'
          >
            <div class='font-medium text-sm'>
              {{ tech.name }}
            </div>
            <div class='text-xs text-muted-foreground mt-1'>
              {{ tech.description }}
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
