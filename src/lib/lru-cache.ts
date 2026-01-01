/**
 * Simple LRU (Least Recently Used) cache implementation.
 *
 * Provides bounded caching with automatic eviction of least recently used items
 * when the cache exceeds its maximum size.
 */
export class LRUCache<K, V> {
  private cache: Map<K, V>
  private readonly maxSize: number

  constructor(maxSize: number) {
    this.cache = new Map()
    this.maxSize = maxSize
  }

  /**
   * Get a value from the cache.
   * Accessing a value moves it to the "most recently used" position.
   */
  get(key: K): V | undefined {
    if (!this.cache.has(key)) {
      return undefined
    }

    // Move to end (most recently used) by deleting and re-adding
    const value = this.cache.get(key)!
    this.cache.delete(key)
    this.cache.set(key, value)
    return value
  }

  /**
   * Check if a key exists in the cache (without affecting LRU order)
   */
  has(key: K): boolean {
    return this.cache.has(key)
  }

  /**
   * Set a value in the cache.
   * If the cache is full, the least recently used item is evicted.
   */
  set(key: K, value: V): void {
    // If key exists, delete first to update LRU position
    if (this.cache.has(key)) {
      this.cache.delete(key)
    }
    // If at capacity, delete the oldest (first) entry
    else if (this.cache.size >= this.maxSize) {
      const oldestKey = this.cache.keys().next().value
      if (oldestKey !== undefined) {
        this.cache.delete(oldestKey)
      }
    }

    this.cache.set(key, value)
  }

  /**
   * Delete a key from the cache
   */
  delete(key: K): boolean {
    return this.cache.delete(key)
  }

  /**
   * Delete all entries whose keys start with a given prefix (for string keys)
   */
  deleteByPrefix(prefix: string): void {
    for (const key of this.cache.keys()) {
      if (typeof key === 'string' && key.startsWith(prefix)) {
        this.cache.delete(key)
      }
    }
  }

  /**
   * Clear the entire cache
   */
  clear(): void {
    this.cache.clear()
  }

  /**
   * Get the current size of the cache
   */
  get size(): number {
    return this.cache.size
  }

  /**
   * Iterate over all keys
   */
  keys(): IterableIterator<K> {
    return this.cache.keys()
  }

  /**
   * Iterate over all values
   */
  values(): IterableIterator<V> {
    return this.cache.values()
  }

  /**
   * Iterate over all entries
   */
  entries(): IterableIterator<[K, V]> {
    return this.cache.entries()
  }
}
