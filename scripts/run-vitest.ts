import { spawn } from 'node:child_process'
import { mkdirSync } from 'node:fs'
import { createRequire } from 'node:module'
import { join } from 'node:path'

const args = process.argv.slice(2)
const vitestArgs = args.length > 0 ? args : ['run']

const require = createRequire(import.meta.url)
const vitestEntry = require.resolve('vitest/vitest.mjs', { paths: [process.cwd()] })

const cacheDir = join(process.cwd(), '.cache')
mkdirSync(cacheDir, { recursive: true })

const storageFile = join(cacheDir, 'localstorage.json')
const nodeOptions = (process.env.NODE_OPTIONS ?? '')
  .split(' ')
  .filter(Boolean)
  .filter(option => !option.startsWith('--localstorage-file'))
  .concat(`--localstorage-file=${storageFile}`)
  .join(' ')

const nodeArgs = [vitestEntry, ...vitestArgs]

const child = spawn('node', nodeArgs, {
  cwd: process.cwd(),
  env: {
    ...process.env,
    NODE_OPTIONS: nodeOptions,
  },
  stdio: 'inherit',
})

child.on('exit', code => {
  process.exit(code ?? 1)
})
