import { platform } from '@tauri-apps/plugin-os'

export enum Platform {
  Android   = 'android',
  Dragonfly = 'dragonfly',
  FreeBSD   = 'freebsd',
  IOS       = 'ios',
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
  {
    android:   Platform.Android,
    dragonfly: Platform.Dragonfly,
    freebsd:   Platform.FreeBSD,
    ios:       Platform.IOS,
    linux:     Platform.Linux,
    macos:     Platform.MacOS,
    netbsd:    Platform.NetBSD,
    openbsd:   Platform.OpenBSD,
    solaris:   Platform.Solaris,
    windows:   Platform.Windows,
  }[platform()]
    ?? Platform.Unknown
)
