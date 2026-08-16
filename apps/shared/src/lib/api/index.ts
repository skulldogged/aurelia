// Aurelia Shared API - Unified API client
//
// This module provides the unified API client for web and Electron (HTTP).

export { setApiClient } from '../..'
// Re-export unified API client and setup function
export { apiClient } from '../../api/apiClient'

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
export { isDesktop, isElectron } from '../platform'
