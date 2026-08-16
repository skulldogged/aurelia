import { spawn } from 'child_process'
import { dirname, resolve } from 'path'
import { fileURLToPath } from 'url'

import { bundleElectron } from './bundle'

const __dirname = dirname(fileURLToPath(import.meta.url))
const root = resolve(__dirname, '..')

const run = (command: string, args: string[]): Promise<void> => new Promise((resolvePromise, reject) => {
  const child = spawn(command, args, {
    cwd:   root,
    shell: false,
    stdio: 'inherit',
  })
  child.on('error', reject)
  child.on('exit', code => {
    if (code === 0) resolvePromise()
    else reject(new Error(`${command} ${args.join(' ')} failed with exit code ${code}`))
  })
})

await bundleElectron()
await run('bun', ['x', 'vite', 'build'])
console.log('Electron build complete: dist/ + dist-electron/')
