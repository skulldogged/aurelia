import { readonly, ref, type Ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import type { Album, Artist } from '@/bindings'

const currentView = ref('home')
const canGoBack = ref(false)
const canGoForward = ref(false)

const ROUTE_MAP: Record<string, string> = {
  'albums':  '/albums',
  'artists': '/artists',
  'home':    '/',
  'library': '/songs',
}

// Pure functions for navigation logic
const getRoutePath = (view: string): string | undefined => ROUTE_MAP[view]

const createArtistPath = (artist: Artist): string => `/songs/artist/${artist.id}`

const createAlbumPath = (album: Album): string => `/songs/album/${album.id}`

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

  // Set up route watcher
  watch(() => route.name, newName => {
    if (newName)
      currentView.value = newName as string
  }, { immediate: true })

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
