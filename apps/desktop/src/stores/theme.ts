import { useLocalStorage } from '@vueuse/core'
import { defineStore } from 'pinia'
import { computed, watch } from 'vue'

import { COLOR_SCHEMES } from '@/lib/colorSchemes'
import { logger } from '@/lib/logger'


export const useThemeStore = defineStore('theme', () => {
  // Determine default theme based on system preference if no saved preference exists
  const getDefaultTheme = (): string => {
    if (typeof window === 'undefined') return 'default-light'

    const saved = localStorage.getItem('color-scheme')
    if (saved) return saved

    const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches
    return prefersDark ? 'default-dark' : 'default-light'
  }

  const selectedSchemeName = useLocalStorage('color-scheme', getDefaultTheme())

  const selectedScheme = computed(() =>
    COLOR_SCHEMES.find(s => s.name === selectedSchemeName.value) || COLOR_SCHEMES[0],
  )

  const colorSchemes = computed(() => COLOR_SCHEMES)

  const isDarkMode = computed(() => {
    const name = selectedScheme.value?.name
    if (!name) return false
    // Light themes explicitly have 'light' in name, everything else is dark
    return !name.includes('light')
  })


  const setColorScheme = (schemeName: string): void => {
    selectedSchemeName.value = schemeName
  }

  watch(
    selectedScheme,
    async newScheme => {
      if (typeof window === 'undefined') return

      const root = document.documentElement

      if (newScheme) {
        const colors = newScheme.colors
        root.style.setProperty('--background', colors.background)
        root.style.setProperty('--background-dark', colors.backgroundDark)
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
        root.style.setProperty('--success', colors.success)
        root.style.setProperty('--sidebar', colors.sidebar)
        root.style.setProperty('--font-family', '"Rubik", sans-serif')
      }
    },
    { deep: true, immediate: true },
  )


  return {
    colorSchemes,
    isDarkMode,
    selectedScheme,
    selectedSchemeName,
    setColorScheme,
  }
})
