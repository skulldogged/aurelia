export enum Platform {
  Dragonfly = 'dragonfly',
  FreeBSD   = 'freebsd',
  Linux     = 'linux',
  MacOS     = 'macos',
  NetBSD    = 'netbsd',
  OpenBSD   = 'openbsd',
  Solaris   = 'solaris',
  Unknown   = 'unknown',
  Windows   = 'windows',
}

/**
 * Check if the current environment is Tauri
 */
export const isTauri = (): boolean => (
  typeof window !== 'undefined' &&
  (
    '__TAURI_INTERNALS__' in window ||
    (window as Window & { __TAURI__?: unknown }).__TAURI__ !== undefined
  )
)

// Cache for platform value to avoid repeated async calls
let cachedPlatform: null | Platform = null

/**
* Get the current platform
* @returns The platform name ('linux', 'windows', 'macos', etc.)
*/
export const getPlatform = (): Platform => {
  // Return cached value if available
  if (cachedPlatform !== null) {
    return cachedPlatform
  }

  // Web fallback - detect from user agent
  if (!isTauri()) {
    if (typeof navigator !== 'undefined') {
      const ua = navigator.userAgent.toLowerCase()
      if (ua.includes('mac os x')) return Platform.MacOS
      if (ua.includes('windows')) return Platform.Windows
      if (ua.includes('linux')) return Platform.Linux
    }
    return Platform.Unknown
  }

  // On Tauri, return unknown initially - use initializePlatform to set correctly
  return Platform.Unknown
}

/**
 * Initialize platform detection for Tauri apps.
 * Call this once during app startup on desktop.
 */
export const initializePlatform = async (): Promise<Platform> => {
  if (!isTauri()) {
    cachedPlatform = getPlatform()
    return cachedPlatform
  }

  try {
    const { platform } = await import('@tauri-apps/plugin-os')
    const platformMap: Record<string, Platform> = {
      dragonfly: Platform.Dragonfly,
      freebsd:   Platform.FreeBSD,
      linux:     Platform.Linux,
      macos:     Platform.MacOS,
      netbsd:    Platform.NetBSD,
      openbsd:   Platform.OpenBSD,
      solaris:   Platform.Solaris,
      windows:   Platform.Windows,
    }
    cachedPlatform = platformMap[platform()] ?? Platform.Unknown
    return cachedPlatform
  } catch {
    cachedPlatform = Platform.Unknown
    return cachedPlatform
  }
}

/**
* Check if the current platform is desktop
* @returns true if running on desktop platforms
*/
export const isDesktop = (): boolean => isTauri()
