import { type ChildProcess, spawn } from 'child_process'
import { createConnection } from 'net'
import { dirname, resolve } from 'path'
import { fileURLToPath } from 'url'

import { bundleElectron } from './bundle'
import { resolveBackendCommand } from './resolve-backend'
import { electronDistDir, resolveElectronBinary } from './resolve-electron'

const __dirname = dirname(fileURLToPath(import.meta.url))
const root = resolve(__dirname, '..')
const repoRoot = resolve(root, '../../..')
const RENDERER_URL = 'http://127.0.0.1:3002'
const BACKEND_PORT = Number(process.env.AURELIA_PORT || 3000)

const waitForPort = (port: number, timeoutMs = 30000): Promise<void> => new Promise((resolveWait, reject) => {
  const startTime = Date.now()
  const tryConnect = (): void => {
    const socket = createConnection({ host: '127.0.0.1', port }, () => {
      socket.destroy()
      resolveWait()
    })
    socket.on('error', () => {
      socket.destroy()
      if (Date.now() - startTime > timeoutMs) {
        reject(new Error(`Port ${port} did not become available within ${timeoutMs}ms`))
        return
      }
      setTimeout(tryConnect, 300)
    })
  }
  tryConnect()
})

const children: ChildProcess[] = []

const spawnChild = (
  command: string,
  args: string[],
  extraEnv: Record<string, string> = {},
  cwd = root,
): ChildProcess => {
  const env = { ...process.env, ...extraEnv }
  // Agent shells and some desktop sessions set this, which makes Electron
  // behave like plain Node and hides the builtin `electron` module.
  delete env.ELECTRON_RUN_AS_NODE
  const child = spawn(command, args, {
    cwd,
    env,
    shell: false,
    stdio: 'inherit',
  })
  children.push(child)
  return child
}

const isPortOpen = (port: number): Promise<boolean> => new Promise(resolvePort => {
  const socket = createConnection({ host: '127.0.0.1', port }, () => {
    socket.destroy()
    resolvePort(true)
  })
  socket.on('error', () => {
    socket.destroy()
    resolvePort(false)
  })
})

const shutdown = (): void => {
  for (const child of children) {
    if (!child.killed) child.kill('SIGTERM')
  }
}

process.on('SIGINT', () => {
  shutdown()
  process.exit(0)
})
process.on('SIGTERM', () => {
  shutdown()
  process.exit(0)
})

await bundleElectron()

if (await isPortOpen(BACKEND_PORT)) {
  console.log(
    `Reusing existing Aurelia backend on 127.0.0.1:${BACKEND_PORT}. `
    + 'If /api/images or /api/auth/provider-capabilities 404, stop that process and rerun so cargo can start a current backend.',
  )
} else {
  const backendCommand = resolveBackendCommand(repoRoot, false)
  console.log(`Starting Aurelia backend: ${backendCommand.command} ${backendCommand.args.join(' ')}`)
  const backend = spawnChild(backendCommand.command, backendCommand.args, {
    AURELIA_HOST: '127.0.0.1',
    AURELIA_PORT: String(BACKEND_PORT),
  }, backendCommand.cwd ?? repoRoot)
  backend.on('error', error => {
    console.error(`Failed to spawn Aurelia backend (${backendCommand.command}):`, error)
  })
  backend.on('exit', code => {
    if (code !== 0 && code !== null && !backend.killed) {
      console.error(`Aurelia backend exited with code ${code}`)
    }
  })
}

const vite = spawnChild('bun', ['x', 'vite', '--port', '3002', '--host', '127.0.0.1'])
vite.on('exit', code => {
  if (code !== 0 && code !== null) {
    shutdown()
    process.exit(code)
  }
})

await Promise.all([
  waitForPort(3002),
  waitForPort(BACKEND_PORT, 180000),
])

const electronBin = resolveElectronBinary(root)
const electron = spawnChild(electronBin, ['.'], {
  ELECTRON_OVERRIDE_DIST_PATH: process.env.ELECTRON_OVERRIDE_DIST_PATH || electronDistDir(electronBin),
  ELECTRON_RENDERER_URL:       RENDERER_URL,
})

electron.on('exit', code => {
  shutdown()
  process.exit(code ?? 0)
})
