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
* Get the current platform
* @returns The platform name ('linux', 'windows', 'macos', etc.)
*/
export const getPlatform = (): Platform => (
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

/**
* Check if the current platform is desktop
* @returns true if running on desktop platforms
*/
export const isDesktop = (): boolean => true
