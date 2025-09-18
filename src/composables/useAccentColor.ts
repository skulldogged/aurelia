import { watch, computed } from 'vue'
import { useLocalStorage } from '@vueuse/core'
import { useTheme } from './useTheme'

export const useAccentColor = () => {
  const { selectedScheme } = useTheme()

  const accentColors = computed(() => selectedScheme.value?.accentColors || [])

  const accentColorName = useLocalStorage('accent-color', 'blue')

  const accentColor = computed(() => {
    const color = accentColors.value.find(c => c.name === accentColorName.value)
    return color || accentColors.value[0] || { name: 'blue', displayName: 'Blue', hex: '#3b82f6' }
  })

  const setAccentColor = (colorName: string) => {
    accentColorName.value = colorName
  }

  const getLuminance = (hex: string) => {
    const r = parseInt(hex.slice(1, 3), 16)
    const g = parseInt(hex.slice(3, 5), 16)
    const b = parseInt(hex.slice(5, 7), 16)
    const a = [r, g, b].map(v => {
      v /= 255
      return v <= 0.03928 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4)
    })
    return a[0] * 0.2126 + a[1] * 0.7152 + a[2] * 0.0722
  }

  const applyAccentColor = (color: { hex: string, foreground?: string }) => {
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
    accentColor,
    setAccentColor,
    accentColors,
  }
}
