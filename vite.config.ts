import tailwindcss from '@tailwindcss/vite'
import vue from '@vitejs/plugin-vue'
import path from 'path'
import { defineConfig } from 'vite'
import { VitePWA } from 'vite-plugin-pwa'

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST

export default defineConfig(async () => ({
  clearScreen: false,
  css:         {
    lightningcss: {
      nonStandard: {
        pseudoClasses: true,
      },
    },
  },

  plugins: [
    vue(),
    tailwindcss(),
    VitePWA({
      registerType: 'autoUpdate',
      workbox:      {
        globPatterns:   ['**/*.{js,css,html,ico,png,svg}'],
        runtimeCaching: [
          {
            handler: 'StaleWhileRevalidate',
            options: {
              cacheableResponse: {
                statuses: [0, 200],
              },
              cacheName:  'jellyfin-image-cache',
              expiration: {
                maxAgeSeconds: 60 * 60 * 24 * 30,
                maxEntries:    2000,
              },
            },
            urlPattern: /\/Items\/[a-f0-9]+\/Images\/(Primary|Backdrop|Logo)/,
          },
        ],
      },
    }),
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
