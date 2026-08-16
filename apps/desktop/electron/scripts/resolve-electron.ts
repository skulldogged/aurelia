import { execFileSync } from 'node:child_process'
import { existsSync } from 'node:fs'
import { createRequire } from 'node:module'

const require = createRequire(import.meta.url)

export const resolveElectronBinary = (): string => {
  const envPath = process.env.ELECTRON_EXEC_PATH || process.env.ELECTRON_PATH
  if (envPath && existsSync(envPath)) return envPath

  // The electron package exports the real Chromium path, not the bun/npm shim.
  try {
    const fromPackage = require('electron') as string
    if (fromPackage && existsSync(fromPackage)) return fromPackage
  } catch {
    // Fall through to PATH.
  }

  try {
    const fromPath = execFileSync(process.platform === 'win32' ? 'where' : 'which', ['electron'], {
      encoding: 'utf8',
    }).split(/\r?\n/)[0]?.trim()
    if (fromPath && existsSync(fromPath)) return fromPath
  } catch {
    // no system electron
  }

  throw new Error(
    'Electron binary not found. Run bun install from the repo root.',
  )
}
