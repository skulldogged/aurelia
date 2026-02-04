import { describe, expect, it, vi } from 'vitest'

import {
  andThenAsync,
  err,
  fromPromise,
  isErr,
  isOk,
  map,
  mapErr,
  ok,
  unwrap,
  unwrapErr,
  withCustomState,
} from '../src/lib/result'

describe('result utilities', () => {
  it('creates ok and err results', () => {
    const success = ok(42)
    const failure = err('boom')

    expect(isOk(success)).toBe(true)
    expect(isErr(failure)).toBe(true)
  })

  it('maps values and errors', () => {
    const success = map(ok(2), value => value * 3)
    const failure = mapErr(err('oops'), value => `${value}!`)

    expect(unwrap(success)).toBe(6)
    expect(unwrapErr(failure)).toBe('oops!')
  })

  it('andThenAsync short-circuits on error', async () => {
    const handler = vi.fn(async () => ok('ok'))
    const result = await andThenAsync(err('nope'), handler)

    expect(isErr(result)).toBe(true)
    expect(handler).not.toHaveBeenCalled()
  })

  it('fromPromise captures errors', async () => {
    const success = await fromPromise(Promise.resolve('done'))
    const failure = await fromPromise(Promise.reject(new Error('bad')))

    expect(isOk(success)).toBe(true)
    expect(isErr(failure)).toBe(true)
    expect(unwrapErr(failure)).toBe('bad')
  })

  it('withCustomState runs callbacks', async () => {
    const onStart = vi.fn()
    const onSuccess = vi.fn()
    const onFinally = vi.fn()

    const result = await withCustomState(() => Promise.resolve(ok('value')), {
      onFinally,
      onStart,
      onSuccess,
    })

    expect(isOk(result)).toBe(true)
    expect(onStart).toHaveBeenCalled()
    expect(onSuccess).toHaveBeenCalledWith('value')
    expect(onFinally).toHaveBeenCalled()
  })
})
