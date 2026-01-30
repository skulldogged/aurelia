import tailwindcss from '@tailwindcss/vite'
import vue from '@vitejs/plugin-vue'
import path from 'path'
import VueRouter from 'unplugin-vue-router/vite'
import { defineConfig } from 'vite'

export default defineConfig({
  build: {
    chunkSizeWarningLimit: 1000,
    outDir: 'dist',
  },
  clearScreen: false,
  css: {
    transformer: 'lightningcss',
  },

  plugins: [
    VueRouter({
      routesFolder: [
        {
          src: '../../shared/src/pages',
          path: '',
        }
      ],
      exclude: ['**/node_modules/**', '**/components/**'],
    }),
    vue(),
    tailwindcss(),
  ],

  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
      '@shared': path.resolve(__dirname, '../../shared/src'),
    },
  },
  server: {
    host: true,
    port: 5173,
    strictPort: true,
    proxy: {
      '/api': {
        target: 'http://localhost:3000',
        changeOrigin: true,
      },
      '/ws': {
        target: 'http://localhost:3000',
        ws: true,
      },
    },
  },
})
