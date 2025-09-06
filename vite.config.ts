import { defineConfig } from 'vite'
import path from 'path'
import vue from '@vitejs/plugin-vue'
import tailwindcss from '@tailwindcss/vite'
import { VitePWA } from 'vite-plugin-pwa'

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [
    vue(),
    tailwindcss(),
    VitePWA({
      registerType: 'autoUpdate',
      workbox:      {
        // You can keep globPatterns to pre-cache your app's shell (JS, CSS, etc.)
        globPatterns:   ['**/*.{js,css,html,ico,png,svg}'],
        // This is where the magic happens for dynamic URLs
        runtimeCaching: [
          {
            urlPattern: /\/Items\/[a-f0-9]+\/Images\/(Primary|Backdrop|Logo)/,
            handler:    'StaleWhileRevalidate',
            options:    {
              cacheName:  'jellyfin-image-cache',
              // END ADDITION
              expiration: {
                maxEntries:    2000,
                maxAgeSeconds: 60 * 60 * 24 * 30,
              },
              cacheableResponse: {
                statuses: [0, 200],
              },
            },
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

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server:      {
    port:       1420,
    strictPort: true,
    host:       host || false,
    hmr:        host
      ? {
        protocol: 'ws',
        host,
        port:     1421,
      }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ['**/src-tauri/**'],
    },
  },
}))
