import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import { usePlayerControls } from '../src/composables/usePlayerControls'
import { usePlayerStore } from '../src/stores/player'

describe('usePlayerControls', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('falls back to store when no player ref exists', () => {
    const controls = usePlayerControls()
    const store = usePlayerStore()
    const toggleSpy = vi.spyOn(store, 'togglePlay')

    controls.handleTogglePlayPause()

    expect(toggleSpy).toHaveBeenCalled()
  })

  it('uses player ref for actions when available', () => {
    const controls = usePlayerControls()
    const togglePlayPause = vi.fn()
    const onSeek = vi.fn()

    controls.musicPlayerRef.value = {
      nextSong: vi.fn(),
      onSeek,
      previousSong: vi.fn(),
      togglePlayPause,
      toggleRepeat: vi.fn(),
      toggleShuffle: vi.fn(),
    }

    controls.handleTogglePlayPause()
    controls.handleSeek(25)

    expect(togglePlayPause).toHaveBeenCalled()
    expect(onSeek).toHaveBeenCalledWith([25])
  })

  it('closes other panels when toggling queue', () => {
    const controls = usePlayerControls()

    controls.toggleEqualizer()
    controls.toggleLyrics()
    controls.toggleQueue()

    expect(controls.isQueueOpen.value).toBe(true)
    expect(controls.isEqualizerOpen.value).toBe(false)
    expect(controls.isLyricsOpen.value).toBe(false)
  })
})
