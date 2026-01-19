import { useLocalStorage } from '@vueuse/core'
import { defineStore } from 'pinia'
import { computed, watch } from 'vue'

import { type AccentColor, AccentColorName } from '@/lib/colorSchemes'
import { useThemeStore } from '@/stores/theme'

export const useAccentColorStore = defineStore('accentColor', () => {
  const accentColorName = useLocalStorage<AccentColorName>('accent-color', AccentColorName.Blue)

  const themeStore = useThemeStore()

  const accentColors = computed(() => themeStore.selectedScheme?.accentColors || [])

  const accentColor = computed(() =>
    accentColors.value.find(c => c.name === accentColorName.value) || accentColors.value[0] || {
      foreground: '#fafafa',
      hex:        '#3b82f6',
      name:       AccentColorName.Blue,
    })

  const currentAccentHex = computed(() => accentColor.value.hex)

  const currentAccentForeground = computed(() => accentColor.value.foreground)

  const setAccentColor = (colorName: AccentColorName): void => {
    accentColorName.value = colorName
  }

  const applyAccentColor = (color: AccentColor): void => {
    if (typeof window !== 'undefined' && color) {
      const root = document.documentElement

      root.style.setProperty('--accent', color.hex)

      root.style.setProperty('--accent-foreground', color.foreground)
    }
  }

  watch(accentColor, newColor => {
    if (newColor)
      applyAccentColor(newColor)
  }, { immediate: true })

  return {
    accentColor,
    accentColorName,
    accentColors,
    applyAccentColor,
    currentAccentForeground,
    currentAccentHex,
    setAccentColor,
  }
})