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

/**
 * Check if the current platform is mobile
 * @returns true if running on Android or iOS
 */
export const isMobile = (): boolean => {
  const current = getPlatform()
  return current === Platform.Android || current === Platform.IOS
}

/**
* Check if the current platform is desktop
* @returns true if running on desktop platforms
*/
export const isDesktop = (): boolean => !isMobile()

/**
  * Check if the current orientation is portrait on mobile devices
  * @returns true if on mobile and in portrait orientation
  */
export const isMobilePortrait = (): boolean => {
  if (!isMobile() || typeof window === 'undefined') return false
  return window.innerHeight > window.innerWidth
}
