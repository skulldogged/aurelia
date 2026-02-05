import { describe, expect, it } from 'vitest'

import {
  chain,
  filter,
  fromNullable,
  fromPredicate,
  isNone,
  isSome,
  map,
  match,
  unwrap,
  unwrapOr,
  unwrapOrElse,
} from '../src/lib/option'

describe('option utilities', () => {
  it('creates options from nullable values', () => {
    expect(isNone(fromNullable(null))).toBe(true)
    expect(isSome(fromNullable('value'))).toBe(true)
  })

  it('creates options from predicates', () => {
    const even = fromPredicate(2, value => value % 2 === 0)
    const odd = fromPredicate(3, value => value % 2 === 0)

    expect(isSome(even)).toBe(true)
    expect(isNone(odd)).toBe(true)
  })

  it('maps and chains values', () => {
    const value = fromNullable(2)
    const mapped = map(value, v => v * 2)
    const chained = chain(mapped, v => fromNullable(v + 1))

    expect(unwrap(chained)).toBe(5)
  })

  it('filters and matches', () => {
    const value = fromNullable(10)
    const filtered = filter(value, v => v > 20)

    expect(isNone(filtered)).toBe(true)

    const matched = match(value, {
      None: () => 'none',
      Some: v => `some:${v}`,
    })

    expect(matched).toBe('some:10')
  })

  it('unwrapOr helpers return defaults', () => {
    const none = fromNullable(null)
    expect(unwrapOr(none, 5)).toBe(5)
    expect(unwrapOrElse(none, () => 7)).toBe(7)
  })

  it('unwrap throws on none', () => {
    const none = fromNullable(null)
    expect(() => unwrap(none)).toThrow('Called unwrap on None')
  })
})
