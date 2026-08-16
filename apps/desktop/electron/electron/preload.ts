import { contextBridge, ipcRenderer } from 'electron'

declare const __AURELIA_VERSION__: string

const onResized = (handler: () => void): (() => void) => {
  const listener = (): void => {
    handler()
  }
  ipcRenderer.on('desktop:window-resized', listener)
  return () => {
    ipcRenderer.removeListener('desktop:window-resized', listener)
  }
}

const onLastFmToken = (handler: (token: string) => void): (() => void) => {
  const listener = (_event: unknown, token: unknown): void => {
    if (typeof token === 'string' && token.length > 0) handler(token)
  }
  ipcRenderer.on('desktop:lastfm-token', listener)
  return () => {
    ipcRenderer.removeListener('desktop:lastfm-token', listener)
  }
}

contextBridge.exposeInMainWorld('aureliaDesktop', {
  appVersion: __AURELIA_VERSION__,
  lastFm:     {
    onToken:   onLastFmToken,
    startAuth: () => ipcRenderer.invoke('desktop:lastfm-start-auth') as Promise<{ callbackUrl: string }>,
  },
  openUrl:           (url: string) => ipcRenderer.invoke('desktop:open-url', url),
  platform:          process.platform,
  quit:              () => ipcRenderer.invoke('desktop:quit'),
  setCloseToTray:    (enabled: boolean) => ipcRenderer.invoke('desktop:set-close-to-tray', enabled),
  setMinimizeToTray: (enabled: boolean) => ipcRenderer.invoke('desktop:set-minimize-to-tray', enabled),
  window:            {
    close:          () => ipcRenderer.invoke('desktop:window-close'),
    hide:           () => ipcRenderer.invoke('desktop:window-hide'),
    isMaximized:    () => ipcRenderer.invoke('desktop:window-is-maximized'),
    minimize:       () => ipcRenderer.invoke('desktop:window-minimize'),
    onResized,
    show:           () => ipcRenderer.invoke('desktop:window-show'),
    setTitleBarOverlay: (options: { color: string; symbolColor: string }) =>
      ipcRenderer.invoke('desktop:set-titlebar-overlay', options),
    toggleMaximize: () => ipcRenderer.invoke('desktop:window-toggle-maximize'),
  },
})
