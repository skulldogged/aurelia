import { ref, watch, onMounted, onUnmounted, readonly } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import type { Artist, Album } from '@/bindings'

export const useNavigation = () => {
  const router = useRouter()
  const route = useRoute()

  // Navigation state
  const currentView = ref('home')
  const canGoBack = ref(false)
  const canGoForward = ref(false)

  // Watch route changes to update currentView
  watch(() => route.name, newName => {
    if (newName) {
      currentView.value = newName as string
    }
  }, { immediate: true })

  // Update navigation state
  const updateNavState = () => {
    canGoBack.value = window.history.state.position > 0
    canGoForward.value = window.history.state.position < window.history.length - 1
  }

  // Set up history event listener
  onMounted(() => {
    updateNavState()
    router.afterEach(() => {
      updateNavState()
    })
    window.addEventListener('popstate', updateNavState)
  })

  onUnmounted(() => {
    window.removeEventListener('popstate', updateNavState)
  })

  // Navigation actions
  const navigateBack = () => {
    router.back()
  }

  const navigateForward = () => {
    router.forward()
  }

  const handleNavigation = (view: string) => {
    const routeMap: Record<string, string> = {
      'home':    '/',
      'library': '/songs',
      'artists': '/artists',
      'albums':  '/albums',
    }

    const routePath = routeMap[view]
    if (routePath) {
      router.push(routePath)
    }
  }

  const navigateToArtist = (artist: Artist) => {
    router.push(`/songs/artist/${artist.id}`)
  }

  const navigateToAlbum = (album: Album) => {
    router.push(`/songs/album/${encodeURIComponent(album.name)}`)
  }

  return {
    currentView:  readonly(currentView),
    canGoBack:    readonly(canGoBack),
    canGoForward: readonly(canGoForward),
    navigateBack,
    navigateForward,
    handleNavigation,
    navigateToArtist,
    navigateToAlbum,
  }
}
