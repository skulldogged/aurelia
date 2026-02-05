import path from 'path'

import vue from '@vitejs/plugin-vue'
import { defineConfig } from 'vitest/config'

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  test: {
    environment: 'happy-dom',
    globals: true,
    include: [
      'tests/**/*.spec.ts',
      'tests/**/*.test.ts',
      'src/**/*.spec.ts',
      'src/**/*.test.ts',
    ],
    setupFiles: ['./tests/setup.ts'],
  },
})
