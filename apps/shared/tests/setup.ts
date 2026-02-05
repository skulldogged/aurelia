import '@testing-library/jest-dom/vitest'
import { afterEach, beforeEach } from 'vitest'

interface StorageMock extends Storage {
  _store: Map<string, string>
}

const createStorageMock = (): StorageMock => {
  const store = new Map<string, string>()
  return {
    _store: store,
    clear: () => {
      store.clear()
    },
    getItem: key => (store.has(key) ? store.get(key)! : null),
    key: index => Array.from(store.keys())[index] ?? null,
    get length() {
      return store.size
    },
    removeItem: key => {
      store.delete(key)
    },
    setItem: (key, value) => {
      store.set(key, value)
    },
  } as StorageMock
}

const installStorage = (name: 'localStorage' | 'sessionStorage', value: Storage): void => {
  Object.defineProperty(globalThis, name, {
    configurable: true,
    value,
    writable:     true,
  })

  if (typeof window !== 'undefined') {
    Object.defineProperty(window, name, {
      configurable: true,
      value,
      writable:     true,
    })
  }
}

const localStorageMock = createStorageMock()
const sessionStorageMock = createStorageMock()
installStorage('localStorage', localStorageMock)
installStorage('sessionStorage', sessionStorageMock)

beforeEach(() => {
  localStorageMock.clear()
  sessionStorageMock.clear()
})

import { ensureTestStorage } from '@/test-utils/storage'

ensureTestStorage()

afterEach(() => {
  localStorageMock.clear()
  sessionStorageMock.clear()
})
