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
  // State
  const accentColorName = useLocalStorage('accent-color', 'blue')

  // Get theme store for accessing current scheme
  const themeStore = useThemeStore()

  // Getters
  const accentColors = computed(() => {
    return themeStore.selectedScheme?.accentColors || []
  })

  const accentColor = computed(() => {
    const color = accentColors.value.find(c => c.name === accentColorName.value)
    // Fallback to the first available color if the stored one isn't found
    return color || accentColors.value[0] || {
      name:        'blue',
      displayName: 'Blue',
      hex:         '#3b82f6',
    }
  })

  const currentAccentHex = computed(() => accentColor.value.hex)

  const currentAccentForeground = computed(() => {
    const color = accentColor.value
    if (color.foreground) {
      return color.foreground
    }

    // Calculate foreground based on luminance
    const luminance = getLuminance(color.hex)
    return luminance > 0.4
      ? 'oklch(0.21 0.006 285.885)' // dark text
      : 'oklch(0.985 0 0)' // light text
  })

  // Actions
  const setAccentColor = (colorName: string) => {
    accentColorName.value = colorName
  }

  const setAccentColorByHex = (hex: string) => {
    const color = accentColors.value.find(c => c.hex === hex)
    if (color) {
      setAccentColor(color.name)
    }
  }

  const resetToDefault = () => {
    accentColorName.value = 'blue'
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

      // Set accent color
      root.style.setProperty('--accent', color.hex)

      // Set accent foreground color based on luminance
      if (color.foreground) {
        root.style.setProperty('--accent-foreground', color.foreground)
      } else if (luminance > 0.4) {
        root.style.setProperty('--accent-foreground', 'oklch(0.21 0.006 285.885)') // dark text
      } else {
        root.style.setProperty('--accent-foreground', 'oklch(0.985 0 0)') // light text
      }
    }
  }

  const getContrastColor = (hex: string): string => {
    const luminance = getLuminance(hex)
    return luminance > 0.4 ? '#000000' : '#ffffff'
  }

  // Watch for accent color changes and apply them
  watch(accentColor, newColor => {
    if (newColor) {
      applyAccentColor(newColor)
    }
  }, { immediate: true })

  return {
    // State
    accentColorName,
    // Getters
    accentColor,
    accentColors,
    currentAccentHex,
    currentAccentForeground,
    // Actions
    setAccentColor,
    setAccentColorByHex,
    resetToDefault,
    applyAccentColor,
    getLuminance,
    getContrastColor,
  }
})