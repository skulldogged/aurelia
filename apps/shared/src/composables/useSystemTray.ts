import { onMounted } from 'vue'

import { getAureliaDesktop } from '../lib/desktop-shell'
import { logger } from '../lib/logger'
import { useSystemTrayStore } from '../stores'

export const useSystemTray = (): {
  hideMainWindow:    () => Promise<void>
  quitApplication:   () => Promise<void>
  setCloseToTray:    (closeToTray: boolean) => Promise<void>
  setMinimizeToTray: (minimizeToTray: boolean) => Promise<void>
  showMainWindow:    () => Promise<void>
} => {
  const systemTrayStore = useSystemTrayStore()

  const showMainWindow = async (): Promise<void> => {
    try {
      const desktop = getAureliaDesktop()
      if (desktop) {
        await desktop.window.show()
      }
    } catch (error) {
      logger.error('Failed to show main window:', error)
    }
  }

  const hideMainWindow = async (): Promise<void> => {
    try {
      const desktop = getAureliaDesktop()
      if (desktop) {
        await desktop.window.hide()
      }
    } catch (error) {
      logger.error('Failed to hide main window:', error)
    }
  }

  const quitApplication = async (): Promise<void> => {
    try {
      const desktop = getAureliaDesktop()
      if (desktop) {
        await desktop.quit()
      }
    } catch (error) {
      logger.error('Failed to quit application:', error)
    }
  }

  const setMinimizeToTray = async (minimizeToTray: boolean): Promise<void> => {
    try {
      systemTrayStore.setMinimizeToTray(minimizeToTray)
      const desktop = getAureliaDesktop()
      if (desktop) {
        await desktop.setMinimizeToTray(minimizeToTray)
      }
    } catch (error) {
      logger.error('Failed to set minimize to tray:', error)
      // Revert store change on failure
      systemTrayStore.setMinimizeToTray(!minimizeToTray)
    }
  }

  const setCloseToTray = async (closeToTray: boolean): Promise<void> => {
    try {
      systemTrayStore.setCloseToTray(closeToTray)
      const desktop = getAureliaDesktop()
      if (desktop) {
        await desktop.setCloseToTray(closeToTray)
      }
    } catch (error) {
      logger.error('Failed to set close to tray:', error)
      // Revert store change on failure
      systemTrayStore.setCloseToTray(!closeToTray)
    }
  }

  // Initialize system tray settings when the composable is first used
  onMounted(async () => {
    try {
      await setMinimizeToTray(systemTrayStore.minimizeToTray)
      await setCloseToTray(systemTrayStore.closeToTray)
    } catch (error) {
      logger.error('Failed to initialize system tray settings:', error)
    }
  })

  return {
    hideMainWindow,
    quitApplication,
    setCloseToTray,
    setMinimizeToTray,
    showMainWindow,
  }
}
