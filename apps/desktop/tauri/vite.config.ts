import { dirname, resolve } from 'path'
import { fileURLToPath } from 'url'
import { defineConfig, mergeConfig } from 'vite'

import { createSharedViteConfig } from '../../shared/src/lib/vite/index'

const __dirname = dirname(fileURLToPath(import.meta.url))
const host = process.env.TAURI_DEV_HOST

export default defineConfig(
  mergeConfig(
    createSharedViteConfig({
      projectDir: __dirname,
      sharedDir:  resolve(__dirname, '../../shared/src'),
    }),
    {
      server: {
        hmr: host
          ? {
            host,
            port:     3001,
            protocol: 'ws',
          }
          : undefined,
        host:       host || false,
        port:       3001,
        strictPort: true,
        watch:      {
          ignored: ['**/src-tauri/**'],
        },
      },
    },
  ),
)
