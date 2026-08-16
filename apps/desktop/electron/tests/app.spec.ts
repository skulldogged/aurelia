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

vi.mock('@shared', () => ({
  getApiClient: () => ({
    getSyncState: () => Promise.resolve({
      data: {
        albumCount:   0,
        artistCount:  0,
        lastSyncTime: null,
        songCount:    0,
      },
      status: 'ok',
    }),
  }),
  isDesktop: () => true,
}))

vi.mock('@shared/components/layout/MainLayout.vue', () => ({
  default: stubs.MainLayoutStub,
}))

vi.mock('@shared/components/player/Equalizer.vue', () => ({
  default: stubs.Stub,
}))

vi.mock('@shared/components/player/FullscreenPlayer.vue', () => ({
  default: stubs.Stub,
}))

vi.mock('@shared/components/player/LyricsSidebar.vue', () => ({
  default: stubs.Stub,
}))

vi.mock('@shared/components/player/MusicPlayer.vue', () => ({
  default: stubs.Stub,
}))

vi.mock('@shared/components/player/Queue.vue', () => ({
  default: stubs.Stub,
}))

vi.mock('@shared/components/shared/GlobalSearch.vue', () => ({
  default: stubs.Stub,
}))

vi.mock('@shared/components/ui/Button.vue', () => ({
  default: stubs.Stub,
}))

vi.mock('@shared/components/ui/dialog', () => ({
  Dialog:            stubs.Stub,
  DialogContent:     stubs.Stub,
  DialogDescription: stubs.Stub,
  DialogFooter:      stubs.Stub,
  DialogHeader:      stubs.Stub,
  DialogTitle:       stubs.Stub,
}))

vi.mock('@shared/components/ui/sonner', () => ({
  Toaster: stubs.Stub,
}))

vi.mock('@shared/pages/login.vue', () => ({
  default: stubs.LoginStub,
}))

vi.mock('@shared/lib/auth-interceptor', () => ({
  setAuthLogout: vi.fn(),
}))

vi.mock('@shared/composables/useAuth', () => ({
  useAuth: () => ({
    authStatus:  ref('loggedOut'),
    clearError:  vi.fn(),
    credentials: ref(null),
    error:       ref(null),
    login:       vi.fn(),
    logout:      vi.fn(),
  }),
}))

vi.mock('@shared/composables/useDiscordPresence', () => ({
  useDiscordPresence: () => ({}),
}))

vi.mock('@shared/composables/useLastFm', () => ({
  useLastFm: () => ({}),
}))

vi.mock('@shared/composables/useListenBrainz', () => ({
  useListenBrainz: () => ({}),
}))

vi.mock('@shared/composables/useNavigation', () => ({
  useNavigation: () => ({
    canGoBack:        ref(false),
    canGoForward:     ref(false),
    currentView:      ref('home'),
    handleNavigation: vi.fn(),
    navigateBack:     vi.fn(),
    navigateForward:  vi.fn(),
    navigateToAlbum:  vi.fn(),
    navigateToArtist: vi.fn(),
  }),
}))

vi.mock('@shared/composables/usePlayerControls', () => ({
  usePlayerControls: () => ({
    handleNextSong:         vi.fn(),
    handlePreviousSong:     vi.fn(),
    handleSeek:             vi.fn(),
    handleTogglePlayPause:  vi.fn(),
    handleToggleRepeat:     vi.fn(),
    handleToggleShuffle:    vi.fn(),
    isEqualizerOpen:        ref(false),
    isFullScreenPlayerOpen: ref(false),
    isLyricsOpen:           ref(false),
    isQueueOpen:            ref(false),
    musicPlayerRef:         ref(null),
    playerStore:            {
      canGoNext:       vi.fn(() => false),
      canGoPrevious:   vi.fn(() => false),
      currentIndex:    -1,
      currentSong:     null,
      currentTime:     0,
      duration:        0,
      isMuted:         false,
      isPlaying:       false,
      isShuffled:      false,
      playlist:        [],
      progress:        0,
      repeatMode:      'none',
      setCurrentIndex: vi.fn(),
      setCurrentSong:  vi.fn(),
      setHasLyrics:    vi.fn(),
      setPlaylist:     vi.fn(),
      setVolume:       vi.fn(),
      toggleMute:      vi.fn(),
      volume:          1,
    },
    toggleEqualizer:        vi.fn(),
    toggleFullScreenPlayer: vi.fn(),
    toggleLyrics:           vi.fn(),
    toggleQueue:            vi.fn(),
  }),
}))

vi.mock('@shared/composables/usePlayerSession', () => ({
  usePlayerSession: () => ({}),
}))

vi.mock('@shared/composables/useSongInteractions', () => ({
  useSongInteractions: () => ({
    playInstantMix:         vi.fn(),
    playSong:               vi.fn(),
    playSongs:              vi.fn(),
    removeSongFromPlaylist: vi.fn(),
    toggleFavorite:         vi.fn(),
    updatePlaylist:         vi.fn(),
  }),
}))

vi.mock('@shared/composables/useSystemTray', () => ({
  useSystemTray: () => ({}),
}))

vi.mock('@shared/composables/useTopBar', () => ({
  useTopBar: () => ({
    topBarContent: ref(null),
  }),
}))

vi.mock('@shared/composables/useVisualizerData', () => ({
  useVisualizerData: () => ({
    frequencyData:  ref([]),
    setEnabled:     vi.fn(),
    timeDomainData: ref([]),
  }),
}))

vi.mock('@shared/stores', () => ({
  useHomeStore: () => ({
    refreshHomeData: vi.fn(),
    resetHomeData:   vi.fn(),
  }),
  useLibraryStore: () => ({
    clearCache:  vi.fn(),
    clearData:   vi.fn(),
    isLoaded:    true,
    loadLibrary: vi.fn(),
    syncLibrary: vi.fn(),
  }),
}))

vi.mock('@vueuse/core', () => ({
  useColorMode:    () => ({}),
  useLocalStorage: (_key: string, defaultValue: unknown) => ref(defaultValue),
  useMagicKeys:    () => ({
    'Ctrl+K': ref(false),
  }),
}))

describe('desktop App', () => {
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
