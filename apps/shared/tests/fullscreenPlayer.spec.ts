import { cleanup, render } from '@testing-library/vue'
import { afterEach, describe, expect, it } from 'vitest'

import FullscreenPlayerHeader from '../src/components/player/FullscreenPlayerHeader.vue'

describe('FullscreenPlayerHeader', () => {
  afterEach(() => {
    cleanup()
  })

  it('keeps close and lyrics outside the drag region', () => {
    const { getAllByRole, container } = render(FullscreenPlayerHeader, {
      props: {
        hasLyrics:  true,
        isDesktop:  true,
        showLyrics: false,
      },
    })

    const buttons = getAllByRole('button')
    const dragRegions = container.querySelectorAll('[data-drag-region]')

    expect(buttons).toHaveLength(2)
    expect(dragRegions).toHaveLength(1)
    expect(dragRegions[0]?.className).toContain('flex-1')
    expect(buttons.every(button => !dragRegions[0]?.contains(button))).toBe(true)
    expect(buttons.every(button => button.closest('[data-no-drag]'))).toBe(true)
  })
})
