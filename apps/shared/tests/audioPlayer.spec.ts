import { afterEach, describe, expect, it } from 'vitest'

import {
  createAudioPlayer,
  resetAudioPlayer,
  RustAudioPlayerImpl,
  WebAudioPlayerImpl,
} from '../src/audio'

const originalAureliaDesktop = (window as Window & { aureliaDesktop?: unknown }).aureliaDesktop

afterEach(() => {
  resetAudioPlayer()
  if (originalAureliaDesktop === undefined) {
    delete (window as Window & { aureliaDesktop?: unknown }).aureliaDesktop
  } else {
    (window as Window & { aureliaDesktop?: unknown }).aureliaDesktop = originalAureliaDesktop
  }
})

describe('audio player factory', () => {
  it('uses the rust backend player in electron', () => {
    (window as Window & { aureliaDesktop?: unknown }).aureliaDesktop = {
      appVersion: '0.1.0',
      platform:   'linux',
    }

    expect(createAudioPlayer()).toBeInstanceOf(RustAudioPlayerImpl)
  })

  it('uses the web audio player in the browser', () => {
    delete (window as Window & { aureliaDesktop?: unknown }).aureliaDesktop

    expect(createAudioPlayer()).toBeInstanceOf(WebAudioPlayerImpl)
  })
})
