import { useLocalStorage } from '@vueuse/core'
import { defineStore } from 'pinia'

export const useSystemTrayStore = defineStore('systemTray', () => {
  const minimizeToTray = useLocalStorage('minimize-to-tray', true)
  const closeToTray = useLocalStorage('close-to-tray', false)

  const setMinimizeToTray = (value: boolean): void => {
    minimizeToTray.value = value
  }

  const setCloseToTray = (value: boolean): void => {
    closeToTray.value = value
  }

  const toggleMinimizeToTray = (): void => {
    minimizeToTray.value = !minimizeToTray.value
  }

  const toggleCloseToTray = (): void => {
    closeToTray.value = !closeToTray.value
  }

  return {
    closeToTray,
    minimizeToTray,
    setCloseToTray,
    setMinimizeToTray,
    toggleCloseToTray,
    toggleMinimizeToTray,
  }
})
