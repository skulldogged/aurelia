// Aurelia Shared API - Unified API client
//
// This module provides the unified API client that works on both
// desktop (via Tauri IPC) and web (via HTTP).

// Re-export unified API client
export { apiClient } from '../api/apiClient'

// Re-export generated types
export type {
  Album,
  AppError,
  Artist,
  Credentials,
  HomeViewData,
  LibraryData,
  Playlist,
  Song,
} from '../generated'

// Platform detection utilities
export { isTauri } from '../lib/platform'

// Legacy Tauri client factory (used by desktop app during transition)
export { createTauriClient } from './tauriClient'
