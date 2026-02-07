import tailwindcss from '@tailwindcss/vite'
import vue from '@vitejs/plugin-vue'
import path from 'path'
import VueRouter from 'unplugin-vue-router/vite'
import { defineConfig } from 'vite'

const host = process.env.TAURI_DEV_HOST

export default defineConfig(async () => ({
  build: {
    chunkSizeWarningLimit: 1000,
  },
  clearScreen: false,
  css:         {
    lightningcss: {
      nonStandard: {
        pseudoClasses: true,
      },
    },
  },

  plugins: [
    // VueRouter must be before vue()
    VueRouter({
      exclude:      ['**/node_modules/**', '**/components/**'],
      routesFolder: [
        {
          path: '',
          src:  '../../shared/src/pages',
        },
      ],
    }),
    vue(),
    tailwindcss(),
  ],

  resolve: {
    alias: {
      '@':       path.resolve(__dirname, './src'),
      '@shared': path.resolve(__dirname, '../../shared/src'),
    },
  },
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
  test: {
    environment: 'happy-dom',
    globals:     true,
    include:     [
      'tests/**/*.spec.ts',
      'tests/**/*.test.ts',
      'src/**/*.spec.ts',
      'src/**/*.test.ts',
    ],
    setupFiles: ['./tests/setup.ts'],
  },
}))
