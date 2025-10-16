<script setup lang="ts">
  import { getVersion } from '@tauri-apps/api/app'
  import { type, version as osVersion } from '@tauri-apps/plugin-os'
  import { open } from '@tauri-apps/plugin-opener'
  import {
    ExternalLink,
    Github,
    Heart,
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
    } catch (error) {
      appVersion.value = 'Unknown'
      platformInfo.value = 'Unknown'
    }
  })

  const openLink = async (url: string): Promise<void> => {
    await open(url)
  }

  const techStack = [
    { name: 'Vue 3', description: 'Progressive JavaScript framework' },
    { name: 'TypeScript', description: 'Typed superset of JavaScript' },
    { name: 'Tauri', description: 'Desktop application framework' },
    { name: 'Pinia', description: 'State management' },
    { name: 'Tailwind CSS', description: 'Utility-first CSS framework' },
    { name: 'shadcn-vue', description: 'UI component library' },
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
        <div class='flex items-start space-x-4'>
          <div class='flex-shrink-0 size-16 bg-gradient-to-br from-primary to-accent rounded-xl flex items-center justify-center text-2xl font-bold text-primary-foreground'>
            A
          </div>
          <div class='flex-1'>
            <h3 class='text-2xl font-bold'>
              Aurelia
            </h3>
            <p class='text-sm text-muted-foreground mt-1'>
              A modern desktop music player for Jellyfin
            </p>
            <div class='flex items-center space-x-4 mt-3 text-sm'>
              <div>
                <span class='text-muted-foreground'>Version:</span>
                <span class='ml-2 font-mono'>{{ appVersion }}</span>
              </div>
            </div>
          </div>
        </div>

        <div class='p-4 bg-popover rounded-lg border border-border/30'>
          <div class='text-sm space-y-2'>
            <div class='flex items-center justify-between'>
              <span class='text-muted-foreground'>Platform:</span>
              <span class='font-mono'>{{ platformInfo }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Links -->
      <div class='space-y-3'>
        <h3 class='text-sm font-medium text-muted-foreground'>
          Links
        </h3>
        <div class='grid gap-3'>
          <Button
            @click='openLink("https://github.com/pupbrained/aurelia")'
            variant='outline'
            class='w-full justify-between'
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
            class='w-full justify-between'
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

      <!-- Credits -->
      <div class='space-y-3'>
        <h3 class='text-sm font-medium text-muted-foreground'>
          Acknowledgments
        </h3>
        <div class='p-4 bg-popover rounded-lg border border-border/30 space-y-2'>
          <div class='flex items-start space-x-2 text-sm'>
            <Heart class='size-4 text-red-500 flex-shrink-0 mt-0.5' />
            <div>
              <p class='text-muted-foreground'>
                Built with
                <Button
                  @click='openLink("https://tauri.app")'
                  variant='link'
                  class='h-auto p-0 text-accent hover:text-accent/80'
                >
                  Tauri
                </Button>
              </p>
              <p class='text-muted-foreground mt-1'>
                UI inspired by
                <Button
                  @click='openLink("https://www.shadcn-vue.com")'
                  variant='link'
                  class='h-auto p-0 text-accent hover:text-accent/80'
                >
                  shadcn-vue
                </Button>
              </p>
            </div>
          </div>
        </div>
      </div>

      <!-- Copyright -->
      <div class='pt-4 border-t border-border/30'>
        <p class='text-xs text-center text-muted-foreground'>
          © {{ new Date().getFullYear() }} Aurelia. All rights reserved.
        </p>
      </div>
    </div>
  </div>
</template>
