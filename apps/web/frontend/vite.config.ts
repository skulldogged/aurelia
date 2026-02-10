import { resolve } from 'path'
import { defineConfig, mergeConfig } from 'vite'
import { createSharedViteConfig, getDirname } from '@aurelia/shared/vite'

const __dirname = getDirname(import.meta)

export default defineConfig(
  mergeConfig(
    createSharedViteConfig({
      projectDir: __dirname,
      sharedDir: resolve(__dirname, '../../shared/src'),
    }),
    {
      build: {
        outDir: 'dist',
      },
      server: {
        host: true,
        port: 5173,
        proxy: {
          '/api': {
            changeOrigin: true,
            target: 'http://localhost:3000',
          },
          '/ws': {
            target: 'http://localhost:3000',
            ws: true,
          },
        },
        strictPort: true,
      },
    },
  ),
)
