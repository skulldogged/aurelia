import '@testing-library/jest-dom/vitest'
import { afterEach, beforeEach } from 'vitest'

const createStorageMock = (): Storage => {
  const store = new Map<string, string>()
  return {
    clear: () => {
      store.clear()
    },
    getItem: (key: string) => store.get(key) ?? null,
    key: (index: number) => Array.from(store.keys())[index] ?? null,
    get length() {
      return store.size
    },
    removeItem: (key: string) => {
      store.delete(key)
    },
    setItem: (key: string, value: string) => {
      store.set(key, value)
    },
  } as Storage
}

const installStorage = (name: 'localStorage' | 'sessionStorage', value: Storage): void => {
  Object.defineProperty(globalThis, name, {
    configurable: true,
    value,
    writable: true,
  })
  if (typeof window !== 'undefined') {
    Object.defineProperty(window, name, {
      configurable: true,
      value,
      writable: true,
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

afterEach(() => {
  localStorageMock.clear()
  sessionStorageMock.clear()
})
