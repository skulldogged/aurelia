import userEvent from '@testing-library/user-event'
import { render } from '@testing-library/vue'
import { describe, expect, it } from 'vitest'

import AlphabetNav from '../src/components/shared/AlphabetNav.vue'

describe('AlphabetNav', () => {
  it('emits when clicking available letters', async () => {
    const user = userEvent.setup()
    const { emitted, getByText } = render(AlphabetNav, {
      props: {
        activeLetter:     'A',
        availableLetters: new Set(['A', 'B']),
      },
    })

    const clearButton = getByText('Clear')
    await user.click(clearButton)

    const letterB = getByText('B')
    await user.click(letterB)

    expect(emitted().select).toEqual([[null], ['B']])
  })

  it('disables unavailable letters', async () => {
    const user = userEvent.setup()
    const { emitted, getByText } = render(AlphabetNav, {
      props: {
        availableLetters: new Set(['A']),
      },
    })

    const letterC = getByText('C')
    expect(letterC).toBeDisabled()

    await user.click(letterC)
    expect(emitted().select).toBeUndefined()
  })
})
