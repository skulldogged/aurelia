import { platform } from '@tauri-apps/plugin-os'

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
const isTauri = (): boolean => (
  typeof window !== 'undefined' && 
  (
    '__TAURI_INTERNALS__' in window || 
    (window as any).__TAURI__ !== undefined
  )
)

/**
* Get the current platform
* @returns The platform name ('linux', 'windows', 'macos', etc.)
*/
export const getPlatform = (): Platform => {
  if (!isTauri()) {
    if (typeof navigator !== 'undefined') {
      const ua = navigator.userAgent.toLowerCase()
      if (ua.includes('mac os x')) return Platform.MacOS
      if (ua.includes('windows')) return Platform.Windows
      if (ua.includes('linux')) return Platform.Linux
    }
    return Platform.Unknown
  }

  try {
    return (
      ({
        dragonfly: Platform.Dragonfly,
        freebsd:   Platform.FreeBSD,
        linux:     Platform.Linux,
        macos:     Platform.MacOS,
        netbsd:    Platform.NetBSD,
        openbsd:   Platform.OpenBSD,
        solaris:   Platform.Solaris,
        windows:   Platform.Windows,
      } as Record<string, Platform>)[platform()]
      ?? Platform.Unknown
    )
  } catch (e) {
    return Platform.Unknown
  }
}

/**
* Check if the current platform is desktop
* @returns true if running on desktop platforms
*/
export const isDesktop = (): boolean => isTauri()
