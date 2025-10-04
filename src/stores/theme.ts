import { useLocalStorage } from '@vueuse/core'
import { defineStore } from 'pinia'
import { computed, watch } from 'vue'

import { COLOR_SCHEMES } from '@/lib/colorSchemes'

export const useThemeStore = defineStore('theme', () => {
  const selectedSchemeName = useLocalStorage('color-scheme', 'default-light')

  const selectedScheme = computed(() =>
    COLOR_SCHEMES.find(s => s.name === selectedSchemeName.value) || COLOR_SCHEMES[0],
  )

  const colorSchemes = computed(() => COLOR_SCHEMES)

  const isDarkMode = computed(() =>
    selectedScheme.value?.name.includes('dark') || false,
  )

  const setColorScheme = (schemeName: string): void => {
    selectedSchemeName.value = schemeName
  }

  watch(selectedScheme, newScheme => {
    if (typeof window !== 'undefined' && newScheme) {
      const root = document.documentElement
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
    }
  }, { immediate: true })

  return {
    colorSchemes,
    isDarkMode,
    selectedScheme,
    selectedSchemeName,
    setColorScheme,
  }
})