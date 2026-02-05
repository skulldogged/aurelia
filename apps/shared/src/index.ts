// Aurelia Shared Package - Platform-agnostic frontend code
// This package contains all shared Vue components, composables, and stores
// that work on both desktop (Tauri) and web (Browser) platforms

// Platform detection and API injection
import type { ApiClient } from './lib/api/types'

let _internalApiClient: ApiClient | null = null
let platform: 'desktop' | 'web' = 'web'

export const getApiClient = (): ApiClient => {
  if (!_internalApiClient) {
    throw new Error('API client not set. Call setApiClient() before using shared components.')
  }
  return _internalApiClient
}

export const getPlatform = (): 'desktop' | 'web' => platform

export const isDesktop = (): boolean => platform === 'desktop'

export const isWeb = (): boolean => platform === 'web'

export const setApiClient = (client: ApiClient, platformType: 'desktop' | 'web'): void => {
  _internalApiClient = client
  platform = platformType
}

// Unified API client (works on both desktop and web)
export { apiClient } from './api/apiClient'
export * from './effect'

// Audio
export * from './audio'

// Components
export { default as MainLayout } from './components/layout/MainLayout.vue'

export { default as Equalizer } from './components/player/Equalizer.vue'
export { default as FullscreenPlayer } from './components/player/FullscreenPlayer.vue'
export { default as LyricsSidebar } from './components/player/LyricsSidebar.vue'
export { default as MusicPlayer } from './components/player/MusicPlayer.vue'
export { default as Queue } from './components/player/Queue.vue'
export { default as GlobalSearch } from './components/shared/GlobalSearch.vue'
export { Toaster } from './components/ui/sonner'
// Composables
export { useAuth } from './composables/useAuth'
export { useLibrary } from './composables/useLibrary'

export { useNavigation } from './composables/useNavigation'

export { usePlayerControls } from './composables/usePlayerControls'
export { usePlayerSession } from './composables/usePlayerSession'
export { useSongInteractions } from './composables/useSongInteractions'
export { useTopBar } from './composables/useTopBar'
export { useVisualizerData } from './composables/useVisualizerData'
// Re-export generated types
export * from './generated'
// WebSocket client for web platform
// Types are exported from lib/api/types
export { default as Login } from './pages/login.vue'

export { useAccentColorStore } from './stores/accentColor'
// Stores
export { useAuthStore } from './stores/auth'
export { useHomeStore } from './stores/home'
export { useLastFmStore } from './stores/lastfm'
export { useLibraryStore } from './stores/library'
export { useListenBrainzStore } from './stores/listenbrainz'
export { usePlayerStore } from './stores/player'
export { usePlaylistStore } from './stores/playlists'
export { useThemeStore } from './stores/theme'
