import { build, type BuildOptions } from 'esbuild'
import { readFileSync } from 'node:fs'
import { dirname, resolve } from 'path'
import { fileURLToPath } from 'url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const root = resolve(__dirname, '..')
const pkg = JSON.parse(readFileSync(resolve(root, 'package.json'), 'utf8')) as { version: string }

const sharedOptions: BuildOptions = {
  bundle:       true,
  external:     ['electron'],
  format:       'cjs',
  outdir:       resolve(root, 'dist-electron'),
  outExtension: { '.js': '.cjs' },
  platform:     'node',
  sourcemap:    true,
  target:       'node20',
}

export const bundleElectron = async (): Promise<void> => {
  await Promise.all([
    build({
      ...sharedOptions,
      entryPoints: { main: resolve(root, 'electron/main.ts') },
    }),
    build({
      ...sharedOptions,
      define: {
        __AURELIA_VERSION__: JSON.stringify(pkg.version),
      },
      entryPoints: { preload: resolve(root, 'electron/preload.ts') },
    }),
  ])

  const preload = readFileSync(resolve(root, 'dist-electron/preload.cjs'), 'utf8')
  if (preload.includes('import_meta_url') || preload.includes('require(\'url\')')) {
    throw new Error(
      'Electron preload bundle must not use Node url polyfills; sandboxed preload scripts cannot load them.',
    )
  }
}

if (import.meta.main) {
  await bundleElectron()
}
