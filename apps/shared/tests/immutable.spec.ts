import { describe, expect, it } from 'vitest'

import { getIn, insertAt, removeAt, setIn, updateIn } from '../src/lib/immutable'

describe('immutable helpers', () => {
  it('sets and gets nested values', () => {
    const state = { a: { b: [1, 2, 3] } }
    const updated = setIn(state, ['a', 'b', 1], 42)

    expect(state.a.b[1]).toBe(2)
    expect(getIn(updated, ['a', 'b', 1])).toBe(42)
  })

  it('updates nested values', () => {
    const state = { a: { b: 5 } }
    const updated = updateIn(state, ['a', 'b'], value => (value as number) + 1)

    expect(state.a.b).toBe(5)
    expect(updated.a.b).toBe(6)
  })

  it('removes and inserts items immutably', () => {
    const items = [1, 2, 3]
    expect(removeAt(items, 1)).toEqual([1, 3])
    expect(insertAt(items, 1, 9)).toEqual([1, 9, 2, 3])
    expect(items).toEqual([1, 2, 3])
  })
})
