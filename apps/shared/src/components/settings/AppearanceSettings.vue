<script setup lang="ts">
  import { storeToRefs } from 'pinia'
  import { computed, ref } from 'vue'

  import Label from '../ui/Label.vue'
  import {
    Select,
    SelectContent,
    SelectGroup,
    SelectItem,
    SelectTrigger,
    SelectValue,
  } from '../ui/select'
  import Switch from '../ui/Switch.vue'
  import { useSystemTray } from '../../composables/useSystemTray'
  import { AccentColorName } from '../../lib/colorSchemes'
  import {
    useAccentColorStore,
    usePlayerStore,
    useSystemTrayStore,
    useThemeStore,
  } from '../../stores'

  const accentColorStore = useAccentColorStore()
  const themeStore = useThemeStore()
  const playerStore = usePlayerStore()
  const systemTrayStore = useSystemTrayStore()

  const { accentColor: accentColorRef, accentColors: accentColorsRef } = storeToRefs(accentColorStore)
  const { colorSchemes: colorSchemesRef, selectedScheme: selectedSchemeRef } = storeToRefs(themeStore)

  const { setAccentColor } = accentColorStore
  const { setColorScheme } = themeStore
  const { setCloseToTray, setMinimizeToTray } = useSystemTray()

  const accentColor = computed(() => accentColorRef.value)
  const accentColors = computed(() => accentColorsRef.value)
  const selectedScheme = computed(() => selectedSchemeRef.value)
  const colorSchemes = computed(() => colorSchemesRef.value)

  const selectedColorScheme = ref(selectedScheme.value.name)
  const selectedAccentColorName = ref(accentColor.value.name)
  const selectedVisualizerStyle = ref(playerStore.visualizerStyle)
  const visualizerEnabled = ref(playerStore.visualizerEnabled)


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
    const validStyles = ['bars', 'curve', 'wave'] as const
    if (value && typeof value === 'string' && validStyles.includes(value as typeof validStyles[number])) {
      const style = value as 'bars' | 'curve' | 'wave'
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

  const handleColorSchemeChange = (value: unknown): void => {
    if (value && typeof value === 'string')
      setColorScheme(value)
  }
</script>

<template>
  <div class='space-y-8'>


    <!-- Theme Settings Row -->
    <div class='space-y-4'>
      <h3 class='text-lg font-medium'>
        Theme Settings
      </h3>
      <div class='grid md:grid-cols-2 lg:grid-cols-3 gap-6'>
        <!-- Color Scheme -->
        <div class='space-y-3'>
          <Label class='text-sm font-medium'>
            Color Scheme
          </Label>
          <Select
            @update:model-value='handleColorSchemeChange'
            v-model='selectedColorScheme'

          >
            <SelectTrigger class='w-full bg-background/40 border-border/20 hover:border-border/40'>
              <SelectValue placeholder='Select a color scheme' />
            </SelectTrigger>
            <SelectContent class='border-border/30'>
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
          <p class='text-xs text-muted-foreground'>
            Choose your preferred theme
          </p>
        </div>

        <!-- Accent Color -->
        <div class='space-y-3'>
          <Label class='text-sm font-medium'>
            Accent Color
          </Label>
          <Select
            @update:model-value='handleAccentColorChange'
            v-model='selectedAccentColorName'

          >
            <SelectTrigger class='w-full bg-background/40 border-border/20 hover:border-border/40'>
              <SelectValue placeholder='Select an accent color' />
            </SelectTrigger>
            <SelectContent class='border-border/30'>
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
          <p class='text-xs text-muted-foreground'>
            Pick your favorite accent color
          </p>
        </div>
      </div>
    </div>

    <!-- Audio Visualizer -->
    <div class='space-y-4 pt-4 border-t border-border/20'>
      <h3 class='text-lg font-medium'>
        Audio Visualizer
      </h3>

      <div class='grid md:grid-cols-2 gap-6'>
        <div
          class='
            flex items-center justify-between p-4 bg-background/40 rounded-lg
            border border-border/20 hover:border-border/40 transition-colors
          '
        >
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
            <SelectTrigger class='w-full bg-background/40 border-border/20 hover:border-border/40'>
              <SelectValue placeholder='Select a style' />
            </SelectTrigger>
            <SelectContent class='border-border/30'>
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

      <p class='text-xs text-muted-foreground'>
        {{
          visualizerEnabled
            ? 'Choose how audio is visualized during playback'
            : 'Show visual effects during playback'
        }}
      </p>
    </div>

    <!-- System Tray -->
    <div class='space-y-4 pt-4 border-t border-border/20'>

      <h3 class='text-lg font-medium'>
        System Tray
      </h3>

      <div class='space-y-3'>
        <div
          class='
            flex items-center justify-between p-4 bg-background/40 rounded-lg
            border border-border/20 hover:border-border/40 transition-colors
          '
        >
          <Label class='text-sm cursor-pointer' for='minimize-switch'>
            Minimize to system tray
          </Label>
          <Switch
            @update:checked='handleMinimizeToggle'
            id='minimize-switch'
            :checked='systemTrayStore.minimizeToTray'
          />
        </div>

        <div
          class='
            flex items-center justify-between p-4 bg-background/40 rounded-lg
            border border-border/20 hover:border-border/40 transition-colors
          '
        >
          <Label class='text-sm cursor-pointer' for='close-switch'>
            Close to system tray
          </Label>
          <Switch
            @update:checked='handleCloseToggle'
            id='close-switch'
            :checked='systemTrayStore.closeToTray'
          />
        </div>

        <p class='text-xs text-muted-foreground'>
          When enabled, the app will hide to the system tray instead of closing completely
        </p>
      </div>
    </div>
  </div>
</template>
