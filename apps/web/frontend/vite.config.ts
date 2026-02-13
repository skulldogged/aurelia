import { dirname, resolve } from 'path'
import { fileURLToPath } from 'url'
import { defineConfig, mergeConfig } from 'vite'

import { createSharedViteConfig } from '../../shared/src/lib/vite/index'

const __dirname = dirname(fileURLToPath(import.meta.url))

export default defineConfig(
  mergeConfig(
    createSharedViteConfig({
      projectDir: __dirname,
      sharedDir:  resolve(__dirname, '../../shared/src'),
    }),
    {
      build: {
        outDir: 'dist',
      },
      server: {
        host:  true,
        port:  5173,
        proxy: {
          '/api': {
            changeOrigin: true,
            target:       'http://localhost:3000',
          },
          '/ws': {
            target: 'http://localhost:3000',
            ws:     true,
          },
        },
        strictPort: true,
      },
    },
  ),
)
