import { readonly, ref, type Ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import type { Album, Artist } from '@/bindings'

const currentView = ref('home')
const canGoBack = ref(false)
const canGoForward = ref(false)

const ROUTE_MAP: Record<string, string> = {
  'albums':    '/albums',
  'artists':   '/artists',
  'home':      '/',
  'library':   '/songs',
  'playlists': '/playlists',
  'settings':  '/settings',
  'songs':     '/songs',
}

// Pure functions for navigation logic
const getRoutePath = (view: string): string | undefined => ROUTE_MAP[view]

const createArtistPath = (artist: Artist): string => `/artists/${artist.id}`

const createAlbumPath = (album: Album): string => `/albums/${album.id}`

const updateNavState = (): void => {
  canGoBack.value = window.history.state.position > 0
  canGoForward.value = window.history.state.position < window.history.length - 1
}

// Navigation functions
const navigateToPath = (router: ReturnType<typeof useRouter>, path: string): void => {
  router.push(path)
}

const navigateBack = (router: ReturnType<typeof useRouter>): void => {
  router.back()
}

const navigateForward = (router: ReturnType<typeof useRouter>): void => {
  router.forward()
}

// Composed navigation functions
const handleNavigation = (router: ReturnType<typeof useRouter>) => (view: string): void => {
  const path = getRoutePath(view)
  if (path) navigateToPath(router, path)
}

const navigateToArtist = (router: ReturnType<typeof useRouter>) => (artist: Artist): void => {
  navigateToPath(router, createArtistPath(artist))
}

const navigateToAlbum = (router: ReturnType<typeof useRouter>) => (album: Album): void => {
  navigateToPath(router, createAlbumPath(album))
}

export interface Navigation {
  canGoBack:        Readonly<Ref<boolean>>
  canGoForward:     Readonly<Ref<boolean>>
  currentView:      Readonly<Ref<string>>
  handleNavigation: (view: string) => void
  navigateBack:     () => void
  navigateForward:  () => void
  navigateToAlbum:  (album: Album) => void
  navigateToArtist: (artist: Artist) => void
}

export const useNavigation = (): Navigation => {
  const router = useRouter()
  const route = useRoute()

  // Set up route watcher to keep UI nav state in sync with current path
  watch(
    () => route.path,
    newPath => {
      if (!newPath) return
      if (newPath === '/') currentView.value = 'home'
      else if (newPath.startsWith('/songs')) currentView.value = 'songs'
      else if (newPath.startsWith('/artists')) currentView.value = 'artists'
      else if (newPath.startsWith('/albums')) currentView.value = 'albums'
      else if (newPath.startsWith('/playlists')) currentView.value = 'playlists'
      else if (newPath.startsWith('/settings')) currentView.value = 'settings'
      else currentView.value = 'home'
    },

  )

  // Set up navigation state management
  updateNavState()
  router.afterEach(() => updateNavState())

  // Add event listener for browser navigation
  if (typeof window !== 'undefined')
    window.addEventListener('popstate', updateNavState)

  return {
    canGoBack:        readonly(canGoBack),
    canGoForward:     readonly(canGoForward),
    currentView:      readonly(currentView),
    handleNavigation: handleNavigation(router),
    navigateBack:     () => navigateBack(router),
    navigateForward:  () => navigateForward(router),
    navigateToAlbum:  navigateToAlbum(router),
    navigateToArtist: navigateToArtist(router),
  }
}
