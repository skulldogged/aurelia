<script setup lang='ts'>
  import { Window } from '@tauri-apps/api/window'
  import { storeToRefs } from 'pinia'
  import { onMounted, ref } from 'vue'

  import { useSystemTray } from '../../composables/useSystemTray'
  import { getPlatform, isDesktop } from '../../lib/platform'
  import { useSystemTrayStore } from '../../stores'
  import CloseIcon from './window-controls/CloseIcon.vue'
  import MaximizeIcon from './window-controls/MaximizeIcon.vue'
  import MinimizeIcon from './window-controls/MinimizeIcon.vue'
  import RestoreIcon from './window-controls/RestoreIcon.vue'

  const isMaximized = ref(false)
  const isLinuxPlatform = ref(false)

  const appWindow = isDesktop() ? Window.getCurrent() : null

  const { closeToTray, minimizeToTray } = storeToRefs(useSystemTrayStore())
  const { hideMainWindow } = useSystemTray()

  const checkMaximized = async (): Promise<void> => {
    if (appWindow) {
      isMaximized.value = await appWindow.isMaximized()
    }
  }

  const handleMinimize = (): Promise<void> => {
    if (!appWindow) return Promise.resolve()
    return minimizeToTray.value ? hideMainWindow() : appWindow.minimize()
  }

  const handleClose = (): Promise<void> => {
    if (!appWindow) return Promise.resolve()
    return closeToTray.value ? hideMainWindow() : appWindow.close()
  }

  const handleToggleMaximize = (): Promise<void> => {
    if (!appWindow) return Promise.resolve()
    return appWindow.toggleMaximize()
  }

  if (appWindow) {
    appWindow.onResized(checkMaximized)
  }

  onMounted(async () => {
    if (isDesktop()) {
      await checkMaximized()
      isLinuxPlatform.value = getPlatform() === 'linux'
    }
  })
</script>

<template>
  <div
    v-if='isLinuxPlatform'
    class='flex h-auto items-center gap-[13px]'
  >
    <button
      @click='handleMinimize'
      class='
        flex aspect-square h-6 w-6 cursor-default items-center justify-center
        rounded-full bg-sidebar p-0 text-foreground
        hover:bg-muted/80 active:bg-muted/60
      '
    >
      <MinimizeIcon class='h-[9px] w-[9px]' />
    </button>
    <button
      @click='handleToggleMaximize'
      class='
        flex aspect-square h-6 w-6 cursor-default items-center justify-center
        rounded-full bg-sidebar p-0 text-foreground
        hover:bg-muted/80 active:bg-muted/60
      '
    >
      <RestoreIcon
        v-if='isMaximized'
        class='h-[9px] w-[9px]'
      />
      <MaximizeIcon
        v-else
        class='h-2 w-2'
      />
    </button>
    <button
      @click='handleClose'
      class='
        flex aspect-square h-6 w-6 cursor-default items-center justify-center
        rounded-full bg-sidebar p-0 text-foreground
        hover:bg-accent hover:text-accent-foreground
        active:bg-accent/90 active:text-accent-foreground
      '
    >
      <CloseIcon class='h-2 w-2' />
    </button>
  </div>

  <div
    v-else
    class='flex h-12 items-center'
  >
    <button
      @click='handleMinimize'
      class='
        inline-flex h-full w-[46px] cursor-default items-center justify-center
        rounded-none bg-transparent text-foreground
        hover:bg-black/5 active:bg-black/3
        dark:hover:bg-white/6 dark:active:bg-white/4
      '
    >
      <MinimizeIcon class='h-[9px] w-[9px]' />
    </button>
    <button
      @click='handleToggleMaximize'
      class='
        inline-flex h-full w-[46px] cursor-default items-center justify-center
        rounded-none bg-transparent text-foreground
        hover:bg-black/5 active:bg-black/3
        dark:hover:bg-white/6 dark:active:bg-white/4
      '
    >
      <RestoreIcon
        v-if='isMaximized'
        class='h-[9px] w-[9px]'
      />
      <MaximizeIcon
        v-else
        class='h-2 w-2'
      />
    </button>
    <button
      @click='handleClose'
      class='
        inline-flex h-full w-[46px] cursor-default items-center justify-center
        rounded-none bg-transparent text-foreground
        hover:bg-accent hover:text-accent-foreground
        active:bg-accent/90 active:text-accent-foreground
      '
    >
      <CloseIcon class='h-2 w-2' />
    </button>
  </div>
</template>
