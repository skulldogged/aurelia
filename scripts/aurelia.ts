#!/usr/bin/env node
import { spawn, type ChildProcess, type SpawnOptions } from 'child_process'
import { createHash } from 'crypto'
import { existsSync } from 'fs'
import { mkdir, readdir, readFile, stat, writeFile } from 'fs/promises'
import { dirname, join, relative, resolve } from 'path'

const PLATFORM = process.argv.find(arg => arg.startsWith('--platform='))?.split('=')[1] || 'web'
const COMMAND = process.argv[2]
const SKIP_BINDINGS = process.argv.includes('--skip-bindings')
const FORCE_BINDINGS = process.argv.includes('--force-bindings')
const FAST_BUILD = process.argv.includes('--fast')

const ROOT = resolve(import.meta.dirname, '..')
const TAURI_DESKTOP_DIR = join(ROOT, 'apps/desktop/tauri')
const BINDINGS_STATE_FILE = join(ROOT, 'target/.aurelia/bindings-state.json')
const BINDINGS_INPUT_DIRS = [
  join(ROOT, 'crates/aurelia-api'),
  join(ROOT, 'crates/aurelia-api-macros'),
  join(ROOT, 'crates/aurelia-core'),
  join(ROOT, 'crates/uniffi-bindgen'),
]
const BINDINGS_INPUT_FILES = [
  join(ROOT, 'Cargo.toml'),
  join(ROOT, 'Cargo.lock'),
  join(ROOT, 'apps/web/backend/Cargo.toml'),
  join(ROOT, 'apps/desktop/tauri/src-tauri/Cargo.toml'),
]
const BINDINGS_OUTPUTS = [
  join(ROOT, 'apps/shared/src/generated/index.ts'),
  join(ROOT, 'apps/shared/src/generated/httpClient.ts'),
  join(ROOT, 'apps/shared/src/api/apiClient.ts'),
  join(ROOT, 'apps/shared/src/lib/api/types.ts'),
]

const run = (cmd: string, args: string[], opts: SpawnOptions = {}): Promise<void> =>
  new Promise((resolve, reject) => {
    const proc = spawn(cmd, args, { cwd: ROOT, shell: true, stdio: 'inherit', ...opts })
    proc.on('error', reject)
    proc.on('close', code => code === 0 ? resolve() : reject(new Error(`Exit ${code}`)))
  })

const runConcurrent = (
  commands: Array<{ args: string[]; cmd: string; cwd?: string; name: string }>
): Promise<void> => new Promise((resolve, reject) => {
  const procs: ChildProcess[] = []
  let settled = false
  let completed = 0

  const terminateOthers = (failedIndex: number): void => {
    for (const [index, proc] of procs.entries()) {
      if (index !== failedIndex && !proc.killed) {
        proc.kill('SIGTERM')
      }
    }
  }

  for (const [index, command] of commands.entries()) {
    const proc = spawn(command.cmd, command.args, {
      cwd: command.cwd ?? ROOT,
      shell: true,
      stdio: 'inherit',
    })
    procs.push(proc)

    proc.on('error', error => {
      if (settled) return
      settled = true
      terminateOthers(index)
      reject(error)
    })

    proc.on('close', code => {
      if (settled) return
      if (code !== 0) {
        settled = true
        terminateOthers(index)
        reject(new Error(`${command.name} failed with exit code ${code}`))
        return
      }

      completed += 1
      if (completed === commands.length) {
        settled = true
        resolve()
      }
    })
  }
})

interface BindingsState {
  fingerprint: string
  updatedAt: string
}

const rustBuildProfileArgs = (): string[] => FAST_BUILD ? ['--profile', 'local-release'] : ['--release']

const listFilesRecursively = async (dir: string): Promise<string[]> => {
  if (!existsSync(dir)) {
    return []
  }

  const entries = await readdir(dir, { withFileTypes: true })
  const files: string[] = []

  for (const entry of entries.sort((a, b) => a.name.localeCompare(b.name))) {
    const fullPath = join(dir, entry.name)
    if (entry.isDirectory()) {
      files.push(...await listFilesRecursively(fullPath))
    } else if (entry.isFile()) {
      files.push(fullPath)
    }
  }

  return files
}

const outputsExist = (): boolean => BINDINGS_OUTPUTS.every(path => existsSync(path))

const readBindingsState = async (): Promise<BindingsState | null> => {
  if (!existsSync(BINDINGS_STATE_FILE)) {
    return null
  }

  try {
    const stateRaw = await readFile(BINDINGS_STATE_FILE, 'utf8')
    const state = JSON.parse(stateRaw) as Partial<BindingsState>
    if (typeof state.fingerprint === 'string') {
      return {
        fingerprint: state.fingerprint,
        updatedAt: typeof state.updatedAt === 'string' ? state.updatedAt : '',
      }
    }
  } catch {
    return null
  }

  return null
}

const writeBindingsState = async (state: BindingsState): Promise<void> => {
  await mkdir(dirname(BINDINGS_STATE_FILE), { recursive: true })
  await writeFile(BINDINGS_STATE_FILE, `${JSON.stringify(state, null, 2)}\n`, 'utf8')
}

const computeBindingsFingerprint = async (): Promise<string> => {
  const hasher = createHash('sha256')
  const allFiles = new Set<string>()

  for (const inputFile of BINDINGS_INPUT_FILES) {
    if (existsSync(inputFile)) {
      allFiles.add(inputFile)
    }
  }

  for (const dir of BINDINGS_INPUT_DIRS) {
    for (const file of await listFilesRecursively(dir)) {
      allFiles.add(file)
    }
  }

  const sortedFiles = [...allFiles].sort()
  for (const file of sortedFiles) {
    const relPath = relative(ROOT, file)
    const fileStat = await stat(file)
    hasher.update(relPath)
    hasher.update('\0')
    hasher.update(fileStat.size.toString())
    hasher.update('\0')
    hasher.update(fileStat.mtimeMs.toString())
    hasher.update('\0')
    hasher.update(await readFile(file))
    hasher.update('\0')
  }

  return hasher.digest('hex')
}

const bindings = async (): Promise<void> => {
  if (SKIP_BINDINGS) return

  let fingerprint: string | null = null
  try {
    fingerprint = await computeBindingsFingerprint()
    if (!FORCE_BINDINGS) {
      const cachedState = await readBindingsState()
      if (cachedState?.fingerprint === fingerprint && outputsExist()) {
        console.log('Bindings are up to date; skipping generation.')
        return
      }
    }
  } catch (error) {
    console.log(`Could not compute bindings fingerprint (${String(error)}); regenerating.`)
  }

  console.log('Generating bindings...')
  try {
    await run('cargo', ['run', '-p', 'uniffi-bindgen', '--', 'all', '--out-dir', 'apps/shared/src/generated'])
    await run('cargo', ['check', '-p', 'aurelia-api', '--features', 'web'])

    const finalFingerprint = fingerprint ?? await computeBindingsFingerprint()
    await writeBindingsState({
      fingerprint: finalFingerprint,
      updatedAt:   new Date().toISOString(),
    })
  } catch {
    console.log('Binding generation failed, continuing...')
  }
}

const devWeb = async (): Promise<void> => {
  await bindings()
  console.log('Starting web dev server...')
  const backend = spawn('cargo', ['run', '-p', 'aurelia-web-backend'], { cwd: ROOT, shell: true, stdio: 'inherit' })
  await new Promise(r => setTimeout(r, 3000))
  try {
    await run('bun', ['run', 'dev'], { cwd: join(ROOT, 'apps/web/frontend') })
  } finally {
    backend.kill()
  }
}

const devDesktop = async (): Promise<void> => {
  await bindings()
  await run('bun', ['run', 'tauri', 'dev'], { cwd: TAURI_DESKTOP_DIR })
}

const buildWeb = async (): Promise<void> => {
  await bindings()
  await runConcurrent([
    {
      args: ['run', 'build'],
      cmd:  'bun',
      cwd:  join(ROOT, 'apps/web/frontend'),
      name: 'frontend build',
    },
    {
      args: ['build', '-p', 'aurelia-web-backend', ...rustBuildProfileArgs()],
      cmd:  'cargo',
      cwd:  ROOT,
      name: 'backend build',
    },
  ])
  console.log('Build complete: apps/web/frontend/dist/')
}

const buildDesktop = async (): Promise<void> => {
  await bindings()
  if (FAST_BUILD) {
    await run('bun', ['run', 'tauri', 'build', '--', '--profile', 'local-release'], { cwd: TAURI_DESKTOP_DIR })
    return
  }

  await run('bun', ['run', 'tauri', 'build'], { cwd: TAURI_DESKTOP_DIR })
}

const commands = {
  build: {
    desktop:       buildDesktop,
    'desktop-tauri': buildDesktop,
    web:           buildWeb,
  },
  dev: {
    desktop:       devDesktop,
    'desktop-tauri': devDesktop,
    web:           devWeb,
  },
}

if (!commands[COMMAND]?.[PLATFORM]) {
  console.log('Usage: bun run dev|build --platform=web|desktop|desktop-tauri [--skip-bindings] [--force-bindings] [--fast]')
  process.exit(1)
}

commands[COMMAND][PLATFORM]().catch(e => {
  console.error(e.message)
  process.exit(1)
})
