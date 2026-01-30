// Aurelia Shared Package - Platform-agnostic frontend code
// This package contains all shared Vue components, composables, and stores
// that work on both desktop (Tauri) and web (Browser) platforms

// Platform detection and API injection
let apiClient: any = null
let platform: 'desktop' | 'web' = 'web'

export function setApiClient(client: any, platformType: 'desktop' | 'web') {
  apiClient = client
  platform = platformType
}

export function getApiClient() {
  if (!apiClient) {
    throw new Error('API client not set. Call setApiClient() before using shared components.')
  }
  return apiClient
}

export function getPlatform() {
  return platform
}

export function isDesktop() {
  return platform === 'desktop'
}

export function isWeb() {
  return platform === 'web'
}

// Re-export types
export * from './lib/api/types'
export type { ApiClient, Result } from './lib/api/types'

// API clients
export { httpClient, WebSocketClient } from './lib/api/httpClient'

// Stores
export { useAuthStore } from './stores/auth'
export { useLibraryStore } from './stores/library'
export { usePlayerStore } from './stores/player'
export { usePlaylistStore } from './stores/playlists'
export { useHomeStore } from './stores/home'
export { useThemeStore } from './stores/theme'
export { useAccentColorStore } from './stores/accentColor'
export { useLastFmStore } from './stores/lastfm'
export { useListenBrainzStore } from './stores/listenbrainz'

// Composables
export { useAuth } from './composables/useAuth'
export { useLibrary } from './composables/useLibrary'
export { useNavigation } from './composables/useNavigation'
export { usePlayerControls } from './composables/usePlayerControls'
export { usePlayerSession } from './composables/usePlayerSession'
export { useSongInteractions } from './composables/useSongInteractions'
export { useTopBar } from './composables/useTopBar'
export { useWebAudioPlayer } from './composables/useWebAudioPlayer'

// Components
export { default as MainLayout } from './components/layout/MainLayout.vue'
export { default as MusicPlayer } from './components/player/MusicPlayer.vue'
export { default as FullscreenPlayer } from './components/player/FullscreenPlayer.vue'
export { default as Queue } from './components/player/Queue.vue'
export { default as Equalizer } from './components/player/Equalizer.vue'
export { default as LyricsSidebar } from './components/player/LyricsSidebar.vue'
export { default as GlobalSearch } from './components/shared/GlobalSearch.vue'
export { Toaster } from './components/ui/sonner'
export { default as Login } from './pages/login.vue'
