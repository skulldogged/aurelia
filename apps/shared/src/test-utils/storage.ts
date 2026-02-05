type StorageLike = {
  getItem: (key: string) => string | null
  setItem: (key: string, value: string) => void
  removeItem: (key: string) => void
  clear: () => void
  key?: (index: number) => string | null
  length?: number
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
    getItem: key => store.has(key) ? store.get(key)! : null,
    setItem: (key, value) => {
      store.set(key, String(value))
    },
    removeItem: key => {
      store.delete(key)
    },
    clear: () => {
      store.clear()
    },
    key: index => Array.from(store.keys())[index] ?? null,
    get length() {
      return store.size
    },
  }
}

const ensureStorage = (key: 'localStorage' | 'sessionStorage'): void => {
  const existing = (globalThis as Record<string, unknown>)[key]
  if (!isStorageLike(existing)) {
    const fallback = createMemoryStorage()
    Object.defineProperty(globalThis, key, {
      value: fallback,
      configurable: true,
      writable: true,
    })
    if (typeof window !== 'undefined') {
      Object.defineProperty(window, key, {
        value: fallback,
        configurable: true,
        writable: true,
      })
    }
  }
}

export const ensureTestStorage = (): void => {
  ensureStorage('localStorage')
  ensureStorage('sessionStorage')
}
