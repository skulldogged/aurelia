import type { ColorScheme } from 'tauri-plugin-m3'

import { useLocalStorage } from '@vueuse/core'
import { defineStore } from 'pinia'
import { M3 } from 'tauri-plugin-m3'
import { ref, watch } from 'vue'

import { getPlatform, Platform } from '@/lib/platform'

export const useMaterialYouStore = defineStore('materialYou', () => {
  const useMaterialYou = useLocalStorage('use-material-you', false)
  const materialYouColors = ref<ColorScheme | false>(false)

  const setUseMaterialYou = (value: boolean): void => {
    if (getPlatform() !== Platform.Android) return
    useMaterialYou.value = value
  }

  watch(useMaterialYou, async newValue => {
    if (getPlatform() !== Platform.Android) return

    if (newValue) {
      try {
        materialYouColors.value = await M3.getColors('system')
      } catch (e) {
        console.error('Failed to get Material You colors:', e)
      }
    } else {
      materialYouColors.value = false
    }
  }, { immediate: true })

  return {
    materialYouColors,
    setUseMaterialYou,
    useMaterialYou,
  }
})
