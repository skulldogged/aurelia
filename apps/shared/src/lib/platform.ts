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
 * Check if the current environment is the Electron desktop shell.
 */
export const isElectron = (): boolean => (
  typeof window !== 'undefined' &&
  Boolean((window as Window & { aureliaDesktop?: unknown }).aureliaDesktop)
)

// Cache for platform value to avoid repeated async calls
let cachedPlatform: null | Platform = null

const platformFromUserAgent = (): Platform => {
  if (typeof navigator === 'undefined') return Platform.Unknown
  const ua = navigator.userAgent.toLowerCase()
  if (ua.includes('mac os x')) return Platform.MacOS
  if (ua.includes('windows')) return Platform.Windows
  if (ua.includes('linux')) return Platform.Linux
  return Platform.Unknown
}

/**
* Get the current platform
* @returns The platform name ('linux', 'windows', 'macos', etc.)
*/
export const getPlatform = (): Platform => {
  if (cachedPlatform !== null) {
    return cachedPlatform
  }

  return platformFromUserAgent()
}

/**
 * Initialize platform detection for desktop apps.
 * Call this once during app startup on desktop.
 */
export const initializePlatform = async (): Promise<Platform> => {
  if (isElectron()) {
    const electronPlatform = (window as Window & {
      aureliaDesktop?: { platform?: string }
    }).aureliaDesktop?.platform
    const electronMap: Record<string, Platform> = {
      darwin:  Platform.MacOS,
      freebsd: Platform.FreeBSD,
      linux:   Platform.Linux,
      openbsd: Platform.OpenBSD,
      sunos:   Platform.Solaris,
      win32:   Platform.Windows,
    }
    cachedPlatform = (electronPlatform && electronMap[electronPlatform]) || platformFromUserAgent()
    return cachedPlatform
  }

  cachedPlatform = platformFromUserAgent()
  return cachedPlatform
}

/**
* Check if the current platform is desktop
* @returns true if running in the Electron desktop shell
*/
export const isDesktop = (): boolean => isElectron()
