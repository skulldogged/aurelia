import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it } from 'vitest'

import { usePlayerStore } from '../src/stores/player'

const mockSong = (id: string): any => ({
  id,
  name: `Song ${id}`,
})

describe('player store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('clamps volume to [0, 1]', () => {
    const store = usePlayerStore()

    store.setVolume(2)
    expect(store.volume).toBe(1)

    store.setVolume(-1)
    expect(store.volume).toBe(0)
  })

  it('toggles mute and restores volume', () => {
    const store = usePlayerStore()
    store.setVolume(0.8)

    store.toggleMute()
    expect(store.isMuted).toBe(true)
    expect(store.volume).toBe(0)

    store.toggleMute()
    expect(store.isMuted).toBe(false)
    expect(store.volume).toBe(0.8)
  })

  it('cycles repeat mode', () => {
    const store = usePlayerStore()

    store.setRepeatMode('none')
    store.cycleRepeatMode()
    expect(store.repeatMode).toBe('all')

    store.cycleRepeatMode()
    expect(store.repeatMode).toBe('one')

    store.cycleRepeatMode()
    expect(store.repeatMode).toBe('none')
  })

  it('uses stable shuffled traversal and restores linear navigation when disabled', () => {
    const store = usePlayerStore()
    const songs = ['1', '2', '3', '4'].map(mockSong)

    store.setPlaylist(songs)
    store.setCurrentIndex(0)

    store.toggleShuffle()
    expect(store.isShuffled).toBe(true)

    const visited = [store.currentIndex]
    while (store.canGoNext()) {
      store.nextSong()
      visited.push(store.currentIndex)
    }
    expect(new Set(visited).size).toBe(songs.length)

    store.setCurrentIndex(2)
    store.toggleShuffle()
    expect(store.isShuffled).toBe(false)
    expect(store.getPreviousSongIndex()).toBe(1)
    expect(store.getNextSongIndex()).toBe(3)
  })
})
