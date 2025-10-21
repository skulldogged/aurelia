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
    VueRouter({}),
    vue(),
    tailwindcss(),
  ],

  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
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
    port:       3000,
    strictPort: true,
    watch:      {
      ignored: ['**/src-tauri/**'],
    },
  },
}))
