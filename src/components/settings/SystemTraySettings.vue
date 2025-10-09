<script setup lang="ts">
  import {
    Minimize2,
    Monitor,
  } from 'lucide-vue-next'

  import Label from '@/components/ui/Label.vue'
  import Switch from '@/components/ui/Switch.vue'
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
        <div class='flex items-center justify-between p-3 bg-background/50 rounded-lg border border-border/30'>
          <Label class='text-sm font-medium cursor-pointer' for='minimize-switch'>
            Minimize to system tray
          </Label>
          <Switch
            @update:checked='handleMinimizeToggle'
            id='minimize-switch'
            :checked='systemTrayStore.minimizeToTray'
          />
        </div>

        <div class='flex items-center justify-between p-3 bg-background/50 rounded-lg border border-border/30'>
          <Label class='text-sm font-medium cursor-pointer' for='close-switch'>
            Close to system tray
          </Label>
          <Switch
            @update:checked='handleCloseToggle'
            id='close-switch'
            :checked='systemTrayStore.closeToTray'
          />
        </div>
      </div>

      <p class='text-xs text-muted-foreground mt-4'>
        When enabled, the app will hide to the system tray instead of closing completely.
        You can restore it by clicking the tray icon.
      </p>
    </div>
  </section>
</template>
