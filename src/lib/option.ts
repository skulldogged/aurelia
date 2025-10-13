/**
 * Option/Maybe Type for Functional Programming
 *
 * This provides a functional alternative to null/undefined checks,
 * inspired by languages like Haskell, Scala, and Rust.
 */

export interface None {
  readonly _tag: 'None'
}

export type Option<T> = None | Some<T>

export interface Some<T> {
  readonly _tag:  'Some'
  readonly value: T
}

const some = <T>(value: T): Option<T> => ({ _tag: 'Some', value })
const none: Option<never> = { _tag: 'None' }

/**
 * Create an Option from a nullable value
 */
export const fromNullable = <T>(value: null | T | undefined): Option<T> =>
  value == null ? none : some(value)

/**
 * Create an Option from a value and predicate
 */
export const fromPredicate = <T>(value: T, predicate: (value: T) => boolean): Option<T> =>
  predicate(value) ? some(value) : none

/**
 * Check if Option is Some
 */
export const isSome = <T>(option: Option<T>): option is Some<T> => option._tag === 'Some'

/**
 * Check if Option is None
 */
export const isNone = <T>(option: Option<T>): option is None => option._tag === 'None'

/**
 * Get the value from Some, throw error if None
 */
export const unwrap = <T>(option: Option<T>): T => {
  if (isNone(option)) {
    throw new Error('Called unwrap on None')
  }
  return option.value
}

/**
 * Get the value from Some, return default if None
 */
export const unwrapOr = <T>(option: Option<T>, defaultValue: T): T =>
  isSome(option) ? option.value : defaultValue

/**
 * Get the value from Some, compute default if None
 */
export const unwrapOrElse = <T>(option: Option<T>, defaultFn: () => T): T =>
  isSome(option) ? option.value : defaultFn()

/**
 * Map over Option
 */
export const map = <T, U>(option: Option<T>, fn: (value: T) => U): Option<U> =>
  isSome(option) ? some(fn(option.value)) : none

/**
 * Chain Options (flatMap)
 */
export const chain = <T, U>(option: Option<T>, fn: (value: T) => Option<U>): Option<U> =>
  isSome(option) ? fn(option.value) : none

/**
 * Filter Option by predicate
 */
export const filter = <T>(option: Option<T>, predicate: (value: T) => boolean): Option<T> =>
  isSome(option) && predicate(option.value) ? option : none

/**
 * Pattern matching for Option
 */
export const match = <T, U>(
  option: Option<T>,
  patterns: {
    None: () => U
    Some: (value: T) => U
  },
): U => (isSome(option) ? patterns.Some(option.value) : patterns.None())

/**
 * Fold Option into a single value
 */
export const fold = <T, U>(
  option: Option<T>,
  onNone: () => U,
  onSome: (value: T) => U,
): U => (isSome(option) ? onSome(option.value) : onNone())

/**
 * Get the first Some from a list of Options
 */
export const firstSome = <T>(options: Option<T>[]): Option<T> => {
  for (const option of options) {
    if (isSome(option)) return option
  }
  return none
}

/**
 * Convert Option to Result-like structure
 */
export const toResult = <T, E>(option: Option<T>, error: E): { data?: T; error?: E } =>
  isSome(option) ? { data: option.value } : { error }

/**
 * Apply function to Option if Some
 */
export const tap = <T>(option: Option<T>, fn: (value: T) => void): Option<T> => {
  if (isSome(option)) fn(option.value)
  return option
}

/**
 * Option constructors
 */
export const Option = {
  fromNullable,
  fromPredicate,
  none,
  some,
} as const