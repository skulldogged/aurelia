import tailwindcss from '@tailwindcss/vite'
import vue from '@vitejs/plugin-vue'
import { dirname, resolve } from 'path'
import { fileURLToPath } from 'url'
import VueRouter from 'unplugin-vue-router/vite'
import type { UserConfig, UserConfigExport } from 'vitest/config'

export interface SharedViteConfigOptions {
  projectDir: string
  sharedDir?: string
}

export const createSharedViteConfig = (options: SharedViteConfigOptions): UserConfig => {
  const { projectDir, sharedDir } = options
  const resolvedSharedDir = sharedDir ?? resolve(projectDir, '../shared/src')

  return {
    build: {
      chunkSizeWarningLimit: 1000,
    },
    clearScreen: false,
    plugins: [
      VueRouter({
        exclude: ['**/node_modules/**', '**/components/**'],
        routesFolder: [
          {
            path: '',
            src: resolve(resolvedSharedDir, 'pages'),
          },
        ],
      }),
      vue(),
      tailwindcss(),
    ],
    resolve: {
      alias: [
        { find: '@', replacement: resolve(projectDir, './src') },
        { find: '@shared', replacement: resolvedSharedDir },
        { find: /^@shared\/(.*)$/, replacement: resolve(resolvedSharedDir, '$1') },
      ],
    },
    test: {
      environment: 'happy-dom',
      globals: true,
      include: ['tests/**/*.spec.ts', 'tests/**/*.test.ts', 'src/**/*.spec.ts', 'src/**/*.test.ts'],
      setupFiles: ['./tests/setup.ts'],
    },
  }
}

export type { UserConfig, UserConfigExport }
