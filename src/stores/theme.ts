import { defineStore } from 'pinia'
import { computed, watch } from 'vue'
import { useLocalStorage } from '@vueuse/core'
import { COLOR_SCHEMES, type ColorScheme } from '@/lib/color-schemes'

export const useThemeStore = defineStore('theme', () => {
  const selectedSchemeName = useLocalStorage('color-scheme', 'default-light')

  const selectedScheme = computed(() => {
    return COLOR_SCHEMES.find(s => s.name === selectedSchemeName.value) || COLOR_SCHEMES[0]
  })

  const colorSchemes = computed(() => COLOR_SCHEMES)

  const isDarkMode = computed(() => {
    return selectedScheme.value?.name.includes('dark') || false
  })

  const setColorScheme = (schemeName: string) => {
    selectedSchemeName.value = schemeName
  }

  const toggleDarkMode = () => {
    const currentScheme = selectedScheme.value
    if (currentScheme) {
      const isDark = currentScheme.name.includes('dark')
      const baseName = currentScheme.name.replace('-dark', '').replace('-light', '')
      const newSchemeName = isDark ? `${baseName}-light` : `${baseName}-dark`

      const targetScheme = COLOR_SCHEMES.find(s => s.name === newSchemeName)
      if (targetScheme) {
        setColorScheme(newSchemeName)
      }
    }
  }

  const applyColorScheme = (scheme: ColorScheme) => {
    if (typeof window !== 'undefined') {
      const root = document.documentElement
      const colors = scheme.colors

      root.style.setProperty('--background', colors.background)
      root.style.setProperty('--foreground', colors.foreground)
      root.style.setProperty('--card', colors.card)
      root.style.setProperty('--card-foreground', colors.cardForeground)
      root.style.setProperty('--popover', colors.popover)
      root.style.setProperty('--popover-foreground', colors.popoverForeground)
      root.style.setProperty('--primary', colors.primary)
      root.style.setProperty('--primary-foreground', colors.primaryForeground)
      root.style.setProperty('--secondary', colors.secondary)
      root.style.setProperty('--secondary-foreground', colors.secondaryForeground)
      root.style.setProperty('--muted', colors.muted)
      root.style.setProperty('--muted-foreground', colors.mutedForeground)
      root.style.setProperty('--destructive', colors.destructive)
      root.style.setProperty('--destructive-foreground', colors.destructiveForeground)
      root.style.setProperty('--border', colors.border)
      root.style.setProperty('--input', colors.input)
      root.style.setProperty('--ring', colors.ring)
      root.style.setProperty('--chart-1', colors.chart1)
      root.style.setProperty('--chart-2', colors.chart2)
      root.style.setProperty('--chart-3', colors.chart3)
      root.style.setProperty('--chart-4', colors.chart4)
      root.style.setProperty('--chart-5', colors.chart5)
      root.style.setProperty('--sidebar', colors.sidebar)
    }
  }

  const resetToDefault = () => {
    setColorScheme('default-light')
  }

  watch(selectedScheme, newScheme => {
    if (newScheme) {
      applyColorScheme(newScheme)
    }
  }, { immediate: true })

  return {
    selectedSchemeName,
    selectedScheme,
    colorSchemes,
    isDarkMode,
    setColorScheme,
    toggleDarkMode,
    applyColorScheme,
    resetToDefault,
  }
})