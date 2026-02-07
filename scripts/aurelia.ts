#!/usr/bin/env node
import { spawn } from 'child_process'
import { existsSync } from 'fs'
import { join, resolve } from 'path'

const PLATFORM = process.argv.find(arg => arg.startsWith('--platform='))?.split('=')[1] || 'web'
const COMMAND = process.argv[2]
const SKIP_BINDINGS = process.argv.includes('--skip-bindings')

const ROOT = resolve(import.meta.dirname, '..')
const TAURI_DESKTOP_DIR = join(ROOT, 'apps/desktop/tauri')
const MACOS_DESKTOP_DIR = join(ROOT, 'apps/desktop/macos')
const MACOS_PACKAGE = join(MACOS_DESKTOP_DIR, 'Package.swift')
const MACOS_XCODE_PROJECT = join(MACOS_DESKTOP_DIR, 'AureliaMac.xcodeproj')
const MACOS_XCODEGEN_SPEC = join(MACOS_DESKTOP_DIR, 'project.yml')
const IOS_RUST_BUILD_SCRIPT = join(ROOT, 'apps/mobile/ios/build-rust.sh')
const IOS_XCFRAMEWORK = join(ROOT, 'apps/mobile/ios/AureliaCore/AureliaCoreFFI.xcframework')
const MACOS_SLICE = process.arch === 'arm64' ? 'macos-arm64' : 'macos-x86_64'
const IOS_XCFRAMEWORK_MACOS_SLICE = join(IOS_XCFRAMEWORK, MACOS_SLICE)

const run = (cmd: string, args: string[], opts = {}): Promise<void> => new Promise((resolve, reject) => {
  const proc = spawn(cmd, args, { cwd: ROOT, shell: true, stdio: 'inherit', ...opts })
  proc.on('close', code => code === 0 ? resolve() : reject(new Error(`Exit ${code}`)))
})

const canRun = (cmd: string): Promise<boolean> => new Promise(resolve => {
  const proc = spawn('bash', ['-lc', `command -v ${cmd} >/dev/null 2>&1`], {
    cwd: ROOT,
    stdio: 'ignore',
  })
  proc.on('close', code => resolve(code === 0))
})

const bindings = async (): Promise<void> => {
  if (SKIP_BINDINGS) return
  console.log('Generating bindings...')
  try {
    await run('cargo', ['build', '-p', 'uniffi-bindgen'])
    await run('cargo', ['run', '-p', 'uniffi-bindgen', '--', 'all', '--out-dir', 'apps/shared/src/generated'])
    await run('cargo', ['build', '-p', 'aurelia-api', '--features', 'web'])
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
  await run('bun', ['run', 'build'], { cwd: join(ROOT, 'apps/web/frontend') })
  await run('cargo', ['build', '-p', 'aurelia-web-backend', '--release'])
  console.log('Build complete: apps/web/frontend/dist/')
}

const buildDesktop = async (): Promise<void> => {
  await bindings()
  await run('bun', ['run', 'tauri', 'build'], { cwd: TAURI_DESKTOP_DIR })
}

const ensureMacosPackage = (): void => {
  if (!existsSync(MACOS_PACKAGE)) {
    throw new Error(
      `Native macOS package is not scaffolded yet. Expected: ${MACOS_PACKAGE}`
    )
  }
}

const ensureMacosXcodeProject = async (): Promise<void> => {
  if (!existsSync(MACOS_XCODEGEN_SPEC)) {
    throw new Error(
      `Native macOS Xcode spec is missing. Expected: ${MACOS_XCODEGEN_SPEC}`
    )
  }

  if (!(await canRun('xcodegen'))) {
    throw new Error(
      'xcodegen is required to generate AureliaMac.xcodeproj. Install with: brew install xcodegen'
    )
  }

  await run('xcodegen', ['--spec', MACOS_XCODEGEN_SPEC], { cwd: MACOS_DESKTOP_DIR })

  if (!existsSync(MACOS_XCODE_PROJECT)) {
    throw new Error(
      `Failed to generate native macOS project. Expected: ${MACOS_XCODE_PROJECT}`
    )
  }
}

const ensureIosCoreXcframework = async (): Promise<void> => {
  if (existsSync(IOS_XCFRAMEWORK_MACOS_SLICE)) {
    return
  }

  console.log('Missing macOS slice in AureliaCoreFFI.xcframework; building iOS/macOS Rust artifacts...')
  await run('bash', [IOS_RUST_BUILD_SCRIPT])

  if (!existsSync(IOS_XCFRAMEWORK_MACOS_SLICE)) {
    throw new Error(
      `AureliaCoreFFI.xcframework is still missing required slice: ${IOS_XCFRAMEWORK_MACOS_SLICE}`
    )
  }
}

const devDesktopMacos = async (): Promise<void> => {
  await bindings()
  ensureMacosPackage()
  await ensureIosCoreXcframework()
  await ensureMacosXcodeProject()
  await run('open', [MACOS_XCODE_PROJECT])
}

const buildDesktopMacos = async (): Promise<void> => {
  await bindings()
  ensureMacosPackage()
  await ensureIosCoreXcframework()
  await ensureMacosXcodeProject()
  await run('xcodebuild', [
    '-project',
    MACOS_XCODE_PROJECT,
    '-scheme',
    'AureliaMac',
    '-configuration',
    'Debug',
    '-destination',
    'platform=macOS',
    'build',
  ])
}

const commands = {
  build: {
    desktop:       buildDesktop,
    'desktop-macos': buildDesktopMacos,
    'desktop-tauri': buildDesktop,
    web:           buildWeb,
  },
  dev: {
    desktop:       devDesktop,
    'desktop-macos': devDesktopMacos,
    'desktop-tauri': devDesktop,
    web:           devWeb,
  },
}

if (!commands[COMMAND]?.[PLATFORM]) {
  console.log('Usage: bun run dev|build --platform=web|desktop|desktop-tauri|desktop-macos')
  process.exit(1)
}

commands[COMMAND][PLATFORM]().catch(e => {
  console.error(e.message)
  process.exit(1)
})
