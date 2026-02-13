import type { UserConfig } from 'vitest/config'

import { resolve } from 'path'
import { defineConfig, mergeConfig } from 'vitest/config'

import sharedConfig from './apps/shared/vitest.config.ts'

const makeConfig = (projectRoot: string, name: string): UserConfig => mergeConfig(sharedConfig, {
  resolve: {
    alias: {
      '@':        resolve(projectRoot, './src'),
      '@shared':  resolve(__dirname, './apps/shared/src'),
      '@shared/': resolve(__dirname, './apps/shared/src/'),
    },
  },
  root: projectRoot,
  test: { name },
})

const sharedProjectConfig = mergeConfig(sharedConfig, { test: { name: 'shared' } })
const webProjectConfig = makeConfig('./apps/web/frontend', 'web')
const desktopProjectConfig = makeConfig('./apps/desktop/tauri', 'desktop')

export default defineConfig({
  test: {
    projects: [sharedProjectConfig, webProjectConfig, desktopProjectConfig],
  },
})
