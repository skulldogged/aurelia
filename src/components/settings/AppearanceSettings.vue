<script setup lang="ts">
  import {
    Palette,
  } from 'lucide-vue-next'
  import { storeToRefs } from 'pinia'
  import { computed, onMounted, ref } from 'vue'

  import { commands } from '@/bindings'
  import Label from '@/components/ui/Label.vue'
  import {
    Select,
    SelectContent,
    SelectGroup,
    SelectItem,
    SelectTrigger,
    SelectValue,
  } from '@/components/ui/select'
  import Switch from '@/components/ui/Switch.vue'
  import { useSystemTray } from '@/composables/useSystemTray'
  import { AccentColorName } from '@/lib/colorSchemes'
  import { getPlatform } from '@/lib/platform'
  import { useAccentColorStore, useBlurStore, usePlayerStore, useSystemTrayStore, useThemeStore } from '@/stores'

  const accentColorStore = useAccentColorStore()
  const themeStore = useThemeStore()
  const blurStore = useBlurStore()
  const playerStore = usePlayerStore()
  const systemTrayStore = useSystemTrayStore()

  const { accentColor: accentColorRef, accentColors: accentColorsRef } = storeToRefs(accentColorStore)
  const { colorSchemes: colorSchemesRef, selectedScheme: selectedSchemeRef } = storeToRefs(themeStore)
  const { blurModes: blurModesRef, selectedBlurMode: selectedBlurModeRef } = storeToRefs(blurStore)

  const { setAccentColor } = accentColorStore
  const { setColorScheme } = themeStore
  const { setBlurMode } = blurStore
  const { setCloseToTray, setMinimizeToTray } = useSystemTray()

  const accentColor = computed(() => accentColorRef.value)
  const accentColors = computed(() => accentColorsRef.value)
  const selectedScheme = computed(() => selectedSchemeRef.value)
  const colorSchemes = computed(() => colorSchemesRef.value)
  const selectedBlurMode = computed(() => selectedBlurModeRef.value)
  const blurModes = computed(() => blurModesRef.value)

  const selectedColorScheme = ref(selectedScheme.value.name)
  const selectedBlurModeName = ref(selectedBlurMode.value.name)
  const selectedAccentColorName = ref(accentColor.value.name)
  const selectedVisualizerStyle = ref(playerStore.visualizerStyle)
  const visualizerEnabled = ref(playerStore.visualizerEnabled)
  const isLinuxPlatform = ref(false)

  // Helper to capitalize enum names for display
  const formatEnumName = (name: string): string =>
    name
      .split('-')
      .map(word => word.charAt(0).toUpperCase() + word.slice(1))
      .join(' ')

  // Visualizer styles
  const visualizerStyles = [
    { displayName: 'Bars (Mirror)', value: 'bars-mirror' },
    { displayName: 'Bars', value: 'bars' },
    { displayName: 'Curve', value: 'curve' },
    { displayName: 'Wave', value: 'wave' },
  ]

  // Linux-specific transparency modes
  const transparencyModes = [
    { displayName: 'Disabled', name: 'none' },
    { displayName: 'Enabled', name: 'acrylic' },
  ]

  const applyTransparencyClass = (modeName: string): void =>
    void (modeName === 'none'
      ? document.body.classList.add('transparency-disabled')
      : document.body.classList.remove('transparency-disabled'))

  const handleBlurModeChange = async (value: unknown): Promise<void> => {
    if (value && typeof value === 'string') {
      try {
        // Update the store first
        setBlurMode(value)
        // Update the local ref for the Select component
        selectedBlurModeName.value = value
        // Apply CSS class for transparency
        applyTransparencyClass(value)
        // Apply the blur mode to the window (Windows/macOS only)
        await commands.setBlurMode(value)
      } catch (error) {
        console.error('Failed to set blur mode:', error)
      }
    }
  }

  const handleAccentColorChange = (value: unknown): void => {
    if (value && typeof value === 'string' && Object.values(AccentColorName).includes(value as AccentColorName)) {
      setAccentColor(value as AccentColorName)
      selectedAccentColorName.value = value as AccentColorName
    }
  }

  const handleVisualizerToggle = (checked: boolean): void => {
    visualizerEnabled.value = checked
    playerStore.setVisualizerEnabled(checked)
  }

  const handleVisualizerStyleChange = (value: unknown): void => {
    const validStyles = ['bars', 'bars-mirror', 'curve', 'wave'] as const
    if (value && typeof value === 'string' && validStyles.includes(value as typeof validStyles[number])) {
      const style = value as 'bars' | 'bars-mirror' | 'curve' | 'wave'
      playerStore.setVisualizerStyle(style)
      selectedVisualizerStyle.value = style
    }
  }

  const handleMinimizeToggle = async (checked: boolean): Promise<void> => {
    systemTrayStore.minimizeToTray = checked
    await setMinimizeToTray(checked)
  }

  const handleCloseToggle = async (checked: boolean): Promise<void> => {
    systemTrayStore.closeToTray = checked
    await setCloseToTray(checked)
  }

  // Apply initial blur mode when component mounts
  onMounted(async () => {
    try {
      // Check platform
      isLinuxPlatform.value = getPlatform() === 'linux'

      // Update the local ref to match the current store value
      selectedBlurModeName.value = selectedBlurMode.value.name
      // Apply initial CSS class
      applyTransparencyClass(selectedBlurMode.value.name)
      // Apply initial blur mode
      await commands.setBlurMode(selectedBlurMode.value.name)
    } catch (error) {
      console.error('Failed to apply initial blur mode:', error)
    }
  })

  const handleColorSchemeChange = (value: unknown): void => {
    if (value && typeof value === 'string')
      setColorScheme(value)
  }
</script>

<template>
  <div class='bg-sidebar rounded-lg'>
    <!-- Header -->
    <div class='p-6'>
      <div class='flex items-center space-x-3'>
        <div class='p-2 bg-accent/10 rounded-lg'>
          <Palette class='size-5 text-accent' />
        </div>
        <h2 class='text-2xl font-semibold'>
          Appearance
        </h2>
      </div>
      <p class='text-sm text-muted-foreground mt-2'>
        Customize the look and feel of your music player
      </p>
    </div>

    <!-- Settings Grid -->
    <div class='p-6 space-y-6'>
      <!-- Theme Settings Row -->
      <div class='grid md:grid-cols-3 gap-6'>
        <!-- Color Scheme -->
        <div>
          <Label class='text-sm font-medium mb-3 block'>
            Color Scheme
          </Label>
          <Select @update:model-value='handleColorSchemeChange' v-model='selectedColorScheme'>
            <SelectTrigger class='w-full !bg-popover !border-border/30'>
              <SelectValue placeholder='Select a color scheme' />
            </SelectTrigger>
            <SelectContent class='!border-border/30'>
              <SelectGroup>
                <SelectItem
                  v-for='scheme in colorSchemes'
                  :key='scheme.name'
                  :value='scheme.name'
                  class='cursor-pointer'
                >
                  {{ formatEnumName(scheme.name) }}
                </SelectItem>
              </SelectGroup>
            </SelectContent>
          </Select>
          <p class='text-xs text-muted-foreground mt-2'>
            Choose your preferred theme
          </p>
        </div>

        <!-- Accent Color -->
        <div>
          <Label class='text-sm font-medium mb-3 block'>
            Accent Color
          </Label>
          <Select @update:model-value='handleAccentColorChange' v-model='selectedAccentColorName'>
            <SelectTrigger class='w-full !bg-popover !border-border/30'>
              <SelectValue placeholder='Select an accent color' />
            </SelectTrigger>
            <SelectContent class='!border-border/30'>
              <SelectGroup>
                <SelectItem
                  v-for='color in accentColors'
                  :key='color.name'
                  :value='color.name'
                  class='cursor-pointer'
                >
                  <div class='flex items-center space-x-3'>
                    <div
                      :style='{ backgroundColor: color.hex }'
                      class='size-4 rounded-full border border-border/20'
                    />
                    <span>{{ formatEnumName(color.name) }}</span>
                  </div>
                </SelectItem>
              </SelectGroup>
            </SelectContent>
          </Select>
          <p class='text-xs text-muted-foreground mt-2'>
            Pick your favorite accent color
          </p>
        </div>

        <!-- Window Effects -->
        <div>
          <Label class='text-sm font-medium mb-3 block'>
            {{ isLinuxPlatform ? 'Window Transparency' : 'Window Blur' }}
          </Label>
          <Select @update:model-value='handleBlurModeChange' :model-value='selectedBlurModeName'>
            <SelectTrigger class='w-full !bg-popover !border-border/30'>
              <SelectValue :placeholder='isLinuxPlatform ? "Select transparency" : "Select a blur mode"' />
            </SelectTrigger>
            <SelectContent class='!border-border/30'>
              <SelectGroup v-if='isLinuxPlatform'>
                <SelectItem
                  v-for='mode in transparencyModes'
                  :key='mode.name'
                  :value='mode.name'
                  class='cursor-pointer'
                >
                  {{ mode.displayName }}
                </SelectItem>
              </SelectGroup>
              <SelectGroup v-else>
                <SelectItem
                  v-for='mode in blurModes'
                  :key='mode.name'
                  :disabled='!mode.supported'
                  :value='mode.name'
                  class='cursor-pointer'
                >
                  {{ mode.displayName }}
                </SelectItem>
              </SelectGroup>
            </SelectContent>
          </Select>
          <p class='text-xs text-muted-foreground mt-2'>
            {{ isLinuxPlatform ? 'Toggle window transparency effect' : 'Choose the background blur effect' }}
          </p>
        </div>
      </div>

      <!-- Audio Visualizer -->
      <div>
        <Label class='text-sm font-medium mb-4 block'>
          Audio Visualizer
        </Label>

        <div class='grid md:grid-cols-2 gap-4'>
          <div class='flex items-center justify-between p-3 bg-popover rounded-lg border border-border/30'>
            <Label class='text-sm cursor-pointer' for='visualizer-switch'>
              Enable visualizer
            </Label>
            <Switch
              @update:checked='handleVisualizerToggle'
              id='visualizer-switch'
              :checked='visualizerEnabled'
            />
          </div>

          <div v-if='visualizerEnabled' class='h-full'>
            <Select @update:model-value='handleVisualizerStyleChange' v-model='selectedVisualizerStyle'>
              <SelectTrigger class='w-full !h-full !bg-popover !border-border/30'>
                <SelectValue placeholder='Select a style' />
              </SelectTrigger>
              <SelectContent class='!border-border/30'>
                <SelectGroup>
                  <SelectItem
                    v-for='style in visualizerStyles'
                    :key='style.value'
                    :value='style.value'
                    class='cursor-pointer'
                  >
                    {{ style.displayName }}
                  </SelectItem>
                </SelectGroup>
              </SelectContent>
            </Select>
          </div>
        </div>

        <p class='text-xs text-muted-foreground mt-3'>
          {{
            visualizerEnabled
              ? 'Choose how audio is visualized during playback'
              : 'Show visual effects during playback'
          }}
        </p>
      </div>

      <!-- System Tray -->
      <div>
        <Label class='text-sm font-medium mb-4 block'>
          System Tray
        </Label>

        <div class='space-y-3'>
          <div class='flex items-center justify-between p-3 bg-popover rounded-lg border border-border/30'>
            <Label class='text-sm cursor-pointer' for='minimize-switch'>
              Minimize to system tray
            </Label>
            <Switch
              @update:checked='handleMinimizeToggle'
              id='minimize-switch'
              :checked='systemTrayStore.minimizeToTray'
            />
          </div>

          <div class='flex items-center justify-between p-3 bg-popover rounded-lg border border-border/30'>
            <Label class='text-sm cursor-pointer' for='close-switch'>
              Close to system tray
            </Label>
            <Switch
              @update:checked='handleCloseToggle'
              id='close-switch'
              :checked='systemTrayStore.closeToTray'
            />
          </div>

          <p class='text-xs text-muted-foreground pl-4'>
            When enabled, the app will hide to the system tray instead of closing completely
          </p>
        </div>
      </div>
    </div>
  </div>
</template>
