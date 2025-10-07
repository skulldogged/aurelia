import { platform } from '@tauri-apps/plugin-os'

let cachedPlatform: null | string = null

/**
 * Get the current platform
 * @returns The platform name ('linux', 'windows', 'macos', etc.)
 */
export const getPlatform = async (): Promise<string> => {
  if (cachedPlatform === null) {
    cachedPlatform = await platform()
  }
  return cachedPlatform
}

/**
 * Check if the current platform is Linux
 * @returns True if running on Linux
 */
export const isLinux = async (): Promise<boolean> => {
  const p = await getPlatform()
  return p === 'linux'
}

/**
 * Check if the current platform is macOS
 * @returns True if running on macOS
 */
export const isMacOS = async (): Promise<boolean> => {
  const p = await getPlatform()
  return p === 'macos'
}

/**
 * Check if the current platform is Windows
 * @returns True if running on Windows
 */
export const isWindows = async (): Promise<boolean> => {
  const p = await getPlatform()
  return p === 'windows'
}
