/**
 * Functional Programming Utilities
 *
 * This module provides common functional programming utilities to promote
 * immutability, composition, and pure functions throughout the codebase.
 */

/**
 * Identity function - returns its argument unchanged
 */
export const identity = <T>(x: T): T => x

/**
 * Constant function - always returns the same value
 */
export const constant = <T>(x: T) => (): T => x

/**
 * Function composition - compose(f, g)(x) = f(g(x))
 */
export const compose = <A, B, C>(f: (b: B) => C, g: (a: A) => B) => (x: A): C => f(g(x))

/**
 * Pipe function - pipe(x, f, g, h) = h(g(f(x)))
 */
export const pipe = <T>(x: T, ...fns: Array<(arg: T) => T>): T =>
  fns.reduce((acc, fn) => fn(acc), x)

/**
 * Curry a function of 2 arguments
 */
export const curry2 = <A, B, C>(fn: (a: A, b: B) => C) => (a: A) => (b: B): C => fn(a, b)

/**
 * Curry a function of 3 arguments
 */
export const curry3 = <A, B, C, D>(fn: (a: A, b: B, c: C) => D) => (a: A) => (b: B) => (c: C): D => fn(a, b, c)

/**
 * Flip arguments of a binary function
 */
export const flip = <A, B, C>(fn: (a: A, b: B) => C) => (b: B, a: A): C => fn(a, b)

/**
 * Tap function - perform side effect and return original value
 */
export const tap = <T>(fn: (x: T) => void) => (x: T): T => {
  fn(x)
  return x
}

/**
 * When predicate - apply function only if condition is met
 */
export const when = <T>(predicate: (x: T) => boolean, fn: (x: T) => T) => (x: T): T =>
  predicate(x) ? fn(x) : x

/**
 * Unless predicate - apply function only if condition is not met
 */
export const unless = <T>(predicate: (x: T) => boolean, fn: (x: T) => T) => (x: T): T =>
  predicate(x) ? x : fn(x)

/**
 * Apply function n times
 */
export const times = <T>(n: number, fn: (x: T) => T) => (x: T): T => {
  let result = x
  for (let i = 0; i < n; i++)
    result = fn(result)

  return result
}

/**
 * Memoize a function
 */
export const memoize = <TArgs extends readonly unknown[], TReturn>(
  fn: (...args: TArgs) => TReturn,
): (...args: TArgs) => TReturn => {
  const cache = new Map<string, TReturn>()
  return (...args: TArgs): TReturn => {
    const key = JSON.stringify(args)
    if (cache.has(key)) {
      return cache.get(key)!
    }
    const result = fn(...args)
    cache.set(key, result)
    return result
  }
}

/**
 * Create a function that always returns the same value (K combinator)
 */
export const always = <T>(x: T) => (): T => x

/**
 * Logical not as a function
 */
export const not = (x: boolean): boolean => !x

/**
 * Check if value is truthy
 */
export const isTruthy = <T>(x: T): x is NonNullable<T> => Boolean(x)

/**
 * Check if value is falsy
 */
export const isFalsy = <T>(x: T): boolean => !x