// Aurelia Shared API - Unified API client
//
// This module provides the unified API client that works on both
// desktop (via Tauri IPC) and web (via HTTP).

// Re-export unified API client and setup function
export { apiClient } from '../../api/apiClient'
export { setApiClient } from '../..'

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
} from '../../generated'

// Platform detection utilities
export { isTauri } from '../platform'
