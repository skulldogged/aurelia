#!/usr/bin/env bun
import { spawn } from 'child_process'
import { existsSync } from 'fs'
import { resolve } from 'path'

const ROOT = resolve(import.meta.dirname, '..')
const GPUI_PACKAGE = 'aurelia-gpui'

const command = process.argv[2] ?? 'run'
const passthroughArgs = process.argv.slice(3)

const defaultTargetDir = (): string => {
  if (process.env.CARGO_TARGET_DIR) {
    return process.env.CARGO_TARGET_DIR
  }

  if (process.platform === 'win32' && existsSync('D:\\')) {
    return 'D:\\aurelia-cargo-target'
  }

  return resolve(ROOT, 'target')
}

const cargoArgsFor = (cmd: string): string[] => {
  switch (cmd) {
    case 'build':
      return ['build', '-p', GPUI_PACKAGE]
    case 'check':
      return ['check', '-p', GPUI_PACKAGE]
    case 'run':
      return ['run', '-p', GPUI_PACKAGE]
    case 'test':
      return ['test', '-p', GPUI_PACKAGE]
    default:
      throw new Error(`Unknown GPUI command: ${cmd}`)
  }
}

const child = spawn('cargo', [...cargoArgsFor(command), ...passthroughArgs], {
  cwd: ROOT,
  env: {
    ...process.env,
    CARGO_TARGET_DIR: defaultTargetDir(),
  },
  shell: true,
  stdio: 'inherit',
})

child.on('exit', code => {
  process.exit(code ?? 1)
})
