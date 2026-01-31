#!/usr/bin/env node
import { spawn } from 'child_process'
import { join, resolve } from 'path'

const PLATFORM = process.argv.find(arg => arg.startsWith('--platform='))?.split('=')[1] || 'web'
const COMMAND = process.argv[2]
const SKIP_BINDINGS = process.argv.includes('--skip-bindings')

const ROOT = resolve(import.meta.dirname, '..')

const run = (cmd: string, args: string[], opts = {}): Promise<void> => new Promise((resolve, reject) => {
  const proc = spawn(cmd, args, { cwd: ROOT, shell: true, stdio: 'inherit', ...opts })
  proc.on('close', code => code === 0 ? resolve() : reject(new Error(`Exit ${code}`)))
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
  await run('bun', ['run', 'tauri', 'dev'], { cwd: join(ROOT, 'apps/desktop') })
}

const devAndroid = async (): Promise<void> => {
  await bindings()
  await run('cargo', [
    'ndk', '-t', 'arm64-v8a', '-t', 'x86_64', '-o',
    'apps/mobile/android/app/src/main/jniLibs',
    'build', '-p', 'aurelia-core', '--release',
  ])
  await run('cargo', [
    'run', '-p', 'uniffi-bindgen', '--', 'generate',
    '--library', 'target/debug/aurelia_core.dll',
    '--language', 'kotlin',
    '--out-dir', 'apps/mobile/android/app/src/main/java',
    '--no-format',
  ])
  await run('./gradlew', ['assembleDebug'], { cwd: join(ROOT, 'apps/mobile/android') })
}

const buildWeb = async (): Promise<void> => {
  await bindings()
  await run('bun', ['run', 'build'], { cwd: join(ROOT, 'apps/web/frontend') })
  await run('cargo', ['build', '-p', 'aurelia-web-backend', '--release'])
  console.log('Build complete: apps/web/frontend/dist/')
}

const buildDesktop = async (): Promise<void> => {
  await bindings()
  await run('bun', ['run', 'tauri', 'build'], { cwd: join(ROOT, 'apps/desktop') })
}

const buildAndroid = async (): Promise<void> => {
  await bindings()
  await run('cargo', [
    'ndk', '-t', 'arm64-v8a', '-t', 'x86_64', '-o',
    'apps/mobile/android/app/src/main/jniLibs',
    'build', '-p', 'aurelia-core', '--release',
  ])
  await run('cargo', [
    'run', '-p', 'uniffi-bindgen', '--', 'generate',
    '--library', 'target/debug/aurelia_core.dll',
    '--language', 'kotlin',
    '--out-dir', 'apps/mobile/android/app/src/main/java',
    '--no-format',
  ])
  await run('./gradlew', ['assembleRelease'], { cwd: join(ROOT, 'apps/mobile/android') })
}

const commands = {
  build: { android: buildAndroid, desktop: buildDesktop, web: buildWeb },
  dev:   { android: devAndroid, desktop: devDesktop, web: devWeb },
}

if (!commands[COMMAND]?.[PLATFORM]) {
  console.log('Usage: bun run dev|build --platform=web|desktop|android')
  process.exit(1)
}

commands[COMMAND][PLATFORM]().catch(e => {
  console.error(e.message)
  process.exit(1)
})
