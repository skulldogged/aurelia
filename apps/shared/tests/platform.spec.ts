import { afterEach, describe, expect, it } from 'vitest'

import { isDesktop, isElectron } from '../src/lib/platform'

const originalAureliaDesktop = (window as Window & { aureliaDesktop?: unknown }).aureliaDesktop

afterEach(() => {
  if (originalAureliaDesktop === undefined) {
    delete (window as Window & { aureliaDesktop?: unknown }).aureliaDesktop
  } else {
    (window as Window & { aureliaDesktop?: unknown }).aureliaDesktop = originalAureliaDesktop
  }
})

describe('platform detection', () => {
  it('does not treat a regular browser as desktop', () => {
    delete (window as Window & { aureliaDesktop?: unknown }).aureliaDesktop
    expect(isElectron()).toBe(false)
    expect(isDesktop()).toBe(false)
  })

  it('treats the electron preload bridge as desktop', () => {
    (window as Window & { aureliaDesktop?: unknown }).aureliaDesktop = {
      appVersion: '0.1.0',
      platform:   'linux',
    }
    expect(isElectron()).toBe(true)
    expect(isDesktop()).toBe(true)
  })

  it('does not treat an empty window as electron', () => {
    delete (window as Window & { aureliaDesktop?: unknown }).aureliaDesktop
    expect(isElectron()).toBe(false)
  })
})
