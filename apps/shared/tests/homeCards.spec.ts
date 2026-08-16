import { cleanup, render } from '@testing-library/vue'
import { afterEach, describe, expect, it } from 'vitest'

import AlbumCard from '../src/components/shared/AlbumCard.vue'
import CardPlayOverlay from '../src/components/shared/CardPlayOverlay.vue'
import SongCard from '../src/components/shared/SongCard.vue'
import type { Album, Song } from '../src/lib/api/types'

const album = {
  artist:    'Marlowe Vale',
  artistId:  'artist-1',
  id:        'album-1',
  name:      'Glass Harbor',
  songCount: 11,
} as Album

const song = {
  albumId:    'album-1',
  artistIds:  ['artist-1'],
  artists:    ['Marlowe Vale'],
  id:         'song-1',
  name:       'Second Tide',
} as Song

const stubs = {
  ImageLoader:      { template: '<div data-testid="art" />' },
  ImagePlaceholder: true,
  RouterLink:       { template: '<a><slot /></a>' },
}

describe('home carousel cards', () => {
  afterEach(() => {
    cleanup()
  })

  it('uses the same non-blurred play button on albums and songs', () => {
    const albumView = render(AlbumCard, {
      global: { stubs },
      props:  { album, serverUrl: 'http://x', token: 't' },
    })
    const albumButton = albumView.getByRole('button')
    cleanup()

    const songView = render(SongCard, {
      global: { stubs },
      props:  { serverUrl: 'http://x', song, token: 't' },
    })
    const songButton = songView.getByRole('button')

    expect(albumButton.className).toBe(songButton.className)
    expect(albumButton.className).toContain('bg-white/40')
    expect(albumButton.className).not.toContain('backdrop-blur')
  })

  it('shows a song-count disc on albums only', () => {
    const albumView = render(AlbumCard, {
      global: { stubs },
      props:  { album, serverUrl: 'http://x', token: 't' },
    })
    expect(albumView.getByText('11')).toBeTruthy()
    cleanup()

    const songView = render(SongCard, {
      global: { stubs },
      props:  { serverUrl: 'http://x', song, token: 't' },
    })
    expect(songView.queryByText('11')).toBeNull()
  })

  it('does not lift album art on hover', () => {
    const { container } = render(AlbumCard, {
      global: { stubs },
      props:  { album, serverUrl: 'http://x', token: 't' },
    })

    expect(container.innerHTML).not.toContain('translateY')
    expect(container.innerHTML).not.toContain('album-card-image')
  })

  it('does not clip song art in a nested overflow-hidden box', () => {
    const { container } = render(SongCard, {
      global: { stubs },
      props:  { serverUrl: 'http://x', song, token: 't' },
    })

    expect(container.querySelector('.overflow-hidden')).toBeNull()
  })

  it('keeps the overlay from stealing clicks until hover', () => {
    const { container } = render(CardPlayOverlay)

    const overlay = container.firstElementChild as HTMLElement
    expect(overlay.className).toContain('pointer-events-none')
    expect(overlay.className).toContain('group-hover:pointer-events-auto')
  })
})
