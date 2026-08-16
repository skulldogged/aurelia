import { isElectron } from './platform'

export interface AureliaDesktopApi {
  appVersion: string
  lastFm?: {
    onToken:   (handler: (token: string) => void) => () => void
    startAuth: () => Promise<{ callbackUrl: string }>
  }
  openUrl:           (url: string) => Promise<void>
  platform:          string
  quit:              () => Promise<void>
  setCloseToTray:    (enabled: boolean) => Promise<void>
  setMinimizeToTray: (enabled: boolean) => Promise<void>
  window: {
    close:          () => Promise<void>
    hide:           () => Promise<void>
    isMaximized:    () => Promise<boolean>
    minimize:       () => Promise<void>
    onResized:      (handler: () => void) => () => void
    setTitleBarOverlay?: (options: { color: string; symbolColor: string }) => Promise<void>
    show:                () => Promise<void>
    toggleMaximize:      () => Promise<void>
  }
}

export interface DesktopWindowControls {
  close:          () => Promise<void>
  hide:           () => Promise<void>
  isMaximized:    () => Promise<boolean>
  minimize:       () => Promise<void>
  onResized:      (handler: () => void) => Promise<() => void>
  quit:           () => Promise<void>
  show:           () => Promise<void>
  toggleMaximize: () => Promise<void>
}

declare global {
  interface Window {
    aureliaDesktop?: AureliaDesktopApi
  }
}

export const getAureliaDesktop = (): AureliaDesktopApi | undefined => {
  if (typeof window === 'undefined') return undefined
  return window.aureliaDesktop
}

export const getDesktopWindow = async (): Promise<DesktopWindowControls | null> => {
  const api = getAureliaDesktop()
  if (!api) return null

  return {
    close:          () => api.window.close(),
    hide:           () => api.window.hide(),
    isMaximized:    () => api.window.isMaximized(),
    minimize:       () => api.window.minimize(),
    onResized:      async handler => api.window.onResized(handler),
    quit:           () => api.quit(),
    show:           () => api.window.show(),
    toggleMaximize: () => api.window.toggleMaximize(),
  }
}

export const openExternalUrl = async (url: string): Promise<void> => {
  const api = getAureliaDesktop()
  if (api) {
    await api.openUrl(url)
    return
  }

  window.open(url, '_blank', 'noopener,noreferrer')
}

export const quitDesktopApp = async (): Promise<void> => {
  const win = await getDesktopWindow()
  if (win) {
    await win.quit()
  }
}

export const writeClipboardText = async (text: string): Promise<void> => {
  await navigator.clipboard.writeText(text)
}

export const syncDesktopTitleBarOverlay = (): void => {
  const api = getAureliaDesktop()
  if (!api?.window.setTitleBarOverlay || typeof document === 'undefined') return

  const styles = getComputedStyle(document.documentElement)
  const color = styles.getPropertyValue('--background-dark').trim()
    || styles.getPropertyValue('--sidebar').trim()
    || styles.getPropertyValue('--background').trim()
  const symbolColor = styles.getPropertyValue('--foreground').trim()
  if (!color || !symbolColor) return

  void api.window.setTitleBarOverlay({ color, symbolColor })
}

export { isElectron }
