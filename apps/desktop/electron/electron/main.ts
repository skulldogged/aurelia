import { app, BrowserWindow, ipcMain, Menu, nativeImage, shell, Tray } from 'electron'
import { type ChildProcess, execFileSync, spawn } from 'node:child_process'
import { existsSync, readFileSync } from 'node:fs'
import { homedir } from 'node:os'
import { createServer, type Server } from 'node:http'
import { createConnection } from 'node:net'
import path from 'node:path'

const RENDERER_URL = process.env.ELECTRON_RENDERER_URL
const IS_DEV = Boolean(RENDERER_URL)
const BACKEND_HOST = '127.0.0.1'
const BACKEND_PORT = Number(process.env.AURELIA_PORT || 3000)
const REPO_ROOT = path.resolve(__dirname, '../../../..')
const ICON_CANDIDATES = [
  path.join(REPO_ROOT, 'apps/desktop/electron/public/icon.png'),
  path.join(REPO_ROOT, 'apps/desktop/electron/public/app-icon.png'),
]

let mainWindow: BrowserWindow | null = null
let tray: null | Tray = null
let backendProcess: ChildProcess | null = null
let lastFmAuthServer: null | Server = null
let closeToTray = false
let minimizeToTray = false
let isQuitting = false

const waitForPort = (port: number, timeoutMs = 120000): Promise<void> => new Promise((resolve, reject) => {
  const startTime = Date.now()
  const tryConnect = (): void => {
    const socket = createConnection({ host: BACKEND_HOST, port }, () => {
      socket.destroy()
      resolve()
    })
    socket.on('error', () => {
      socket.destroy()
      if (Date.now() - startTime > timeoutMs) {
        reject(new Error(`Backend did not start on ${BACKEND_HOST}:${port} within ${timeoutMs}ms`))
        return
      }
      setTimeout(tryConnect, 400)
    })
  }
  tryConnect()
})

const isPortOpen = (port: number): Promise<boolean> => new Promise(resolve => {
  const socket = createConnection({ host: BACKEND_HOST, port }, () => {
    socket.destroy()
    resolve(true)
  })
  socket.on('error', () => {
    socket.destroy()
    resolve(false)
  })
})

const resolveIconPath = (): string | undefined => ICON_CANDIDATES.find(candidate => existsSync(candidate))

const findRepoRoot = (): string => {
  let dir = __dirname
  for (let i = 0; i < 8; i++) {
    const cargoToml = path.join(dir, 'Cargo.toml')
    const electronApp = path.join(dir, 'apps/desktop/electron')
    if (existsSync(cargoToml) && existsSync(electronApp)) return dir
    const parent = path.dirname(dir)
    if (parent === dir) break
    dir = parent
  }
  return REPO_ROOT
}

const resolveBackendBinary = (): string | undefined => {
  const repoRoot = findRepoRoot()
  return [
    process.env.AURELIA_BACKEND,
    path.join(repoRoot, 'target/release/aurelia-web-backend'),
    path.join(repoRoot, 'target/debug/aurelia-web-backend'),
  ].find((candidate): candidate is string => Boolean(candidate && existsSync(candidate)))
}

const startBackend = async (): Promise<void> => {
  if (await isPortOpen(BACKEND_PORT)) {
    console.log(`Reusing existing Aurelia backend on ${BACKEND_HOST}:${BACKEND_PORT}`)
    return
  }

  if (IS_DEV) {
    console.warn(
      `No Aurelia backend on ${BACKEND_HOST}:${BACKEND_PORT}. `
      + 'The Electron window will still open; start aurelia-web-backend from the dev script.',
    )
    return
  }

  const command = resolveBackendBinary()
  if (!command) {
    throw new Error(
      'Could not find aurelia-web-backend. Build it with `cargo build -p aurelia-web-backend` or set AURELIA_BACKEND.',
    )
  }

  const rendererDist = path.join(__dirname, '../dist')
  const env = {
    ...process.env,
    AURELIA_DATA_DIR: path.join(app.getPath('userData'), 'data'),
    AURELIA_HOST:     BACKEND_HOST,
    AURELIA_PORT:     String(BACKEND_PORT),
    ...(existsSync(rendererDist) ? { AURELIA_STATIC_DIR: rendererDist } : {}),
  }

  console.log(`Starting Aurelia backend: ${command}`)
  backendProcess = spawn(command, [], {
    env,
    stdio: 'inherit',
  })
  backendProcess.on('error', error => {
    console.error('Failed to spawn Aurelia backend:', error)
  })
  backendProcess.on('exit', (code, signal) => {
    if (!isQuitting) {
      console.error(`Aurelia backend exited unexpectedly (code=${code}, signal=${signal})`)
    }
    backendProcess = null
  })

  await waitForPort(BACKEND_PORT)
}

const stopBackend = (): void => {
  if (!backendProcess || backendProcess.killed) return
  backendProcess.kill('SIGTERM')
  backendProcess = null
}

const showMainWindow = (): void => {
  if (!mainWindow) return
  if (mainWindow.isMinimized()) mainWindow.restore()
  mainWindow.show()
  mainWindow.focus()
}

const createTray = (): void => {
  if (tray) return
  const iconPath = resolveIconPath()
  const image = iconPath ? nativeImage.createFromPath(iconPath) : nativeImage.createEmpty()
  tray = new Tray(image.isEmpty() ? nativeImage.createFromDataURL('data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==') : image)
  tray.setToolTip('Aurelia')
  tray.setContextMenu(Menu.buildFromTemplate([
    { click: showMainWindow, label: 'Show Aurelia' },
    { type: 'separator' },
    {
      click: () => {
        isQuitting = true
        app.quit()
      },
      label: 'Quit',
    },
  ]))
  tray.on('click', showMainWindow)
}

const TITLEBAR_HEIGHT = 48
const DEFAULT_OVERLAY_COLOR = '#0a0a0a'
const DEFAULT_OVERLAY_SYMBOL = '#e0e0e0'
const WINDOW_CONTROL_BUTTONS = new Set(['close', 'maximize', 'minimize'])

const isCssColor = (value: unknown): value is string =>
  typeof value === 'string' && /^(#|rgb|hsl|oklch)/i.test(value.trim())

const parseDecorationLayout = (layout: string): boolean => {
  const buttons = layout
    .replace(/['"]/g, '')
    .split(':')
    .flatMap(side => side.split(','))
    .map(token => token.trim().toLowerCase())
    .filter(Boolean)

  return buttons.some(button => WINDOW_CONTROL_BUTTONS.has(button))
}

const readGtkDecorationLayout = (): string | undefined => {
  const configHome = process.env.XDG_CONFIG_HOME || path.join(homedir(), '.config')
  const candidates = [
    path.join(configHome, 'gtk-4.0/settings.ini'),
    path.join(configHome, 'gtk-3.0/settings.ini'),
    '/etc/gtk-4.0/settings.ini',
    '/etc/gtk-3.0/settings.ini',
  ]

  for (const file of candidates) {
    if (!existsSync(file)) continue
    const match = readFileSync(file, 'utf8').match(/^\s*gtk-decoration-layout\s*=\s*(.+)$/m)
    if (match?.[1]) return match[1].trim()
  }

  return undefined
}

const readGsettingsButtonLayout = (): string | undefined => {
  try {
    return execFileSync('gsettings', ['get', 'org.gnome.desktop.wm.preferences', 'button-layout'], {
      encoding: 'utf8',
      timeout:  500,
    }).trim()
  } catch {
    return undefined
  }
}

const shouldShowWindowControls = (): boolean => {
  if (process.platform !== 'linux') return true
  const layout = readGtkDecorationLayout() ?? readGsettingsButtonLayout()
  if (layout === undefined) return true
  return parseDecorationLayout(layout)
}

const showNativeWindowControls = shouldShowWindowControls()

const createWindow = async (): Promise<void> => {
  const iconPath = resolveIconPath()
  mainWindow = new BrowserWindow({
    autoHideMenuBar: true,
    backgroundColor: '#121212',
    height:          720,
    icon:            iconPath,
    minHeight:       350,
    minWidth:        450,
    show:            false,
    title:         'Aurelia',
    titleBarStyle: process.platform === 'darwin' ? 'hiddenInset' : 'hidden',
    ...(process.platform === 'darwin'
      ? {
          titleBarOverlay:      { height: TITLEBAR_HEIGHT },
          trafficLightPosition: { x: 16, y: 16 },
        }
      : showNativeWindowControls
        ? {
            titleBarOverlay: {
              color:       DEFAULT_OVERLAY_COLOR,
              height:      TITLEBAR_HEIGHT,
              symbolColor: DEFAULT_OVERLAY_SYMBOL,
            },
          }
        : {}),
    webPreferences: {
      backgroundThrottling: false,
      contextIsolation:     true,
      nodeIntegration:      false,
      preload:              path.join(__dirname, 'preload.cjs'),
      sandbox:              true,
    },
    width: 1280,
  })

  mainWindow.setMenuBarVisibility(false)

  mainWindow.on('maximize', () => {
    mainWindow?.webContents.send('desktop:window-resized')
  })
  mainWindow.on('unmaximize', () => {
    mainWindow?.webContents.send('desktop:window-resized')
  })
  mainWindow.on('resize', () => {
    mainWindow?.webContents.send('desktop:window-resized')
  })

  mainWindow.on('close', event => {
    if (!isQuitting && closeToTray) {
      event.preventDefault()
      mainWindow?.hide()
    }
  })

  mainWindow.on('minimize', () => {
    if (!minimizeToTray) return
    mainWindow?.hide()
  })

  mainWindow.webContents.setWindowOpenHandler(({ url }) => {
    void shell.openExternal(url)
    return { action: 'deny' }
  })

  mainWindow.webContents.on('did-fail-load', (_event, code, description, url) => {
    console.error(`Failed to load ${url}: ${description} (${code})`)
  })

  mainWindow.webContents.on('preload-error', (_event, preloadPath, error) => {
    console.error(`Failed to load preload script ${preloadPath}:`, error)
  })

  mainWindow.webContents.on('before-input-event', (_event, input) => {
    if (input.type === 'keyDown' && input.control && input.shift && input.key.toLowerCase() === 'i') {
      mainWindow?.webContents.toggleDevTools()
    }
  })

  const reveal = (): void => {
    if (!mainWindow || mainWindow.isDestroyed()) return
    mainWindow.show()
    mainWindow.focus()
  }

  mainWindow.once('ready-to-show', reveal)

  if (IS_DEV && RENDERER_URL) {
    await mainWindow.loadURL(RENDERER_URL)
    if (process.env.ELECTRON_DEVTOOLS === '1') {
      mainWindow.webContents.openDevTools({ mode: 'detach' })
    }
  } else {
    await mainWindow.loadURL(`http://${BACKEND_HOST}:${BACKEND_PORT}`)
  }

  // ready-to-show is not reliable on Linux; it can fire before we subscribe.
  reveal()
}

const registerIpc = (): void => {
  ipcMain.handle('desktop:window-close', () => {
    if (closeToTray) {
      mainWindow?.hide()
      return
    }
    isQuitting = true
    mainWindow?.close()
  })
  ipcMain.handle('desktop:window-hide', () => {
    mainWindow?.hide()
  })
  ipcMain.handle('desktop:window-show', () => {
    showMainWindow()
  })
  ipcMain.handle('desktop:window-minimize', () => {
    if (minimizeToTray) {
      mainWindow?.hide()
      return
    }
    mainWindow?.minimize()
  })
  ipcMain.handle('desktop:window-toggle-maximize', () => {
    if (!mainWindow) return
    if (mainWindow.isMaximized()) mainWindow.unmaximize()
    else mainWindow.maximize()
  })
  ipcMain.handle('desktop:window-is-maximized', () => mainWindow?.isMaximized() ?? false)
  ipcMain.handle('desktop:open-url', async (_event, url: string) => {
    if (typeof url === 'string' && (url.startsWith('https://') || url.startsWith('http://'))) {
      await shell.openExternal(url)
    }
  })
  ipcMain.handle('desktop:quit', () => {
    isQuitting = true
    app.quit()
  })
  ipcMain.handle('desktop:set-close-to-tray', (_event, enabled: boolean) => {
    closeToTray = Boolean(enabled)
  })
  ipcMain.handle('desktop:set-minimize-to-tray', (_event, enabled: boolean) => {
    minimizeToTray = Boolean(enabled)
  })
  ipcMain.handle('desktop:lastfm-start-auth', async () => startLastFmAuthServer())
  ipcMain.handle('desktop:set-titlebar-overlay', (_event, options: unknown) => {
    if (!mainWindow || mainWindow.isDestroyed() || process.platform === 'darwin' || !showNativeWindowControls) return
    if (!options || typeof options !== 'object') return
    const { color, symbolColor } = options as { color?: unknown; symbolColor?: unknown }
    mainWindow.setTitleBarOverlay({
      ...(isCssColor(color) ? { color } : {}),
      ...(isCssColor(symbolColor) ? { symbolColor } : {}),
      height: TITLEBAR_HEIGHT,
    })
  })
}

const stopLastFmAuthServer = (): void => {
  if (!lastFmAuthServer) return
  lastFmAuthServer.close()
  lastFmAuthServer = null
}

const startLastFmAuthServer = (): Promise<{ callbackUrl: string }> => {
  stopLastFmAuthServer()

  return new Promise((resolve, reject) => {
    const server = createServer((req, res) => {
      const requestUrl = new URL(req.url ?? '/', 'http://127.0.0.1')
      const token = requestUrl.searchParams.get('token')
      res.writeHead(200, { 'Content-Type': 'text/html; charset=utf-8' })
      res.end('Last.fm authorization received. You can close this window.')
      if (token && mainWindow && !mainWindow.isDestroyed()) {
        mainWindow.webContents.send('desktop:lastfm-token', token)
      }
      stopLastFmAuthServer()
    })

    server.once('error', error => {
      lastFmAuthServer = null
      reject(error)
    })
    server.listen(0, '127.0.0.1', () => {
      const address = server.address()
      if (!address || typeof address === 'string') {
        server.close()
        reject(new Error('Failed to bind Last.fm callback server'))
        return
      }
      lastFmAuthServer = server
      resolve({ callbackUrl: `http://127.0.0.1:${address.port}` })
    })
  })
}

app.commandLine.appendSwitch('autoplay-policy', 'no-user-gesture-required')
if (process.platform === 'linux') {
  app.commandLine.appendSwitch('gtk-version', '3')
}

const gotLock = app.requestSingleInstanceLock()
if (!gotLock) {
  app.quit()
} else {
  app.on('second-instance', () => {
    showMainWindow()
  })

  app.whenReady().then(async () => {
    registerIpc()
    createTray()
    await startBackend()
    await createWindow()

    app.on('activate', () => {
      if (BrowserWindow.getAllWindows().length === 0) {
        void createWindow()
        return
      }
      showMainWindow()
    })
  }).catch(error => {
    console.error('Failed to start Electron shell:', error)
    app.quit()
  })
}

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit()
  }
})

app.on('before-quit', () => {
  isQuitting = true
  stopLastFmAuthServer()
  stopBackend()
})
