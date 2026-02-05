import { render } from '@testing-library/vue'
import { describe, expect, it, vi } from 'vitest'
import { ref } from 'vue'

import App from '../src/App.vue'

const stubs = vi.hoisted(() => {
  const Stub = { template: '<div />' }
  const LoginStub = { template: '<div data-testid="login">Login</div>' }
  const MainLayoutStub = {
    template: '<div><slot /><slot name="queue" /><slot name="player" /><slot name="top-bar" /></div>',
  }

  return { LoginStub, MainLayoutStub, Stub }
})

vi.mock('@shared', () => {
  const { LoginStub, MainLayoutStub, Stub } = stubs
  return {
    Equalizer: Stub,
    FullscreenPlayer: Stub,
    GlobalSearch: Stub,
    LyricsSidebar: Stub,
    MainLayout: MainLayoutStub,
    MusicPlayer: Stub,
    Queue: Stub,
    Login: LoginStub,
    Toaster: Stub,
    getSyncStateEffect: vi.fn(),
    runAureliaEffect: vi.fn(() =>
      Promise.resolve({
        lastSyncTime: null,
        songCount: 0,
        artistCount: 0,
        albumCount: 0,
      }),
    ),
    useAccentColorStore: () => ({}),
    useAuth: () => ({
      authStatus: ref('loggedOut'),
      clearError: vi.fn(),
      credentials: ref(null),
      error: ref(null),
      login: vi.fn(),
      logout: vi.fn(),
    }),
    useHomeStore: () => ({
      refreshHomeData: vi.fn(),
      resetHomeData: vi.fn(),
    }),
    useLibraryStore: () => ({
      clearCache: vi.fn(),
      clearData: vi.fn(),
      isLoaded: true,
      loadLibrary: vi.fn(),
      syncLibrary: vi.fn(),
    }),
    useNavigation: () => ({
      canGoBack: ref(false),
      canGoForward: ref(false),
      currentView: ref('home'),
      handleNavigation: vi.fn(),
      navigateBack: vi.fn(),
      navigateForward: vi.fn(),
      navigateToAlbum: vi.fn(),
      navigateToArtist: vi.fn(),
    }),
    usePlayerControls: () => ({
      handleNextSong: vi.fn(),
      handlePreviousSong: vi.fn(),
      handleSeek: vi.fn(),
      handleTogglePlayPause: vi.fn(),
      handleToggleRepeat: vi.fn(),
      handleToggleShuffle: vi.fn(),
      isEqualizerOpen: ref(false),
      isFullScreenPlayerOpen: ref(false),
      isLyricsOpen: ref(false),
      isQueueOpen: ref(false),
      musicPlayerRef: ref(null),
      playerStore: {
        currentSong: null,
        currentTime: 0,
        duration: 0,
        isMuted: false,
        isPlaying: false,
        isShuffled: false,
        playlist: [],
        progress: 0,
        repeatMode: 'none',
        currentIndex: -1,
        volume: 1,
        toggleMute: vi.fn(),
        setVolume: vi.fn(),
        setCurrentSong: vi.fn(),
        setPlaylist: vi.fn(),
        setCurrentIndex: vi.fn(),
        setHasLyrics: vi.fn(),
      },
      toggleEqualizer: vi.fn(),
      toggleFullScreenPlayer: vi.fn(),
      toggleLyrics: vi.fn(),
      toggleQueue: vi.fn(),
    }),
    usePlayerSession: () => ({}),
    usePlayerStore: () => ({}),
    useSongInteractions: () => ({
      playInstantMix: vi.fn(),
      playSong: vi.fn(),
      playSongs: vi.fn(),
      removeSongFromPlaylist: vi.fn(),
      toggleFavorite: vi.fn(),
      updatePlaylist: vi.fn(),
    }),
    useThemeStore: () => ({}),
    useTopBar: () => ({
      topBarContent: ref(null),
    }),
    useVisualizerData: () => ({
      frequencyData: ref([]),
      setEnabled: vi.fn(),
      timeDomainData: ref([]),
    }),
  }
})

vi.mock('@shared/components/ui/Button.vue', () => ({
  default: stubs.Stub,
}))

vi.mock('@shared/components/ui/dialog', () => ({
  Dialog: stubs.Stub,
  DialogContent: stubs.Stub,
  DialogDescription: stubs.Stub,
  DialogFooter: stubs.Stub,
  DialogHeader: stubs.Stub,
  DialogTitle: stubs.Stub,
}))

vi.mock('@shared/lib/auth-interceptor', () => ({
  setAuthLogout: vi.fn(),
}))

vi.mock('@shared/composables/useLastFm', () => ({
  useLastFm: () => ({}),
}))

vi.mock('@shared/composables/useListenBrainz', () => ({
  useListenBrainz: () => ({}),
}))

vi.mock('@vueuse/core', () => ({
  useColorMode: () => ({}),
  useMagicKeys: () => ({
    'Ctrl+K': ref(false),
  }),
}))

describe('web App', () => {
  it('renders login screen when logged out', () => {
    const { getByTestId } = render(App, {
      global: {
        stubs: {
          RouterView: stubs.Stub,
        },
      },
    })

    expect(getByTestId('login')).toBeTruthy()
  })
})
