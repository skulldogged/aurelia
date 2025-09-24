import { readonly, ref, type Ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import type { Album, Artist } from '@/bindings'

const currentView = ref('home')
const canGoBack = ref(false)
const canGoForward = ref(false)

const updateNavState = (): void => {
  canGoBack.value = window.history.state.position > 0
  canGoForward.value = window.history.state.position < window.history.length - 1
}

const navigateBack = (router: ReturnType<typeof useRouter>): void => {
  router.back()
}

const navigateForward = (router: ReturnType<typeof useRouter>): void => {
  router.forward()
}

const handleNavigation = (router: ReturnType<typeof useRouter>, view: string): void => {
  const routeMap: Record<string, string> = {
    'albums':  '/albums',
    'artists': '/artists',
    'home':    '/',
    'library': '/songs',
  }

  const routePath = routeMap[view]
  if (routePath)
    router.push(routePath)
}

const navigateToArtist = (router: ReturnType<typeof useRouter>, artist: Artist): void => {
  router.push(`/songs/artist/${artist.id}`)
}

const navigateToAlbum = (router: ReturnType<typeof useRouter>, album: Album): void => {
  router.push(`/songs/album/${encodeURIComponent(album.name)}`)
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
    handleNavigation: view => handleNavigation(router, view),
    navigateBack:     () => navigateBack(router),
    navigateForward:  () => navigateForward(router),
    navigateToAlbum:  album => navigateToAlbum(router, album),
    navigateToArtist: artist => navigateToArtist(router, artist),
  }
}
