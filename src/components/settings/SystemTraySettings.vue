<script setup lang="ts">
  import {
    Minimize2,
    Monitor,
  } from 'lucide-vue-next'

  import { Label } from '@/components/ui/label'
  import { useSystemTray } from '@/composables/useSystemTray'
  import { useSystemTrayStore } from '@/stores'

  const { setCloseToTray, setMinimizeToTray } = useSystemTray()

  const systemTrayStore = useSystemTrayStore()

  const handleMinimizeToggle = async (checked: boolean): Promise<void> => {
    systemTrayStore.minimizeToTray = checked
    await setMinimizeToTray(checked)
  }

  const handleCloseToggle = async (checked: boolean): Promise<void> => {
    systemTrayStore.closeToTray = checked
    await setCloseToTray(checked)
  }
</script>

<template>
  <section class='space-y-6'>
    <div class='flex items-center space-x-3'>
      <div class='p-2 bg-accent/10 rounded-lg'>
        <Monitor class='size-5 text-accent' />
      </div>
      <h2 class='text-2xl font-semibold'>
        System Tray
      </h2>
    </div>

    <div class='bg-card/50 backdrop-blur-sm border border-border/50 rounded-xl p-6 shadow-lg'>
      <div class='flex items-center space-x-3 mb-4'>
        <div class='p-2 bg-primary/10 rounded-lg'>
          <Minimize2 class='size-5 text-primary' />
        </div>
        <h3 class='text-lg font-medium'>
          Minimize Behavior
        </h3>
      </div>
      <p class='text-sm text-muted-foreground mb-4'>
        Control how the application behaves when minimized or closed
      </p>

      <div class='space-y-4'>
        <div class='flex items-center space-x-3 p-3 bg-background/50 rounded-lg border border-border/30'>
          <div class='relative flex items-center justify-center'>
            <input
              @change='handleMinimizeToggle(($event.target as HTMLInputElement).checked)'
              id='minimize-checkbox'
              :checked='systemTrayStore.minimizeToTray'
              class='peer h-5 w-5 shrink-0 appearance-none rounded-sm border border-input
                     ring-offset-background focus-visible:outline-none focus-visible:ring-2
                     focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed
                     disabled:opacity-50 checked:bg-accent checked:text-accent-foreground checked:border-accent'
              type='checkbox'
            >
            <div
              class='absolute inset-0 flex items-center justify-center text-accent-foreground
                       opacity-0 peer-checked:opacity-100 pointer-events-none'
            >
              <svg
                class='h-3 w-3'
                fill='none'
                viewBox='0 0 12 12'
                xmlns='http://www.w3.org/2000/svg'
              >
                <path
                  d='M10.5 3L4.5 9L2 6.5'
                  stroke='currentColor'
                  stroke-linecap='round'
                  stroke-linejoin='round'
                  stroke-width='1.5'
                />
              </svg>
            </div>
          </div>
          <div class='flex items-center space-x-2 flex-1'>
            <Label class='text-sm font-medium cursor-pointer' for='minimize-checkbox'>
              Minimize to system tray
            </Label>
          </div>
        </div>

        <div class='flex items-center space-x-3 p-3 bg-background/50 rounded-lg border border-border/30'>
          <div class='relative flex items-center justify-center'>
            <input
              @change='handleCloseToggle(($event.target as HTMLInputElement).checked)'
              id='close-checkbox'
              :checked='systemTrayStore.closeToTray'
              class='peer h-5 w-5 shrink-0 appearance-none rounded-sm border border-input
                     ring-offset-background focus-visible:outline-none focus-visible:ring-2
                     focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed
                     disabled:opacity-50 checked:bg-accent checked:text-accent-foreground checked:border-accent'
              type='checkbox'
            >
            <div
              class='absolute inset-0 flex items-center justify-center text-accent-foreground
                       opacity-0 peer-checked:opacity-100 pointer-events-none'
            >
              <svg
                class='h-3 w-3'
                fill='none'
                viewBox='0 0 12 12'
                xmlns='http://www.w3.org/2000/svg'
              >
                <path
                  d='M10.5 3L4.5 9L2 6.5'
                  stroke='currentColor'
                  stroke-linecap='round'
                  stroke-linejoin='round'
                  stroke-width='1.5'
                />
              </svg>
            </div>
          </div>
          <div class='flex items-center space-x-2 flex-1'>
            <Label class='text-sm font-medium cursor-pointer' for='close-checkbox'>
              Close to system tray
            </Label>
          </div>
        </div>
      </div>

      <p class='text-xs text-muted-foreground mt-4'>
        When enabled, the app will hide to the system tray instead of closing completely.
        You can restore it by clicking the tray icon.
      </p>
    </div>
  </section>
</template>
