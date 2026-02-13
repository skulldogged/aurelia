/**
 * Immutability Helpers for Functional State Updates
 *
 * These utilities provide functional ways to update nested objects and arrays
 * immutably, promoting predictable state management.
 */

/**
 * Update a property of an object immutably
 */
export const set = <T extends Record<string, unknown>, K extends keyof T>(
  obj: T,
  key: K,
  value: T[K],
): T => ({ ...obj, [key]: value })

export const setIn = <T>(
  obj: T,
  path: (number | string)[],
  value: unknown,
): T => {
  if (path.length === 0) return value as T

  const [head, ...tail] = path

  if (Array.isArray(obj)) {
    const index = head as number
    const newArray = [...obj]
    newArray[index] = tail.length === 0 ? value : setIn(obj[index], tail, value)
    return newArray as T
  }

  if (typeof obj === 'object' && obj !== null)
    return {
      ...obj,
      [head]: tail.length === 0
        ? value
        : setIn((obj as Record<number | string, unknown>)[head], tail, value),
    } as T

  throw new Error('Cannot set property on non-object')
}

/**
 * Get a nested property using a path
 */
export const getIn = <T>(obj: unknown, path: (number | string)[]): T | undefined =>
  path.reduce((current, key) => (current as Record<number | string, unknown> | undefined)?.[key], obj) as T | undefined

/**
 * Update an object using a function
 */
export const update = <T extends Record<string, unknown>, K extends keyof T>(
  obj: T,
  key: K,
  updater: (value: T[K]) => T[K],
): T => ({ ...obj, [key]: updater(obj[key]) })

/**
 * Update nested properties using a path and updater function
 */
export const updateIn = <T>(
  obj: T,
  path: (number | string)[],
  updater: (value: unknown) => unknown,
): T => {
  if (path.length === 0) return updater(obj) as T

  const [head, ...tail] = path

  if (Array.isArray(obj)) {
    const index = head as number
    const newArray = [...obj]
    newArray[index] = tail.length === 0 ? updater(obj[index]) : updateIn(obj[index], tail, updater)
    return newArray as T
  }

  if (typeof obj === 'object' && obj !== null) {
    return {
      ...obj,
      [head]: tail.length === 0
        ? updater((obj as Record<number | string, unknown>)[head])
        : updateIn(
          (obj as Record<number | string, unknown>)[head],
          tail,
          updater,
        ),
    } as T
  }

  throw new Error('Cannot update property on non-object')
}

/**
 * Append to an array immutably
 */
export const append = <T>(array: T[], item: T): T[] => [...array, item]

/**
 * Prepend to an array immutably
 */
export const prepend = <T>(array: T[], item: T): T[] => [item, ...array]

/**
 * Remove item from array by index immutably
 */
export const removeAt = <T>(array: T[], index: number): T[] =>
  [...array.slice(0, index), ...array.slice(index + 1)]

/**
 * Insert item at index immutably
 */
export const insertAt = <T>(array: T[], index: number, item: T): T[] =>
  [...array.slice(0, index), item, ...array.slice(index)]

/**
 * Replace item at index immutably
 */
export const replaceAt = <T>(array: T[], index: number, item: T): T[] =>
  array.map((current, i) => (i === index ? item : current))

/**
 * Filter array immutably (returns new array)
 */
export const filter = <T>(array: T[], predicate: (item: T) => boolean): T[] =>
  array.filter(predicate)

/**
 * Map array immutably (returns new array)
 */
export const map = <T, U>(array: T[], mapper: (item: T) => U): U[] =>
  array.map(mapper)

/**
 * Sort array immutably (returns new array)
 */
export const sort = <T>(array: T[], compareFn: (a: T, b: T) => number): T[] =>
  [...array].sort(compareFn)

/**
 * Reverse array immutably (returns new array)
 */
export const reverse = <T>(array: T[]): T[] => [...array].reverse()

/**
 * Merge objects immutably (shallow)
 */
export const merge = <T extends Record<string, unknown>>(target: T, source: Partial<T>): T =>
  ({ ...target, ...source })

/**
 * Omit properties from object immutably
 */
export const omit = <T extends Record<string, unknown>, K extends keyof T>(
  obj: T,
  keys: K[],
): Omit<T, K> => {
  const result = { ...obj }
  keys.forEach(key => delete result[key])
  return result
}

/**
 * Pick properties from object immutably
 */
export const pick = <T extends Record<string, unknown>, K extends keyof T>(
  obj: T,
  keys: K[],
): Pick<T, K> => {
  const result = {} as Pick<T, K>
  keys.forEach(key => {
    if (key in obj) {
      result[key] = obj[key]
    }
  })
  return result
}