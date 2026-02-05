import { Effect, Layer } from 'effect'

import { StorageError, toErrorMessage } from '../errors'

const getBrowserStorage = (): Storage => {
  if (typeof window === 'undefined' || !window.localStorage) {
    throw new Error('localStorage is unavailable in this runtime')
  }

  return window.localStorage
}

const tryStorage = <T>(
  key: string,
  operation: 'get' | 'remove' | 'set',
  fn: () => T,
): Effect.Effect<T, StorageError> =>
  Effect.try({
    catch: cause =>
      new StorageError({
        cause,
        key,
        message: toErrorMessage(cause),
        operation,
      }),
    try: fn,
  })

export interface StorageService {
  get:     (key: string) => Effect.Effect<null | string, StorageError>
  getJson: <T>(key: string) => Effect.Effect<null | T, StorageError>
  remove:  (key: string) => Effect.Effect<void, StorageError>
  set:     (key: string, value: string) => Effect.Effect<void, StorageError>
  setJson: (key: string, value: unknown) => Effect.Effect<void, StorageError>
}

export class StorageServiceTag extends Effect.Tag('aurelia/StorageService')<
  StorageServiceTag,
  StorageService
>() {}

const makeStorageService = (): StorageService => ({
  get: key => tryStorage(key, 'get', () => getBrowserStorage().getItem(key)),
  getJson: <T>(key: string) =>
    tryStorage(key, 'get', () => {
      const raw = getBrowserStorage().getItem(key)
      if (raw == null) {
        return null
      }

      return JSON.parse(raw) as T
    }),
  remove: key =>
    tryStorage(key, 'remove', () => {
      getBrowserStorage().removeItem(key)
    }),
  set: (key, value) =>
    tryStorage(key, 'set', () => {
      getBrowserStorage().setItem(key, value)
    }),
  setJson: (key, value) =>
    tryStorage(key, 'set', () => {
      getBrowserStorage().setItem(key, JSON.stringify(value))
    }),
})

export const StorageServiceLive = Layer.succeed(StorageServiceTag, makeStorageService())

export const getStorageValue = (key: string): Effect.Effect<null | string, StorageError, StorageServiceTag> =>
  Effect.flatMap(StorageServiceTag, service => service.get(key))

export const getStorageJson = <T>(key: string): Effect.Effect<null | T, StorageError, StorageServiceTag> =>
  Effect.flatMap(StorageServiceTag, service => service.getJson<T>(key))

export const removeStorageValue = (key: string): Effect.Effect<void, StorageError, StorageServiceTag> =>
  Effect.flatMap(StorageServiceTag, service => service.remove(key))

export const setStorageValue = (key: string, value: string): Effect.Effect<void, StorageError, StorageServiceTag> =>
  Effect.flatMap(StorageServiceTag, service => service.set(key, value))

export const setStorageJson = (key: string, value: unknown): Effect.Effect<void, StorageError, StorageServiceTag> =>
  Effect.flatMap(StorageServiceTag, service => service.setJson(key, value))
