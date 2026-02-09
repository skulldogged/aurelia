type StorageLike = {
  clear:      () => void
  getItem:    (key: string) => null | string
  key?:       (index: number) => null | string
  length?:    number
  removeItem: (key: string) => void
  setItem:    (key: string, value: string) => void
}

const isStorageLike = (value: unknown): value is StorageLike => {
  if (!value || typeof value !== 'object')
    return false

  const storage = value as StorageLike
  return typeof storage.getItem === 'function'
    && typeof storage.setItem === 'function'
    && typeof storage.removeItem === 'function'
    && typeof storage.clear === 'function'
}

const createMemoryStorage = (): StorageLike => {
  const store = new Map<string, string>()

  return {
    clear: () => {
      store.clear()
    },
    getItem: key => store.has(key) ? store.get(key)! : null,
    key:     index => Array.from(store.keys())[index] ?? null,
    get length() {
      return store.size
    },
    removeItem: key => {
      store.delete(key)
    },
    setItem: (key, value) => {
      store.set(key, String(value))
    },
  }
}

const ensureStorage = (key: 'localStorage' | 'sessionStorage'): void => {
  const existing = (globalThis as Record<string, unknown>)[key]
  if (!isStorageLike(existing)) {
    const fallback = createMemoryStorage()
    Object.defineProperty(globalThis, key, {
      configurable: true,
      value:        fallback,
      writable:     true,
    })
    if (typeof window !== 'undefined') {
      Object.defineProperty(window, key, {
        configurable: true,
        value:        fallback,
        writable:     true,
      })
    }
  }
}

export const ensureTestStorage = (): void => {
  ensureStorage('localStorage')
  ensureStorage('sessionStorage')
}
