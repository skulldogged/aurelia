import { execFileSync } from 'node:child_process'
import { existsSync } from 'node:fs'
import { delimiter, dirname, join, resolve } from 'node:path'

const isExecutable = (candidate: string): boolean => existsSync(candidate)

const fromOverrideDist = (distPath: string | undefined): string | undefined => {
  if (!distPath) return undefined
  const candidate = join(distPath, process.platform === 'win32' ? 'electron.exe' : 'electron')
  return isExecutable(candidate) ? candidate : undefined
}

const fromPath = (): string | undefined => {
  const pathValue = process.env.PATH
  if (!pathValue) return undefined
  const names = process.platform === 'win32' ? ['electron.cmd', 'electron.exe', 'electron'] : ['electron']
  for (const dir of pathValue.split(delimiter)) {
    for (const name of names) {
      const candidate = resolve(dir, name)
      if (isExecutable(candidate)) return candidate
    }
  }
  return undefined
}

const fromNix = (): string | undefined => {
  if (process.platform !== 'linux') return undefined
  try {
    const outPath = execFileSync('nix', ['eval', '--raw', 'nixpkgs#electron.outPath'], {
      encoding: 'utf8',
      stdio:    ['ignore', 'pipe', 'ignore'],
      timeout:  30_000,
    }).trim()
    const candidate = join(outPath, 'bin', 'electron')
    return isExecutable(candidate) ? candidate : undefined
  } catch {
    return undefined
  }
}

export const resolveElectronBinary = (packageRoot: string): string => {
  const envPath = process.env.ELECTRON_EXEC_PATH || process.env.ELECTRON_PATH
  if (envPath && isExecutable(envPath)) return envPath

  const override = fromOverrideDist(process.env.ELECTRON_OVERRIDE_DIST_PATH)
  if (override) return override

  const onPath = fromPath()
  if (onPath) return onPath

  const localCandidates = [
    resolve(packageRoot, 'node_modules/electron/dist/electron'),
    resolve(packageRoot, 'node_modules/.bin/electron'),
    resolve(packageRoot, '../../../node_modules/electron/dist/electron'),
    resolve(packageRoot, '../../../node_modules/.bin/electron'),
  ]
  const local = localCandidates.find(candidate => isExecutable(candidate))
  if (local) return local

  const nixElectron = fromNix()
  if (nixElectron) return nixElectron

  throw new Error(
    'Electron binary not found. On NixOS, add electron to the flake shell or set ELECTRON_EXEC_PATH. Otherwise run bun install from the repo root.',
  )
}

export const electronDistDir = (electronBin: string): string => dirname(electronBin)
