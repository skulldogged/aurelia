import { defineStore } from 'pinia'
import { computed, watch } from 'vue'
import { useLocalStorage } from '@vueuse/core'
import { useThemeStore } from './theme'

export interface AccentColor {
  name:        string
  displayName: string
  hex:         string
  foreground?: string
}

export const useAccentColorStore = defineStore('accentColor', () => {
  const accentColorName = useLocalStorage('accent-color', 'blue')

  const themeStore = useThemeStore()

  const accentColors = computed(() => {
    return themeStore.selectedScheme?.accentColors || []
  })

  const accentColor = computed(() =>
    accentColors.value.find(c => c.name === accentColorName.value) || accentColors.value[0] || {
      name:        'blue',
      displayName: 'Blue',
      hex:         '#3b82f6',
    })

  const currentAccentHex = computed(() => accentColor.value.hex)

  const currentAccentForeground = computed(() => {
    const color = accentColor.value
    if (color.foreground)
      return color.foreground

    return getLuminance(color.hex) > 0.4
      ? 'oklch(0.21 0.006 285.885)' // dark text
      : 'oklch(0.985 0 0)' // light text
  })

  const setAccentColor = (colorName: string) => {
    accentColorName.value = colorName
  }

  const getLuminance = (hex: string): number => {
    const r = parseInt(hex.slice(1, 3), 16)
    const g = parseInt(hex.slice(3, 5), 16)
    const b = parseInt(hex.slice(5, 7), 16)
    const a = [r, g, b].map(v => {
      v /= 255
      return v <= 0.03928 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4)
    })
    return a[0] * 0.2126 + a[1] * 0.7152 + a[2] * 0.0722
  }

  const applyAccentColor = (color: AccentColor) => {
    if (typeof window !== 'undefined' && color) {
      const root = document.documentElement
      const luminance = getLuminance(color.hex)

      root.style.setProperty('--accent', color.hex)

      if (color.foreground)
        root.style.setProperty('--accent-foreground', color.foreground)
      else if (luminance > 0.4)
        root.style.setProperty('--accent-foreground', 'oklch(0.21 0.006 285.885)') // dark text
      else
        root.style.setProperty('--accent-foreground', 'oklch(0.985 0 0)') // light text
    }
  }

  watch(accentColor, newColor => {
    if (newColor)
      applyAccentColor(newColor)
  }, { immediate: true })

  return {
    accentColorName,
    accentColor,
    accentColors,
    currentAccentHex,
    currentAccentForeground,
    setAccentColor,
    applyAccentColor,
    getLuminance,
  }
})