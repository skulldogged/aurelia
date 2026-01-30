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

  import Button from '../ui/Button.vue'

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
  <div class='space-y-8'>
    <!-- App Info -->
    <div class='space-y-8'>
      <!-- App Info -->
      <div class='space-y-4'>
        <div class='flex items-center space-x-6'>
          <div
            class='
              shrink-0 size-24 bg-linear-to-br from-primary to-accent rounded-xl
              flex items-center justify-center text-5xl font-bold text-primary-foreground
            '
          >
            A
          </div>
          <div class='flex-1'>
            <h3 class='text-3xl font-bold'>
              Aurelia
            </h3>
            <p class='text-base text-muted-foreground mt-2'>
              A modern desktop music player for Jellyfin
            </p>
            <div class='flex items-center space-x-6 mt-3 text-sm'>
              <div>
                <span class='text-muted-foreground'>Version:</span>
                <span class='ml-2 font-mono font-medium'>{{ appVersion }}</span>
              </div>
              <div>
                <span class='text-muted-foreground'>Platform:</span>
                <span class='ml-2 font-mono font-medium'>{{ platformInfo }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Links -->
      <div class='space-y-4 pt-4 border-t border-border/20'>
        <h3 class='text-lg font-medium'>
          Links
        </h3>
        <div class='grid md:grid-cols-2 gap-4'>
          <Button
            @click='openLink("https://github.com/pupbrained/aurelia")'
            class='justify-between h-auto p-4'
            variant='outline'
          >
            <span class='flex items-center space-x-3'>
              <Github class='size-5' />
              <span>GitHub Repository</span>
            </span>
            <ExternalLink class='size-4' />
          </Button>
          <Button
            @click='openLink("https://github.com/pupbrained/aurelia/issues")'
            class='justify-between h-auto p-4'
            variant='outline'
          >
            <span class='flex items-center space-x-3'>
              <Info class='size-5' />
              <span>Report an Issue</span>
            </span>
            <ExternalLink class='size-4' />
          </Button>
        </div>
      </div>

      <!-- Tech Stack -->
      <div class='space-y-4 pt-4 border-t border-border/20'>
        <h3 class='text-lg font-medium'>
          Built With
        </h3>
        <div class='grid md:grid-cols-2 lg:grid-cols-3 gap-4'>
          <div
            v-for='tech in techStack'
            :key='tech.name'
            class='p-4 bg-background/40 rounded-lg border border-border/20 hover:border-border/40 transition-colors'
          >
            <div class='font-medium text-base'>
              {{ tech.name }}
            </div>
            <div class='text-sm text-muted-foreground mt-2'>
              {{ tech.description }}
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
