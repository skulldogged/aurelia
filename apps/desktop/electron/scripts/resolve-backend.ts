import { execFileSync } from 'node:child_process'
import { existsSync } from 'node:fs'
import { homedir } from 'node:os'
import { delimiter, join } from 'node:path'

const isExecutable = (candidate: string): boolean => existsSync(candidate)

const fromPath = (name: string): string | undefined => {
  const pathValue = process.env.PATH
  if (!pathValue) return undefined
  for (const dir of pathValue.split(delimiter)) {
    const candidate = join(dir, name)
    if (isExecutable(candidate)) return candidate
  }
  return undefined
}

export const resolveCargo = (): string | undefined => {
  const home = process.env.HOME || homedir()
  const candidates = [
    process.env.CARGO,
    fromPath('cargo'),
    join(home, '.cargo/bin/cargo'),
  ].filter((value): value is string => Boolean(value))

  for (const candidate of candidates) {
    if (isExecutable(candidate)) return candidate
  }

  try {
    const fromRustup = execFileSync('rustup', ['which', 'cargo'], {
      encoding: 'utf8',
      stdio:    ['ignore', 'pipe', 'ignore'],
      timeout:  10_000,
    }).trim()
    if (fromRustup && isExecutable(fromRustup)) return fromRustup
  } catch {
    // rustup is optional
  }

  return undefined
}

export const resolveBackendCommand = (repoRoot: string, preferBuiltBinary: boolean): {
  args:    string[]
  command: string
  cwd?:    string
} => {
  const releaseBin = join(repoRoot, 'target/release/aurelia-web-backend')
  const debugBin = join(repoRoot, 'target/debug/aurelia-web-backend')

  if (preferBuiltBinary) {
    if (isExecutable(releaseBin)) return { args: [], command: releaseBin }
    if (isExecutable(debugBin)) return { args: [], command: debugBin }
  }

  const cargo = resolveCargo()
  if (cargo) {
    return {
      args:    ['run', '-p', 'aurelia-web-backend'],
      command: cargo,
      cwd:     repoRoot,
    }
  }

  if (isExecutable(releaseBin)) return { args: [], command: releaseBin }
  if (isExecutable(debugBin)) return { args: [], command: debugBin }

  throw new Error(
    'Could not find cargo or a built aurelia-web-backend binary. Install Rust or build the backend first.',
  )
}
