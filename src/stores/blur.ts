import { useLocalStorage } from '@vueuse/core'
import { defineStore } from 'pinia'
import { computed } from 'vue'

export interface BlurMode {
  displayName: string
  name:        string
  supported:   boolean
}

export const BLUR_MODES: BlurMode[] = [
  {
    displayName: 'None',
    name:        'none',
    supported:   true,
  },
  {
    displayName: 'Acrylic',
    name:        'acrylic',
    supported:   true,
  },
  {
    displayName: 'Mica',
    name:        'mica',
    supported:   true,
  },
  {
    displayName: 'Tabbed',
    name:        'tabbed',
    supported:   true,
  },
]

export const useBlurStore = defineStore('blur', () => {
  const selectedBlurModeName = useLocalStorage('blur-mode', 'acrylic')

  const blurModes = computed(() => BLUR_MODES)

  const selectedBlurMode = computed(() =>
    blurModes.value.find(mode => mode.name === selectedBlurModeName.value) || blurModes.value[0],
  )

  const setBlurMode = (modeName: string): void => {
    selectedBlurModeName.value = modeName
  }

  return {
    blurModes,
    selectedBlurMode,
    selectedBlurModeName,
    setBlurMode,
  }
})
